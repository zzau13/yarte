#![allow(warnings)]

use std::{collections::HashMap, mem};

use markup5ever::local_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer},
    parse2, parse_str,
    punctuated::Punctuated,
    Field, FieldValue, Ident, Member, Meta, MetaList, MetaNameValue, NestedMeta, Path, Token, Type,
    VisPublic, Visibility,
};

use yarte_dom::dom::{
    Attribute, Document, Each, Element, ExprId, ExprOrText, Expression, IfBlock, IfElse, Node,
    TreeMap, Var, VarId, VarMap, DOM,
};
use yarte_hir::{Struct, HIR};

use crate::CodeGen;

mod each;
mod if_else;
mod leaf_text;
mod messages;

use self::leaf_text::get_leaf_text;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Step {
    FirstChild,
    NextSibling,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parent {
    Head,
    Body,
    Expr(ExprId),
}

#[derive(Clone, Debug)]
pub enum InsertPoint {
    Append,
    LastBefore(Vec<InsertPath>),
}

#[derive(Clone, Debug)]
pub enum InsertPath {
    Before,
    Expr(ExprId),
}

pub struct WASMCodeGen<'a> {
    s: &'a Struct<'a>,
    build: TokenStream,
    render: TokenStream,
    hydrate: TokenStream,
    helpers: TokenStream,
    steps: Vec<(Option<Ident>, Step)>,
    on: Option<Parent>,
    //
    buff_render: Vec<(HashSet<VarId>, TokenStream)>,
    black_box: Vec<BlackBox>,
    bit_array: Vec<VarId>,
    tree_map: TreeMap,
    var_map: VarMap,
    count: usize,
}

#[derive(Debug)]
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

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen {
            build: TokenStream::new(),
            render: TokenStream::new(),
            hydrate: TokenStream::new(),
            helpers: TokenStream::new(),
            steps: vec![],
            s,
            black_box: vec![],
            bit_array: Vec::new(),
            tree_map: HashMap::new(),
            var_map: HashMap::new(),
            buff_render: vec![],
            on: None,
            count: 0,
        }
    }

    fn initial_state(&self) -> TokenStream {
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

    fn black_box(&mut self, name: &Ident) -> TokenStream {
        let fields = self.black_box.drain(..).map(Into::into).fold(
            Punctuated::<Field, Token![,]>::new(),
            |mut acc, x| {
                acc.push(x);
                acc
            },
        );

        quote! {
            #[doc = "Internal elements and difference tree"]
            pub struct #name {
                #fields
            }
        }
    }

    fn add_black_box_t_root(&mut self) {
        let len = self.bit_array.len();
        let base = match len {
            0..=8 => 8,
            9..=16 => 16,
            17..=32 => 32,
            _ => todo!(),
        };
        self.black_box.push(BlackBox {
            doc: "Difference tree".to_string(),
            name: format_ident!("t_root"),
            ty: parse_str(&format!("u{}", base)).unwrap(),
        });
    }

    fn get_black_box_fields(&mut self) -> Punctuated<FieldValue, Token![,]> {
        let t_root = format_ident!("t_root");
        self.black_box
            .iter()
            .fold(<Punctuated<FieldValue, Token![,]>>::new(), |mut acc, x| {
                if x.name == t_root {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(0)).unwrap(),
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
            })
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
                    member: x
                        .ident
                        .clone()
                        .map(|x| Member::Named(x))
                        .expect("Named fields"),
                    colon_token: Some(<Token![:]>::default()),
                    expr: parse2(expr).expect("valid expression"),
                });
                acc
            },
        )
    }

    fn build_init(&mut self) {
        let ident = format_ident!("{}InitialState", self.s.ident);
        let args = self.get_state_fields();
        self.build.extend(quote! {
            let #ident { #args } = yarte::from_str(&get_state()).unwrap_or_default();
            let doc = yarte::web::window().unwrap_throw().document().unwrap_throw();
        });
    }

    fn hydrate_init(&mut self) {
        self.hydrate.extend(quote! {
            let body = yarte::web::window().unwrap_throw()
                .document().unwrap_throw()
                .body().unwrap_throw();
        });
    }

    fn render_init(&mut self) {
        let name = self.get_black_box_ident();
        self.render.extend(quote! {
            if self.#name.t_root == 0 {
                return;
            }
        });
        self.add_black_box_t_root()
    }

    fn get_black_box_ident(&self) -> Ident {
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

    fn init(&mut self, dom: DOM) {
        self.resolve_tree_var(dom.tree_map, dom.var_map);
        self.build_init();
        self.hydrate_init();
        self.render_init();

        assert_eq!(dom.doc.len(), 1);
        match &dom.doc[0] {
            Node::Elem(Element::Node { name, children, .. }) => {
                assert_eq!(local_name!("html"), name.1);
                assert!(children.iter().all(|x| match x {
                    Node::Elem(Element::Node { name, .. }) => match name.1 {
                        local_name!("body") | local_name!("head") => true,
                        _ => false,
                    },
                    Node::Elem(Element::Text(text)) => text.chars().all(|x| x.is_whitespace()),
                    _ => false,
                }));

                let (head, body) = children.into_iter().fold((None, None), |acc, x| match x {
                    Node::Elem(Element::Node { name, children, .. }) => match name.1 {
                        local_name!("body") => (acc.0, Some(children)),
                        local_name!("head") => (Some(children), acc.1),
                        _ => acc,
                    },
                    _ => acc,
                });
                if let Some(head) = head {
                    self.on = Some(Parent::Head);
                    self.step(head);
                    self.on.take().unwrap();
                }
                if let Some(body) = body {
                    self.on = Some(Parent::Body);
                    self.step(body);
                    self.on.take().unwrap();
                } else {
                    panic!("Need <body> tag")
                }
            }
            _ => panic!("Need html at root"),
        }
        let tokens = self.empty_buff();
        self.render.extend(tokens);
    }

    fn step(&mut self, doc: &Document) {
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
            let last = self.parent_node();
            self.steps.drain(last..);
        }
    }

    fn parent_node(&mut self) -> usize {
        self.steps
            .iter()
            .rposition(|(_, x)| match x {
                Step::FirstChild => true,
                _ => false,
            })
            .unwrap_or_default()
    }

    fn do_step(&mut self, body: &Document, id: ExprId) {
        let on = self.on.replace(Parent::Expr(id));
        let steps = mem::take(&mut self.steps);
        self.step(body);
        self.on = on;
        self.steps = steps;
    }

    #[inline]
    fn resolve_node<'b, F: Iterator<Item = &'b Node> + Clone>(
        &mut self,
        node: &'b Node,
        step: Option<Step>,
        pos: (usize, usize),
        o: F,
    ) {
        let mut buff = vec![];
        match node {
            Node::Elem(Element::Node {
                children, attrs, ..
            }) => {
                // TODO
                for attr in attrs {
                    for e in &attr.value {
                        if let ExprOrText::Expr(e) = e {
                            buff.push((e, Some(attr.name.clone())));
                        }
                    }
                }
                if all_children_text(children) {
                    let (t, e) = get_leaf_text(children, &self.tree_map, &self.var_map);
                    let name = format_ident!("ynode_{}", self.count);
                    let black_box_name = self.get_black_box_ident();
                    self.count += 1;
                    let dom = match self.on.expect("Some parent") {
                        Parent::Body => self.get_black_box_ident(),
                        Parent::Expr(i) => format_ident!("dom_{}", i),
                        Parent::Head => panic!(""),
                    };
                    self.buff_render.push((
                        t,
                        quote! {
                            #dom.#name.set_text_content(Some(&#e));
                        },
                    ));
                    self.black_box.push(BlackBox {
                        doc: "Yarte Node element".to_string(),
                        name,
                        ty: parse2(quote!(yarte::web::Element)).unwrap(),
                    });
                } else {
                    self.steps.push((None, step.expect("Some step")));
                    self.step(children);
                }
            }
            Node::Expr(e) => {
                buff.push((e, None));
            }
            Node::Elem(Element::Text(_)) => (),
        }

        for (i, attr) in buff {
            match i {
                Expression::Each(id, each) => {
                    let _insert_point = self.insert_point(pos, o.clone());
                    self.gen_each(*id, each, quote!())
                }
                Expression::Safe(id, _) | Expression::Unsafe(id, _) => {
                    let node = self.on.unwrap();
                    let vars = self.tree_map.get(id).cloned().unwrap_or_default();
                }
                Expression::IfElse(id, if_else) => {
                    todo!();
                    let IfElse { ifs, if_else, els } = &**if_else;

                    self.if_block(ifs, *id);
                    for b in if_else {
                        self.if_block(b, *id);
                    }
                    if let Some(body) = els {
                        self.do_step(body, *id);
                    }
                }
                Expression::Local(..) => todo!(),
            }
        }
    }

    #[inline]
    fn if_block(&mut self, IfBlock { block, .. }: &IfBlock, id: ExprId) {
        self.do_step(block, id);
    }

    fn insert_point<'b, F: Iterator<Item = &'b Node>>(
        &self,
        pos: (usize, usize),
        o: F,
    ) -> InsertPoint {
        if pos.0 + 1 == pos.1 {
            InsertPoint::Append
        } else {
            let mut buff = Vec::with_capacity(pos.1 - 1 - pos.0);
            let o: Vec<&Node> = o.collect();
            for i in o.iter().skip(pos.0 + 1).rev() {
                match i {
                    Node::Elem(Element::Node { .. }) => {
                        buff.push(InsertPath::Before);
                    }
                    Node::Expr(Expression::Each(id, _)) | Node::Expr(Expression::IfElse(id, _)) => {
                        buff.push(InsertPath::Expr(*id));
                    }
                    _ => (),
                }
            }

            InsertPoint::LastBefore(buff)
        }
    }

    #[allow(warnings)]
    fn empty_buff(&mut self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for (_i, t) in self.buff_render.drain(..) {
            tokens.extend(quote!(if true { #t }));
        }

        tokens
    }

    fn resolve_tree_var(&mut self, tree_map: TreeMap, var_map: VarMap) {
        for expr in tree_map.values() {
            for var_id in expr {}
        }

        self.tree_map = tree_map;
        self.var_map = var_map;
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        self.init(ir.into());

        let initial_state = self.initial_state();
        let black_box_name = format_ident!("{}BlackBox", self.s.ident);
        let bb_fields = self.get_black_box_fields();
        let black_box = self.black_box(&black_box_name);
        let args = self.get_state_fields();
        let bb_ident = self.get_black_box_ident();
        let inner = self.get_inner();
        let build = &self.build;
        let build = quote! {
            #build
            Self {
                #args,
                #inner,
                #bb_ident: #black_box_name { #bb_fields }
            }
        };
        let default = self.s.implement_head(
            quote!(std::default::Default),
            &quote!(fn default() -> Self { #build }),
        );
        let render = &self.render;
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

        quote! {
            #[wasm_bindgen]
            extern "C" {
                fn get_state() -> String;
            }

            #initial_state
            #black_box
            #default
            #enu
            #app
            #helpers
        }
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
