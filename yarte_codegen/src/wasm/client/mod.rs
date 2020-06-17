#![allow(unused_variables, dead_code)]
#![allow(clippy::too_many_arguments)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    iter, mem,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse2, parse_str, punctuated::Punctuated, Field, FieldValue, Ident, Member, Meta, MetaList,
    NestedMeta, Path, Token, Type,
};

use yarte_dom::dom::{
    Attribute, Document, Element, ExprId, ExprOrText, Expression, IfBlock, IfElse, Node, TreeMap,
    Var, VarId, VarInner, VarMap, DOM,
};
use yarte_hir::{Struct, HIR};

use crate::CodeGen;

mod component;
mod each;
mod events;
mod leaf_text;
mod messages;
mod state;
mod utils;

#[cfg(test)]
mod tests;

use self::{
    component::clean,
    leaf_text::get_leaf_text,
    state::{BlackBox, PAttr, Parent, State, Step},
    utils::*,
};

/// Stack automaton for parse DOM representation and generate TokenStream
/// Theory: https://core.ac.uk/download/pdf/82195817.pdf
///
/// Abstract
///
/// > The stack automaton has a two-way input tape, a finite control and a stack.
/// > The stack is similar to a push-down store, in that writing and erasing occur only at the top.
/// > However, the stack head may also move up or down the stack in a read only mode.
/// > Here, nonerasing stack automata only, are considered.
/// > These are stack automata that never erase a symbol from their stack.
/// > It is shown that the deterministic, nonerasing stack automaton is equivalent
/// > to a deterministic, off-line Turing machine whose storage tape never
/// > grows beyond n logz n cells where n is the length of the input.
///
///
/// Deterministic it's equivalent n log n-bounded Turing Machine
/// and that accepts context sensitive languages
pub struct WASMCodeGen<'a> {
    /// State
    stack: Vec<State>,
    /// unique
    count: usize,
    /// Derive struct
    s: &'a Struct<'a>,
    /// Variables grouped by base field
    grouped_map: HashMap<VarId, BTreeSet<VarId>>,
    /// Expresion -> Inner Variables
    tree_map: TreeMap,
    /// VarId -> Variable details
    var_map: HashMap<VarId, VarInner>,
    /// Helpers buffer
    helpers: TokenStream,
    /// Components buffer
    component: Vec<(Ident, TokenStream)>,
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        let mut bases = HashSet::new();
        bases.insert(get_self_id());
        let state = State::new(bases);
        WASMCodeGen {
            component: vec![],
            count: 0,
            grouped_map: Default::default(),
            helpers: TokenStream::new(),
            s,
            stack: vec![state],
            tree_map: Default::default(),
            var_map: Default::default(),
        }
    }

    // Getters
    fn get_bb_t_root<I: Iterator<Item = VarId>>(&self, parents: I) -> (TokenStream, usize) {
        let len = parents.fold(0, |acc, x| {
            acc + self
                .grouped_map
                .get(&x)
                .map(|x| x.len())
                .expect("grouped variables")
        });

        get_t_root_type(len)
    }

    #[inline]
    fn get_current_bb(&self) -> TokenStream {
        match &last!(self).id {
            Parent::Expr(id) => {
                let ident = get_vdom_ident(*id);
                quote!(#ident)
            }
            _ => {
                let ident = self.get_global_bb_ident();
                quote!(self.#ident)
            }
        }
    }

    #[inline]
    fn get_global_bb_ident(&self) -> Ident {
        self.s
            .fields
            .iter()
            .find_map(|x| {
                if is_black_box(&x.ty) {
                    Some(x.ident.clone().unwrap())
                } else {
                    None
                }
            })
            .expect("Black box field")
    }

    fn get_initial_state(&self) -> TokenStream {
        let attr: PAttr = parse2(quote!(#[serde(default)])).unwrap();
        let fields = self
            .s
            .fields
            .iter()
            .filter(|x| is_state(x))
            .map(|x| {
                let mut f = x.clone();
                f.attrs.extend(attr.0.clone());
                f
            })
            .fold(Punctuated::<Field, Token![,]>::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        let name = format_ident!("{}InitialState", self.s.ident);
        quote! {
            #[derive(Default, serde::Deserialize)]
            struct #name {
                #fields
            }
        }
    }

    // TODO: Expressions in path
    fn get_parent_node(&self) -> usize {
        last!(self)
            .steps
            .iter()
            .rposition(|x| match x {
                Step::FirstChild => true,
                _ => false,
            })
            .unwrap_or_default()
    }

    #[inline]
    fn get_state_fields(&self) -> Punctuated<&Ident, Token![,]> {
        self.s.fields.iter().filter(|x| is_state(x)).fold(
            <Punctuated<&Ident, Token![,]>>::new(),
            |mut acc, x| {
                acc.push(&x.ident.as_ref().expect("Named fields"));
                acc
            },
        )
    }

    #[inline]
    fn get_inner(&self) -> Punctuated<FieldValue, Token![,]> {
        self.s.fields.iter().filter(|x| is_inner(&x.attrs)).fold(
            <Punctuated<FieldValue, Token![,]>>::new(),
            |mut acc, x| {
                let expr = x
                    .attrs
                    .iter()
                    .find_map(|x| {
                        if x.path.is_ident("inner") {
                            match x.parse_meta() {
                                Ok(Meta::Path(p)) => Some(quote!(Default::default())),
                                Ok(Meta::List(MetaList { nested, .. })) => {
                                    assert_eq!(nested.len(), 1);
                                    if let NestedMeta::Lit(lit) = &nested[0] {
                                        if let syn::Lit::Str(l) = lit {
                                            let path: Path = parse_str(&l.value()).expect("path");
                                            return Some(quote!(#path()));
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                    .expect("valid inner attribute");
                acc.push(FieldValue {
                    attrs: vec![],
                    member: x.ident.clone().map(Member::Named).expect("Named fields"),
                    colon_token: Some(<Token![:]>::default()),
                    expr: parse2(expr).expect("valid expression"),
                });
                acc
            },
        )
    }

    #[inline]
    fn get_checks(&self, check: BTreeMap<VarId, (Vec<usize>, usize)>) -> TokenStream {
        let mut buff: Vec<TokenStream> = check
            .into_iter()
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<Option<ExprId>, (Vec<usize>, usize)>,
                 (base, (positions, len))| {
                    acc.entry(self.stack.iter().rev().find_map(|x| match &x.id {
                        Parent::Expr(id) if x.bases.contains(&base) => Some(*id),
                        _ => None,
                    }))
                    .and_modify(|x| {
                        let len = x.1;
                        x.0.extend(positions.iter().copied().map(|i| len + i));
                        x.1 += len;
                    })
                    .or_insert((positions, len));
                    acc
                },
            )
            .into_iter()
            .map(|(i, (x, len))| {
                let (t_root, len) = get_t_root_type(len);
                let mut bits = vec![false; len];
                for i in x {
                    bits[i] = true;
                }
                let number = match len {
                    8 => {
                        let number = get_number_u8(bits);
                        quote!(#number)
                    }
                    16 => {
                        let number = get_number_u16(bits);
                        quote!(#number)
                    }
                    32 => {
                        let number = get_number_u32(&bits);
                        quote!(#number)
                    }
                    64 => {
                        let tokens = get_split_32(&bits);
                        quote!(yarte_wasm_app::U64([#tokens]))
                    }
                    128 => {
                        let tokens = get_split_32(&bits);
                        quote!(yarte_wasm_app::U128([#tokens]))
                    }
                    256 => {
                        let tokens = get_split_32(&bits);
                        quote!(yarte_wasm_app::U256([#tokens]))
                    }
                    _ => todo!("more than 256 variables per context"),
                };

                let vdom = if let Some(i) = i {
                    let ident = get_vdom_ident(i);
                    quote!(#ident)
                } else {
                    let bb = self.get_global_bb_ident();
                    quote!(self.#bb)
                };

                quote!(yarte_wasm_app::YNumber::neq_zero(#vdom.t_root & #number))
            })
            .collect();
        let mut buff = buff.drain(..);
        let tokens = buff.next().unwrap_or_default();
        buff.fold(tokens, |mut acc, t| {
            acc.extend(quote!(|| #t));
            acc
        })
    }

    #[inline]
    fn get_check_hash(
        &self,
        checks: BTreeMap<VarId, Vec<VarId>>,
    ) -> BTreeMap<VarId, (Vec<usize>, usize)> {
        checks
            .into_iter()
            .map(|(i, deps)| {
                let group = self.grouped_map.get(&i).expect("group registred");
                let len = group.len();
                let deps: Vec<usize> = deps
                    .into_iter()
                    .map(|a| group.iter().position(|b| a == *b).expect("var in group"))
                    .collect();
                (i, (deps, len))
            })
            .collect()
    }

    fn get_render(&self, curr: &State) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (i, t) in curr.get_render_hash().into_iter() {
            let mut checks: BTreeMap<VarId, Vec<VarId>> = BTreeMap::new();
            for j in i {
                let base = self.var_map.get(&j).unwrap().base;
                checks
                    .entry(base)
                    .and_modify(|x| {
                        x.push(j);
                    })
                    .or_insert_with(|| vec![j]);
            }
            let checks = self.get_checks(self.get_check_hash(checks));
            if checks.is_empty() {
                tokens.extend(t);
            } else {
                tokens.extend(quote!(if #checks { #t }));
            }
        }

        tokens
    }

    // Inits
    #[inline]
    // TODO
    fn init_build(&self, build: TokenStream) -> TokenStream {
        let ident = format_ident!("{}InitialState", self.s.ident);
        let args = self.get_state_fields();

        quote! {
            let #ident { #args } = yarte_wasm_app::from_str(&get_state()).unwrap_or_default();
            let doc = yarte_wasm_app::web::window().unwrap_throw().document().unwrap_throw();
            #build
        }
    }

    #[inline]
    // TODO
    fn init_hydrate(curr: &mut State) -> TokenStream {
        if curr.buff_hydrate.is_empty() {
            quote!()
        } else {
            let body = get_body_ident();
            let mut hydrate = quote! {
                let #body = yarte_wasm_app::web::window().unwrap_throw()
                    .document().unwrap_throw()
                    .body().unwrap_throw();
            };
            // Get step for events
            let steps = get_steps(curr.path_events.iter(), quote!(#body));

            // Ended 'hydrate' buffer
            hydrate.extend(steps);
            hydrate.extend(curr.buff_hydrate.drain(..).flatten());
            hydrate
        }
    }

    #[inline]
    // TODO
    fn init_render(&self, curr: &mut State) -> TokenStream {
        let name = self.get_global_bb_ident();
        let (base, _) = self.get_bb_t_root(iter::once(get_self_id()));
        let render = self.get_render(curr);
        let render = quote! {
            if self.#name.t_root == <#base as yarte_wasm_app::YNumber>::zero() {
                return;
            }
            #render
        };
        curr.add_t_root(base);

        render
    }

    #[inline]
    fn init(&mut self, mut dom: DOM) -> TokenStream {
        self.resolve_tree_var(dom.tree_map, dom.var_map);
        let mut build = TokenStream::new();

        assert_eq!(dom.doc.len(), 1);
        match dom.doc.remove(0) {
            Node::Elem(Element::Node { name, children, .. }) => {
                assert_eq!(ExprOrText::Text("html".into()), name.1);
                assert!(children.iter().all(|x| match x {
                    Node::Elem(Element::Node { name, .. }) => match &name.1 {
                        ExprOrText::Text(s) => s == "body" || s == "head",
                        _ => false,
                    },
                    Node::Elem(Element::Text(text)) => text.chars().all(|x| x.is_whitespace()),
                    _ => false,
                }));

                let (head, body) = children.into_iter().fold((None, None), |acc, x| match x {
                    Node::Elem(Element::Node { name, children, .. }) => match &name.1 {
                        ExprOrText::Text(s) => match s.as_ref() {
                            "body" => (acc.0, Some(children)),
                            "head" => (Some(children), acc.1),
                            _ => acc,
                        },
                        _ => acc,
                    },
                    _ => acc,
                });
                if let Some(head) = head {
                    self.step(head);
                    if !last!(self).path_nodes.is_empty() {
                        todo!("in head expressions")
                    }
                }
                if let Some(body) = body {
                    last_mut!(self).id = Parent::Body;
                    self.step(body);
                    if !last!(self).path_nodes.is_empty() {
                        let ident = get_body_ident();
                        let current = last_mut!(self);
                        let tokens = get_steps(current.path_nodes.iter(), quote!(#ident));
                        build.extend(quote!(let #ident = doc.body().unwrap_throw();));
                        build.extend(tokens);
                        build.extend(mem::take(&mut current.buff_build).into_iter().flatten());
                        current.path_nodes.clear();
                    }
                } else {
                    panic!("Need <body> tag")
                }
            }
            _ => panic!("Need html at root"),
        }

        build
    }

    // Main recursive loop
    fn step(&mut self, doc: Document) {
        let last_node = last!(self).steps.len();
        // TODO: Inline nodes
        let insert_points = doc.iter().filter(|x| match x {
            Node::Elem(Element::Text(_)) => false,
            _ => true,
        });
        let len = insert_points.clone().count();
        let insert_point = get_insert_point(insert_points);

        let mut last = 0usize;
        let nodes = doc.into_iter().map(|x| match x {
            Node::Elem(Element::Text(_)) => (last, x),
            _ => {
                let l = last;
                last += 1;
                (l, x)
            }
        });

        for (i, node) in nodes {
            match node {
                Node::Elem(Element::Node {
                    children, attrs, ..
                }) => {
                    last_mut!(self).steps.push(if i == 0 {
                        Step::FirstChild
                    } else {
                        Step::NextSibling
                    });
                    for attr in attrs {
                        self.resolve_attr(attr);
                    }

                    if all_children_text(children.iter()) {
                        self.write_leaf_text(children);
                    } else {
                        self.step(children);
                    }
                }
                Node::Expr(e) => match e {
                    Expression::Each(id, each) => {
                        self.gen_each(id, *each, len != 1, i == len, insert_point.split_at(i).0);
                        last_mut!(self).steps.push(Step::Each(id));
                    }
                    Expression::IfElse(id, if_else) => {
                        let IfElse { ifs, if_else, els } = *if_else;

                        self.resolve_if_block(ifs, id);
                        for b in if_else {
                            self.resolve_if_block(b, id);
                        }
                        if let Some(body) = els {
                            todo!("resolve if else block expresion");
                        }
                    }
                    Expression::Local(..) => todo!("resolve local expression"),
                    Expression::Safe(id, _) | Expression::Unsafe(id, _) => unreachable!(),
                },
                Node::Elem(Element::Text(_)) => (),
            }
        }

        last_mut!(self).steps.drain(last_node..);
    }

    #[inline]
    fn resolve_attr(&mut self, attr: Attribute) {
        if let Some(event) = is_on_attr(&attr) {
            let (id, msg) = match attr.value.as_slice() {
                [ExprOrText::Expr(Expression::Safe(id, msg))] => (id, &**msg),
                _ => panic!("only use resolve expressions `{{? .. }}` in on attributes"),
            };
            self.write_event(*id, event, msg);
        } else {
            match attr.name {
                ExprOrText::Expr(_) => todo!("name attribute expression"),
                ExprOrText::Text(_) => (),
            }
            for e in &attr.value {
                if let ExprOrText::Expr(e) = e {
                    todo!("expression in attribute")
                }
            }
        }
    }

    #[inline]
    fn resolve_if_block(&mut self, IfBlock { block, .. }: IfBlock, id: ExprId) {
        todo!("resolve if else block expresion");
    }

    #[inline]
    fn resolve_tree_var(&mut self, tree_map: TreeMap, var_map: VarMap) {
        let mut grouped = HashMap::new();
        let var_map: HashMap<VarId, VarInner> = var_map
            .into_iter()
            .filter_map(|(i, x)| match x {
                Var::This(x) => {
                    grouped
                        .entry(x.base)
                        .and_modify(|x: &mut BTreeSet<VarId>| {
                            x.insert(i);
                        })
                        .or_insert_with(|| {
                            // Need Order
                            let mut b = BTreeSet::new();
                            b.insert(i);
                            b
                        });
                    Some((i, x))
                }
                Var::Local(..) => None,
            })
            .collect();

        if grouped.get(&get_self_id()).is_none() {
            todo!("need any field in struct of application")
        }
        self.grouped_map = grouped;
        self.tree_map = tree_map;
        self.var_map = var_map;
    }

    // Clear buffer and return it
    // TODO: empty helpers
    #[inline]
    fn empty_components(&mut self) -> Punctuated<FieldValue, Token![,]> {
        self.component.drain(..).fold(
            <Punctuated<FieldValue, Token![,]>>::new(),
            |mut acc, (i, t)| {
                acc.push(FieldValue {
                    attrs: vec![],
                    member: Member::Named(i),
                    colon_token: Some(<Token![:]>::default()),
                    expr: parse2(quote!({ #t })).unwrap(),
                });

                acc
            },
        )
    }

    // Writes current state
    // TODO: whitespace and text node
    #[inline]
    fn write_leaf_text(&mut self, children: Document) {
        let (t, e) = get_leaf_text(children, &self.tree_map, &self.var_map);
        let name = self.current_node_ident(0);

        let dom = match &last!(self).id {
            Parent::Body => {
                let ident = self.get_global_bb_ident();
                quote!(self.#ident)
            }
            Parent::Expr(i) => {
                let ident = get_vdom_ident(*i);
                quote!(#ident)
            }
            Parent::Head => todo!(),
        };
        let current = last_mut!(self);
        current
            .buff_new
            .push(quote! { #name.set_text_content(Some(&#e)); });

        current
            .path_nodes
            .push((name.clone(), current.steps.clone()));
        current
            .buff_render
            .push((t, quote! { #dom.#name.set_text_content(Some(&#e)); }));
        current.black_box.push(BlackBox {
            doc: "Yarte Node element".into(),
            name,
            ty: parse2(quote!(yarte_wasm_app::web::Element)).unwrap(),
        });
    }

    // Registers
    fn current_node_ident(&mut self, init: usize) -> Ident {
        self.find_current_node(init).unwrap_or_else(|| {
            let id = self.count;
            self.count += 1;
            get_node_ident(id)
        })
    }

    fn find_current_node(&self, init: usize) -> Option<Ident> {
        let current = last!(self);
        let path = &current.steps[init..];
        current
            .path_nodes
            .iter()
            .chain(current.path_events.iter())
            .find_map(|(i, x)| {
                if path.eq(x.as_slice()) {
                    Some(i.clone())
                } else {
                    None
                }
            })
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        let build = self.init(ir.into());

        let mut curr = self.stack.pop().expect("one state");

        // Ended 'hydrate' buffer
        let hydrate = Self::init_hydrate(&mut curr);

        // Black box ident and type
        let bb_ident = self.get_global_bb_ident();
        let bb_type = format_ident!("{}BlackBox", self.s.ident);

        let mut render = self.init_render(&mut curr);
        // Ended 'render' buffer
        render.extend(quote! {
            self.#bb_ident.t_root = yarte_wasm_app::YNumber::zero();
        });

        // BlackBox
        // TODO: specify component type
        let component_type: Type = parse2(quote!(yarte_wasm_app::web::Element)).unwrap();
        let mut bb_field_value = curr.get_black_box_fields(&get_field_root_ident(), true);

        for (i, _) in &self.component {
            curr.black_box.push(BlackBox {
                doc: "Component".to_string(),
                name: i.clone(),
                ty: component_type.clone(),
            })
        }
        let black_box = curr.get_black_box(&bb_type);

        // Add components to black box fields value
        bb_field_value.extend(self.empty_components());
        let args = self.get_state_fields();
        let inner = self.get_inner();
        let mut fields = vec![];
        if !args.is_empty() {
            fields.push(args.into_token_stream());
        }
        if !inner.is_empty() {
            fields.push(inner.into_token_stream())
        }
        fields.push(quote! {
            #bb_ident: #bb_type { #bb_field_value }
        });

        let mut build = self.init_build(build);
        build.extend(quote! {
            Self { #(#fields),* }
        });

        // Into Default::default implementation
        // Ended 'build' buffer
        let build = self.s.implement_head(
            quote!(std::default::Default),
            &quote!(fn default() -> Self { #build }),
        );

        // Make messages for `App::__dispatch` implementation
        let msgs = self
            .s
            .msgs
            .as_ref()
            .expect("Need define messages for application");
        let msgs_type = &msgs.ident;
        let (dispatch, msgs) = messages::gen_messages(msgs);

        // Make App trait body
        let app = quote! {
            type BlackBox = #bb_type;
            type Message = #msgs_type;

            #[doc(hidden)]
            #[inline]
            fn __render(&mut self, __addr: &yarte_wasm_app::Addr<Self>) { # render }

            #[doc(hidden)]
            #[inline]
            fn __hydrate(&mut self, __addr: &yarte_wasm_app::Addr<Self>) { # hydrate }

            #[doc(hidden)]
            fn __dispatch(&mut self, __msg: Self::Message, __addr: &yarte_wasm_app::Addr<Self>) { #dispatch }
        };
        // Implement App trait
        let app = self.s.implement_head(quote!(yarte_wasm_app::App), &app);
        let helpers = &self.helpers;

        let initial_state = self.get_initial_state();

        // Multi app compilation
        clean();

        // Join buffers
        quote! {
            #[wasm_bindgen]
            extern "C" {
                fn get_state() -> String;
            }

            #app
            #msgs
            #initial_state
            #black_box
            #helpers
            #build
        }
    }
}
