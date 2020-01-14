use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use syn::{
    parse2, parse_str, punctuated::Punctuated, token::Comma, Expr, Field, Ident, Token, Type,
    VisPublic, Visibility,
};

use yarte_config::Config;
use yarte_dom::dom::{
    Attribute, Document, Element, ExprId, ExprOrText, Expression, Node, Ns, Var, VarId, DOM,
};
use yarte_dom::ElemInfo;
use yarte_hir::{Struct, HIR};

use crate::CodeGen;
use test::NamePadding::PadNone;

mod each;
mod if_else;

enum Path {
    FirstChild,
    NextSibling,
    LastChild,
    PreviousSibling,
}

pub struct WASMCodeGen<'a> {
    s: &'a Struct<'a>,
    config: &'a Config<'a>,
    build: TokenStream,
    render: TokenStream,
    hydrate: TokenStream,
    helpers: TokenStream,
    black_box: Vec<(Ident, Type)>,
    stack: Vec<ElemInfo>,
    path: Vec<Path>,
    bit_array: HashSet<VarId>,
    tree_map: HashMap<ExprId, Vec<VarId>>,
    var_map: HashMap<VarId, Var>,
}

fn is_state(f: &Field) -> bool {
    todo!()
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen {
            config,
            build: TokenStream::new(),
            render: TokenStream::new(),
            hydrate: TokenStream::new(),
            helpers: TokenStream::new(),
            s,
            black_box: vec![],
            path: vec![],
            bit_array: HashSet::new(),
            tree_map: HashMap::new(),
            var_map: HashMap::new(),
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        if self.stack.is_empty() {
            panic!("no parent ElemInfo")
        }
        self.stack.last_mut().unwrap()
    }

    fn initial_state(&self) -> TokenStream {
        let attr: syn::Attribute = parse2(quote!(#[serde(default)])).unwrap();
        let fields = self
            .s
            .fields
            .iter()
            .filter(|x| is_state(x))
            .map(|x| {
                let mut f = x.clone();
                f.attrs.push(attr.clone());
                f
            })
            .fold(Punctuated < Field, Comma > ::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        quote! {
            #[derive(Default, Deserialize)]
            struct InitialState {
                #fields
            }
        }
    }

    fn black_box(&self) -> TokenStream {
        let fields = self
            .black_box
            .iter()
            .map(|(ident, ty)| Field {
                attrs: vec![],
                vis: Visibility::Inherited,
                ident: ident.clone(),
                colon_token: Some(<Token![;]>::default()),
                ty: ty.clone(),
            })
            .fold(Punctuated < Field, Comma > ::new(), |mut acc, x| {
                acc.push(x);
                acc
            });

        quote! {
            struct BlackBox {
                #fields
            }
        }
    }

    fn init(&mut self, mut dom: DOM) {
        self.resolve_tree_var(dom.tree_map, dom.var_map);

        assert_eq!(dom.doc.len(), 1);
        match dom.doc.remove(0) {
            Node::Elem(Element::Node {
                name,
                attrs,
                children,
            }) => {
                match name.0 {
                    Ns::Html => (),
                    _ => panic!("Need <html> tag"),
                }
                match name.1 {
                    local_name!("html") => (),
                    _ => panic!("Need <html> tag"),
                }
                for attr in attrs {
                    if !check_attr_is_text(attr) {
                        panic!("Only static attributes in <html>")
                    }
                }
                self.read_doc(children);
            }
            _ => panic!("Need <html> tag"),
        }
    }

    fn read_doc(&mut self, doc: Document) {
        let fragment = 1 < doc.len();
        let bound = 0;
        for (i, node) in doc.into_iter().enumerate() {
            match node {
                Node::Elem(elem) => match elem {
                    Element::Node {
                        name,
                        attrs,
                        children,
                    } => {}
                    Element::Text(s) => {}
                },
                Node::Expr(expr) => match expr {
                    Expression::Local(_, _, _) => {}
                    Expression::Unsafe(_, _) => {}
                    Expression::Safe(_, _) => {}
                    Expression::Each(_, _) => {}
                    Expression::IfElse(_, _) => {}
                },
            }
        }
    }

    fn resolve_tree_var(
        &mut self,
        tree_map: HashMap<ExprId, Vec<VarId>>,
        var_map: HashMap<VarId, Var>,
    ) {
        for expr in &tree_map.values() {
            for var_id in expr {
                match var_map.get(&var_id).expect("variable in map") {
                    Var::This(var) if var.starts_with("self.") => {
                        self.bit_array.insert(*var_id);
                    }
                    _ => (),
                }
            }
        }

        self.tree_map = tree_map;
        self.var_map = var_map;
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
        self.read_doc(ir.into());

        let initial_state = self.initial_state();
        let black_box = self.black_box();
        let default = self
            .s
            .implement_head(quote!(std::default::Default), self.build);
        let render = self.render;
        let hydrate = self.hydrate;
        let app = self.s.implement_head(
            quote!(yarte::Template),
            quote! {
            # [doc(hidden)]
            fn __render(& mut self, __addr: & Addr < Self> ) { # render }
            # [doc(hidden)]
            fn __hydrate(& mut self, __addr: & Addr < Self> ) { # hydrate }
            },
        );
        let helpers = self.helpers;

        quote! {
            # [wasm_bindgen]
            extern "C" {
            fn get_state() -> String;
            }

            # initial_state
            # black_box
            # default
            # app
            # helpers
        }
    }
}

fn check_attr_is_text(attr: Attribute) -> bool {
    attr.value.len() == 1
        && match attr.value[0] {
            ExprOrText::Text(..) => true,
            ExprOrText::Expr(..) => false,
        }
}
