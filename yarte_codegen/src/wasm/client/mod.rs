#![allow(unused_variables, dead_code)]
#![allow(clippy::ptr_arg, clippy::too_many_arguments)]

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    iter, mem,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer},
    parse2, parse_str,
    punctuated::Punctuated,
    Field, FieldValue, Ident, Member, Meta, MetaList, NestedMeta, Path, Token, Type, VisPublic,
    Visibility,
};

use yarte_dom::dom::{
    Attribute, Document, Each, Element, ExprId, ExprOrText, Expression, IfBlock, IfElse, Node,
    TreeMap, Var, VarId, VarInner, VarMap, DOM,
};
use yarte_helpers::helpers::calculate_hash;
use yarte_hir::{Struct, HIR};

use crate::CodeGen;

mod component;
mod each;
mod events;
mod leaf_text;
mod messages;

use self::{component::clean, leaf_text::get_leaf_text};

// TODO:
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Step {
    FirstChild,
    NextSibling,
    Each(usize),
}

impl ToTokens for Step {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Step::*;
        tokens.extend(match self {
            FirstChild => quote!(.first_element_child().unwrap_throw()),
            NextSibling => quote!(.next_element_sibling().unwrap_throw()),
            Each(_) => todo!(),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parent {
    Head,
    Body,
    Expr(ExprId),
}

impl Default for Parent {
    fn default() -> Self {
        Parent::Head
    }
}

#[derive(Clone, Debug)]
pub enum InsertPoint {
    Append(Vec<InsertPath>),
    LastBefore(Vec<InsertPath>, Vec<InsertPath>),
}

pub struct Len {
    base: usize,
    expr: Vec<ExprId>,
}

impl From<Vec<InsertPath>> for Len {
    fn from(i: Vec<InsertPath>) -> Self {
        use InsertPath::*;
        let base = i.iter().fold(0, |acc, x| match x {
            Before => acc + 1,
            _ => acc,
        });
        let expr = i.into_iter().fold(vec![], |mut acc, x| {
            if let Expr(i) = x {
                acc.push(i);
            }
            acc
        });

        Len { base, expr }
    }
}

#[derive(Clone, Debug)]
pub enum InsertPath {
    Before,
    Expr(ExprId),
}

type PathNode = (Ident, Vec<Step>);

#[derive(Debug, Default)]
struct State {
    id: Parent,
    bases: Vec<Base>,
    /// black box fields
    black_box: Vec<BlackBox>,
    /// Intermediate buffers
    buff_build: Vec<TokenStream>,
    buff_hydrate: Vec<TokenStream>,
    buff_new: Vec<TokenStream>,
    buff_render: Vec<(BTreeSet<VarId>, TokenStream)>,
    /// Path to nodes in current scope
    path_nodes: Vec<PathNode>,
    /// Path to events in current scope
    path_events: Vec<PathNode>,
    /// path to nodes
    steps: Vec<Step>,
}

pub struct WASMCodeGen<'a> {
    /// stack of PDA
    stack: Vec<State>,
    /// Added current node
    on_node: Option<usize>,
    // TODO: remove
    /// Output buffer
    build: TokenStream,
    helpers: TokenStream,
    hydrate: TokenStream,
    render: TokenStream,
    /// Components buffer
    component: Vec<(Ident, TokenStream)>,
    /// Derive struct
    s: &'a Struct<'a>,
    /// Hash of self
    self_id: VarId,
    /// Variables grouped by base field
    grouped_map: HashMap<VarId, BTreeSet<VarId>>,
    /// Expresion -> Inner Variables
    tree_map: TreeMap,
    /// VarId -> Variable details
    var_map: HashMap<VarId, VarInner>,
    /// unique
    count: usize,
}

#[derive(Debug, Clone)]
struct BlackBox {
    doc: String,
    name: Ident,
    ty: Type,
}

impl Into<Field> for BlackBox {
    fn into(self) -> Field {
        let BlackBox { name, ty, doc } = self;
        let attr: PAttr = parse2(quote!(#[doc = #doc])).unwrap();
        Field {
            attrs: attr.0,
            vis: Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            }),
            ident: Some(name),
            colon_token: Some(<Token![:]>::default()),
            ty,
        }
    }
}

fn is_black_box(ty: &Type) -> bool {
    let black: Type = parse2(quote!(<Self as Template>::BlackBox)).unwrap();
    ty.eq(&black)
}

fn is_inner(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("inner"))
}

fn is_state(Field { attrs, ty, .. }: &Field) -> bool {
    !(is_inner(attrs) || is_black_box(ty))
}

struct PAttr(Vec<syn::Attribute>);

impl Parse for PAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        Ok(PAttr(input.call(syn::Attribute::parse_outer)?))
    }
}

#[derive(Debug)]
enum Base {
    Add(VarId),
    Skip(VarId),
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        let self_id = calculate_hash(&"self");
        let state = State {
            bases: vec![Base::Add(self_id)],
            ..Default::default()
        };
        WASMCodeGen {
            build: TokenStream::new(),
            component: vec![],
            count: 0,
            grouped_map: Default::default(),
            helpers: TokenStream::new(),
            hydrate: TokenStream::new(),
            on_node: None,
            render: TokenStream::new(),
            s,
            self_id,
            stack: vec![state],
            tree_map: Default::default(),
            var_map: Default::default(),
        }
    }

    // Getters
    fn get_black_box_t_root<T: Iterator<Item = VarId>>(&self, parents: T) -> (TokenStream, usize) {
        let len = parents.fold(0, |acc, x| {
            acc + self
                .grouped_map
                .get(&x)
                .map(|x| x.len())
                .expect("grouped variables")
        });

        Self::get_t_root_type(len)
    }

    fn get_t_root_type(len: usize) -> (TokenStream, usize) {
        match len {
            0..=8 => (quote!(u8), 8),
            9..=16 => (quote!(u16), 16),
            17..=32 => (quote!(u32), 32),
            33..=64 => (quote!(yarte::U64), 64),
            65..=128 => (quote!(yarte::U128), 128),
            129..=256 => (quote!(yarte::U256), 256),
            _ => todo!("more than 256 variables per context"),
        }
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

    fn get_insert_point<'b, F: Iterator<Item = &'b Node>>(
        pos: (usize, usize),
        o: F,
    ) -> InsertPoint {
        if pos.0 + 1 == pos.1 {
            let mut head = Vec::with_capacity(pos.0);
            for e in o.take(pos.0) {
                match e {
                    Node::Elem(Element::Node { .. }) => {
                        head.push(InsertPath::Before);
                    }
                    Node::Expr(Expression::Each(id, _)) | Node::Expr(Expression::IfElse(id, _)) => {
                        head.push(InsertPath::Expr(*id));
                    }
                    _ => (),
                }
            }

            InsertPoint::Append(head)
        } else {
            let mut head = Vec::with_capacity(pos.0);
            let mut tail = Vec::with_capacity(pos.1 - 1 - pos.0);
            for (i, e) in o.enumerate() {
                match e {
                    Node::Elem(Element::Node { .. }) => match i.cmp(&pos.0) {
                        Ordering::Greater => tail.push(InsertPath::Before),
                        Ordering::Less => head.push(InsertPath::Before),
                        Ordering::Equal => (),
                    },
                    Node::Expr(Expression::Each(id, _)) | Node::Expr(Expression::IfElse(id, _)) => {
                        match i.cmp(&pos.0) {
                            Ordering::Greater => tail.push(InsertPath::Expr(*id)),
                            Ordering::Less => head.push(InsertPath::Expr(*id)),
                            Ordering::Equal => (),
                        }
                    }
                    _ => (),
                }
            }

            InsertPoint::LastBefore(head, tail)
        }
    }

    fn get_parent_node(&self) -> usize {
        self.stack
            .last()
            .unwrap()
            .steps
            .iter()
            .rposition(|x| match x {
                Step::FirstChild => true,
                _ => false,
            })
            .unwrap_or_default()
    }

    fn get_black_box_fields(
        &self,
        dom: &Ident,
        on_build: bool,
    ) -> Punctuated<FieldValue, Token![,]> {
        let t_root = format_ident!("t_root");
        let root = Self::get_field_root_ident();
        self.stack.last().unwrap().black_box.iter().fold(
            <Punctuated<FieldValue, Token![,]>>::new(),
            |mut acc, x| {
                if x.name == t_root {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(yarte::YNumber::zero())).unwrap(),
                    });
                } else if x.name == root {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(#dom)).unwrap(),
                    });
                } else if on_build && x.name.to_string().starts_with("__closure__") {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(None)).unwrap(),
                    });
                } else {
                    let name = &x.name;
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(#name)).unwrap(),
                    });
                }

                acc
            },
        )
    }

    fn get_state_fields(&self) -> TokenStream {
        self.s
            .fields
            .iter()
            .filter(|x| is_state(x))
            .fold(<Punctuated<&Ident, Token![,]>>::new(), |mut acc, x| {
                acc.push(&x.ident.as_ref().expect("Named fields"));
                acc
            })
            .into_token_stream()
    }

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

    fn get_current_black_box(&self) -> TokenStream {
        match &self.stack.last().unwrap().id {
            Parent::Expr(id) => {
                let ident = Self::get_vdom_ident(*id);
                quote!(#ident)
            }
            _ => {
                let ident = self.get_global_bbox_ident();
                quote!(self.#ident)
            }
        }
    }

    #[inline]
    fn get_table_dom_ident(id: ExprId) -> Ident {
        const TABLE_DOM: &str = "__ytable_dom__";
        format_ident!("{}{}", TABLE_DOM, id)
    }

    #[inline]
    fn get_table_ident(id: ExprId) -> Ident {
        const TABLE: &str = "__ytable__";
        format_ident!("{}{}", TABLE, id)
    }

    #[inline]
    fn get_vdom_ident(id: ExprId) -> Ident {
        const ELEM: &str = "__dom__";
        format_ident!("{}{}", ELEM, id)
    }

    #[inline]
    // TODO: multiple roots
    fn get_field_root_ident() -> Ident {
        const ROOT: &str = "__root";
        format_ident!("{}", ROOT)
    }

    #[inline]
    fn get_component_ty_ident(id: ExprId) -> Ident {
        const TY: &str = "YComponent";
        format_ident!("{}{}", TY, id)
    }

    #[inline]
    fn get_node_ident(id: ExprId) -> Ident {
        const NODE: &str = "__ynode__";
        format_ident!("{}{}", NODE, id)
    }

    #[inline]
    fn get_body_ident() -> Ident {
        format_ident!("__ybody")
    }

    fn get_global_bbox_ident(&self) -> Ident {
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

    fn get_number_u8(bits: Vec<bool>) -> u8 {
        let mut n = 0;
        for (i, b) in bits.into_iter().enumerate() {
            if b {
                n += 1 << i as u8
            }
        }
        n
    }

    fn get_number_u16(bits: Vec<bool>) -> u16 {
        let mut n = 0;
        for (i, b) in bits.into_iter().enumerate() {
            if b {
                n += 1 << i as u16
            }
        }
        n
    }

    fn get_number_u32(bits: &[bool]) -> u32 {
        let mut n = 0;
        for (i, b) in bits.iter().enumerate() {
            if *b {
                n += 1 << i as u32
            }
        }
        n
    }

    fn get_split_32(mut bits: &[bool]) -> Punctuated<syn::Expr, Token![,]> {
        let mut buff = Punctuated::new();
        while !bits.is_empty() {
            let (current, next) = bits.split_at(32);
            bits = next;
            let current = Self::get_number_u32(current);
            buff.push(parse2(quote!(#current)).unwrap());
        }

        buff
    }

    #[inline]
    fn get_checks(&self, check: HashMap<u64, (Vec<usize>, usize)>) -> TokenStream {
        let mut buff: Vec<TokenStream> = check
            .into_iter()
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<Option<usize>, (Vec<usize>, usize)>,
                 (base, (positions, len))| {
                    if let Some((b, x, id)) = self.stack.iter().rev().find_map(|x| match &x.id {
                        Parent::Head | Parent::Body => None,
                        Parent::Expr(id) => x
                            .bases
                            .iter()
                            .find(|x| match x {
                                Base::Skip(id) | Base::Add(id) => *id == base,
                            })
                            .map(|base| (base, x, id)),
                    }) {
                        if let Base::Add(_) = b {
                            acc.entry(Some(*id))
                                .and_modify(|x| {
                                    let len = x.1;
                                    x.0.extend(positions.iter().copied().map(|i| len + i));
                                    x.1 += len;
                                })
                                .or_insert((positions, len));
                        }
                    } else {
                        // Self
                        acc.entry(None)
                            .and_modify(|x| panic!("expected one self group"))
                            .or_insert((positions, len));
                    }
                    acc
                },
            )
            .into_iter()
            .map(|(i, (x, len))| {
                let (t_root, len) = Self::get_t_root_type(len);
                let mut bits = vec![false; len];
                for i in x {
                    bits[i] = true;
                }
                let number = match len {
                    8 => {
                        let number = Self::get_number_u8(bits);
                        quote!(#number)
                    }
                    16 => {
                        let number = Self::get_number_u16(bits);
                        quote!(#number)
                    }
                    32 => {
                        let number = Self::get_number_u32(&bits);
                        quote!(#number)
                    }
                    64 => {
                        let tokens = Self::get_split_32(&bits);
                        quote!(yarte::U64([#tokens]))
                    }
                    128 => {
                        let tokens = Self::get_split_32(&bits);
                        quote!(yarte::U128([#tokens]))
                    }
                    256 => {
                        let tokens = Self::get_split_32(&bits);
                        quote!(yarte::U256([#tokens]))
                    }
                    _ => todo!("more than 256 variables per context"),
                };

                let vdom = if let Some(i) = i {
                    let ident = Self::get_vdom_ident(i);
                    quote!(#ident)
                } else {
                    let bb = self.get_global_bbox_ident();
                    quote!(self.#bb)
                };

                quote!(yarte::YNumber::neq_zero(#vdom.t_root & #number))
            })
            .collect();
        let mut buff = buff.drain(..);
        let tokens = buff.next().unwrap_or(quote!());
        buff.fold(tokens, |mut acc, t| {
            acc.extend(quote!(|| #t));
            acc
        })
    }

    fn get_check_hash(
        &self,
        checks: HashMap<VarId, Vec<VarId>>,
    ) -> HashMap<u64, (Vec<usize>, usize)> {
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

    fn get_render(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (i, t) in self.get_render_hash() {
            let mut checks: HashMap<VarId, Vec<VarId>> = HashMap::new();
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

    fn get_render_hash(&self) -> HashMap<Vec<VarId>, TokenStream> {
        self.stack.last().unwrap().buff_render.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Vec<u64>, TokenStream>, (i, x)| {
                // TODO: priority when collapsed
                acc.entry(i.iter().copied().collect())
                    .and_modify(|old| {
                        old.extend(x.clone());
                    })
                    .or_insert_with(|| x.clone());
                acc
            },
        )
    }

    fn get_black_box(&self, name: &Ident) -> TokenStream {
        let fields = self
            .stack
            .last()
            .unwrap()
            .black_box
            .iter()
            .cloned()
            .map(Into::into)
            .fold(Punctuated::<Field, Token![,]>::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        quote! {
            #[doc = "Internal elements and difference tree"]
            pub struct #name {
                #fields
            }
        }
    }

    // TODO: Fix me!!
    fn get_steps<'b, T: Iterator<Item = &'b PathNode>>(
        mut nodes: T,
        parent: TokenStream,
    ) -> TokenStream {
        let mut buff = vec![];
        let mut stack = vec![];
        if let Some((ident, path)) = nodes.next() {
            buff.push((parent.clone(), ident.clone(), path.clone()));
            stack.push((ident, path))
        }
        for (ident, path) in nodes {
            let mut check = true;
            for (i, last) in stack.iter().rev() {
                if path.starts_with(last) {
                    buff.push((quote!(#i), ident.clone(), path[last.len()..].to_vec()));
                    check = false;
                    break;
                }
            }
            if check {
                buff.push((parent.clone(), ident.clone(), path.clone()));
            }
            stack.push((ident, path))
        }

        let mut tokens = TokenStream::new();
        for (p, i, path) in buff.drain(..) {
            // TODO: remove clone
            if path.is_empty() {
                tokens.extend(quote! {
                    let #i = #p.clone();
                })
            } else {
                tokens.extend(quote! {
                    let #i = #p#(#path)*;
                })
            }
        }

        tokens
    }

    fn get_drop(component: &Ident, roots: &[Ident]) -> TokenStream {
        let mut tokens = TokenStream::new();
        for root in roots {
            tokens.extend(quote!(self.#root.remove();));
        }
        quote! {
            impl Drop for #component {
                fn drop(&mut self) {
                    #tokens
                }
            }
        }
    }

    // Inits
    fn init_build(&mut self) {
        let ident = format_ident!("{}InitialState", self.s.ident);
        let args = self.get_state_fields();
        self.build.extend(quote! {
            let #ident { #args } = yarte::from_str(&get_state()).unwrap_or_default();
            let doc = yarte::web::window().unwrap_throw().document().unwrap_throw();
        });
    }

    fn init_hydrate(&mut self) {
        let body = Self::get_body_ident();
        self.hydrate.extend(quote! {
            let #body = yarte::web::window().unwrap_throw()
                .document().unwrap_throw()
                .body().unwrap_throw();
        });
    }

    fn init_render(&mut self) {
        let name = self.get_global_bbox_ident();
        let (ty, _) = self.get_black_box_t_root(iter::once(self.self_id));
        self.render.extend(quote! {
            if self.#name.t_root == <#ty as yarte::YNumber>::zero() {
                return;
            }
        });
        let (base, _) = self.get_black_box_t_root(iter::once(self.self_id));
        self.stack.last_mut().unwrap().black_box.push(BlackBox {
            doc: "Difference tree".to_string(),
            name: format_ident!("t_root"),
            ty: parse2(base).unwrap(),
        });
    }

    #[inline]
    fn init(&mut self, dom: DOM) {
        self.resolve_tree_var(dom.tree_map, dom.var_map);
        self.init_build();
        self.init_hydrate();
        self.init_render();

        assert_eq!(dom.doc.len(), 1);
        match &dom.doc[0] {
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

                let (head, body) = children.iter().fold((None, None), |acc, x| match x {
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
                    if !self.stack.last().unwrap().path_nodes.is_empty() {
                        let ident = format_ident!("__yhead");
                        let current = self.stack.last_mut().unwrap();
                        self.build
                            .extend(quote!(let #ident = doc.head().unwrap_throw();));
                        // TODO: path events
                        let tokens = Self::get_steps(current.path_nodes.iter(), quote!(#ident));
                        self.build.extend(tokens);
                        self.build
                            .extend(mem::take(&mut current.buff_build).into_iter().flatten());
                        current.path_nodes.clear();
                    }
                }
                if let Some(body) = body {
                    self.stack.last_mut().unwrap().id = Parent::Body;
                    self.step(body);
                    if !self.stack.last().unwrap().path_nodes.is_empty() {
                        let ident = Self::get_body_ident();
                        let current = self.stack.last_mut().unwrap();
                        self.build
                            .extend(quote!(let #ident = doc.body().unwrap_throw();));
                        // TODO: path events
                        let tokens = Self::get_steps(current.path_nodes.iter(), quote!(#ident));
                        self.build.extend(tokens);
                        self.build
                            .extend(mem::take(&mut current.buff_build).into_iter().flatten());
                        current.path_nodes.clear();
                    }
                } else {
                    panic!("Need <body> tag")
                }
            }
            _ => panic!("Need html at root"),
        }
        let tokens = self.get_render();
        self.render.extend(tokens);
    }

    // Main recursive loop
    fn step(&mut self, doc: &Document) {
        let last_node = self.stack.last_mut().unwrap().steps.len();
        let len = doc.iter().fold(0, |acc, x| match x {
            Node::Elem(Element::Text(_)) => acc,
            _ => acc + 1,
        });
        let mut last = 0usize;
        let nodes = doc.iter().map(|x| match x {
            Node::Elem(Element::Text(_)) => (last, x),
            _ => {
                let l = last;
                last += 1;
                (l, x)
            }
        });
        let children = doc.iter().filter(|x| match x {
            Node::Elem(Element::Text(_)) => false,
            _ => true,
        });
        let mut last = None;
        for (i, node) in nodes {
            match node {
                Node::Elem(Element::Node { .. }) if last.is_none() => {
                    last = Some(Step::FirstChild);
                }
                Node::Elem(Element::Node { .. }) => {
                    last = Some(Step::NextSibling);
                }
                _ => (),
            }

            self.resolve_node(node, last, (i, len), children.clone());
        }

        if last.is_some() {
            self.stack.last_mut().unwrap().steps.drain(last_node..);
        }
    }

    // Resolve
    #[inline]
    fn resolve_node<'b, F: Iterator<Item = &'b Node> + Clone>(
        &mut self,
        node: &'b Node,
        step: Option<Step>,
        pos: (usize, usize),
        o: F,
    ) {
        match node {
            Node::Elem(Element::Node {
                children, attrs, ..
            }) => {
                let old = self.on_node.take();
                self.stack
                    .last_mut()
                    .unwrap()
                    .steps
                    .push(step.expect("Some step"));
                for attr in attrs {
                    self.resolve_attr(attr);
                }

                if all_children_text(children) {
                    self.write_leaf_text(children);
                } else {
                    self.step(children);
                }
                self.on_node = old;
            }
            Node::Expr(e) => match e {
                Expression::Each(id, each) => {
                    let insert_point = Self::get_insert_point(pos, o);
                    self.gen_each(*id, each, pos.1 != 1, insert_point)
                }
                Expression::IfElse(id, if_else) => {
                    let IfElse { ifs, if_else, els } = &**if_else;

                    self.resolve_if_block(ifs, *id);
                    for b in if_else {
                        self.resolve_if_block(b, *id);
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

    #[inline]
    fn resolve_attr(&mut self, attr: &Attribute) {
        if let Some(event) = is_on_attr(attr) {
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
    fn resolve_if_block(&mut self, IfBlock { block, .. }: &IfBlock, id: ExprId) {
        todo!("resolve if else block expresion");
    }

    #[inline]
    fn resolve_tree_var(&mut self, tree_map: TreeMap, var_map: VarMap) {
        let mut grouped = HashMap::new();
        let var_map: HashMap<VarId, VarInner> = var_map
            .into_iter()
            .filter(|(_, x)| match x {
                Var::This(..) => true,
                Var::Local(..) => false,
            })
            .map(|(i, x)| match x {
                Var::This(x) => (i, x),
                Var::Local(..) => unreachable!(),
            })
            .inspect(|(var_id, var)| {
                grouped
                    .entry(var.base)
                    .and_modify(|x: &mut BTreeSet<VarId>| {
                        x.insert(*var_id);
                    })
                    .or_insert_with(|| {
                        // Need Order
                        let mut b = BTreeSet::new();
                        b.insert(*var_id);
                        b
                    });
            })
            .collect();

        if grouped.get(&self.self_id).is_none() {
            panic!("need any field in struct of application")
        }
        self.grouped_map = grouped;
        self.tree_map = tree_map;
        self.var_map = var_map;
    }

    // Clear buffer and return it
    fn empty_components(&mut self) -> TokenStream {
        self.component
            .drain(..)
            .fold(
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
            .into_token_stream()
    }

    // Writes current state
    #[inline]
    fn write_leaf_text(&mut self, children: &Document) {
        let (t, e) = get_leaf_text(children, &self.tree_map, &self.var_map);
        let name = self.current_node_ident();

        let dom = match &self.stack.last().unwrap().id {
            Parent::Body => {
                let ident = self.get_global_bbox_ident();
                quote!(self.#ident)
            }
            Parent::Expr(i) => {
                let ident = Self::get_vdom_ident(*i);
                quote!(#ident)
            }
            Parent::Head => todo!(),
        };
        {
            let current = self.stack.last_mut().unwrap();
            current
                .buff_new
                .push(quote! { #name.set_text_content(Some(&#e)); });

            current
                .path_nodes
                .push((name.clone(), current.steps.clone()));
        }
        let current = self.stack.last_mut().unwrap();
        current
            .buff_render
            .push((t, quote! { #dom.#name.set_text_content(Some(&#e)); }));
        current.black_box.push(BlackBox {
            doc: format!("Yarte Node element\n\n```\n{}\n```", e),
            name,
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });
    }

    // Registers
    fn current_node_ident(&mut self) -> Ident {
        Self::get_node_ident(self.on_node.unwrap_or_else(|| {
            let id = self.count;
            self.count += 1;
            self.on_node = Some(id);
            id
        }))
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        self.init(ir.into());

        let initial_state = self.get_initial_state();
        let black_box_name = format_ident!("{}BlackBox", self.s.ident);
        let bb_fields = self.get_black_box_fields(&Self::get_field_root_ident(), true);
        let ty_component: Type = parse2(quote!(yarte::web::Element)).unwrap();
        for (i, _) in &self.component {
            self.stack.last_mut().unwrap().black_box.push(BlackBox {
                doc: "Component".to_string(),
                name: i.clone(),
                ty: ty_component.clone(),
            })
        }
        let components = self.empty_components();
        let black_box = self.get_black_box(&black_box_name);
        let args = self.get_state_fields();
        let bb_ident = self.get_global_bbox_ident();
        let inner = self.get_inner();
        let build = &self.build;
        let mut bb = vec![];
        if !bb_fields.is_empty() {
            bb.push(bb_fields.to_token_stream());
        }
        if !components.is_empty() {
            bb.push(components);
        }
        let mut fields = vec![];
        if !args.is_empty() {
            fields.push(args);
        }
        if !inner.is_empty() {
            fields.push(inner.to_token_stream())
        }
        fields.push(quote! {
            #bb_ident: #black_box_name { #(#bb),* }
        });

        let build = quote! {
            #build
            Self { #(#fields),* }
        };
        let default = self.s.implement_head(
            quote!(std::default::Default),
            &quote!(fn default() -> Self { #build }),
        );
        let render = &self.render;
        let render = quote! {
            #render
            self.#bb_ident.t_root = yarte::YNumber::zero();
        };
        // TODO: Remove Output buffer
        let current = self.stack.last().unwrap();
        let body = Self::get_body_ident();
        let steps = Self::get_steps(current.path_events.iter(), quote!(#body));
        let hydrate = &current.buff_hydrate;
        self.hydrate.extend(quote! {
            #steps
            #(#hydrate)*
        });
        let hydrate = &self.hydrate;
        let msgs = self
            .s
            .msgs
            .as_ref()
            .expect("Need define messages for application");
        let (dispatch, enu) = messages::gen_messages(msgs);
        let type_msgs = &msgs.ident;
        let app = quote! {
            type BlackBox = #black_box_name;
            type Message = #type_msgs;

            #[doc(hidden)]
            #[inline]
            fn __render(&mut self, __addr: &yarte::Addr<Self>) { # render }

            #[doc(hidden)]
            #[inline]
            fn __hydrate(&mut self, __addr: &yarte::Addr<Self>) { # hydrate }

            #[doc(hidden)]
            fn __dispatch(&mut self, __msg: Self::Message, __addr: &yarte::Addr<Self>) { #dispatch }
        };
        let app = self.s.implement_head(quote!(yarte::Template), &app);
        let helpers = &self.helpers;

        // Multi app compilation
        clean();

        quote! {
            #[wasm_bindgen]
            extern "C" {
                fn get_state() -> String;
            }

            #app
            #enu
            #initial_state
            #black_box
            #helpers
            #default
        }
    }
}

fn is_on_attr(attr: &Attribute) -> Option<&str> {
    match &attr.name {
        ExprOrText::Text(s) if s.starts_with("on") => Some(s),
        _ => None,
    }
}

fn all_children_text(doc: &Document) -> bool {
    !doc.iter().all(|x| {
        if let Node::Elem(Element::Text(_)) = x {
            true
        } else {
            false
        }
    }) && doc.iter().all(|x| match x {
        Node::Elem(Element::Text(_)) => true,
        Node::Expr(e) => match e {
            Expression::IfElse(_, block) => {
                let IfElse { ifs, if_else, els } = &**block;
                all_if_block_text(ifs)
                    && if_else.iter().all(|x| all_if_block_text(x))
                    && els.as_ref().map(|x| all_children_text(x)).unwrap_or(true)
            }
            Expression::Each(_, block) => {
                let Each { body, .. } = &**block;
                all_children_text(body)
            }
            Expression::Local(..) => false,
            _ => true,
        },
        _ => false,
    })
}

#[inline]
fn all_if_block_text(IfBlock { block, .. }: &IfBlock) -> bool {
    all_children_text(block)
}

fn check_attr_is_text(attr: Attribute) -> bool {
    attr.value.len() == 1
        && match attr.value[0] {
            ExprOrText::Text(..) => true,
            ExprOrText::Expr(..) => false,
        }
}
