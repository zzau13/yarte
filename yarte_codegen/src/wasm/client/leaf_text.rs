#![allow(warnings)]

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Expr, Token};

use yarte_dom::dom::{Document, Element, ExprId, Expression, Node, TreeMap, Var, VarId, VarMap};

pub fn get_leaf_text(
    children: &Document,
    tree_map: &TreeMap,
    var_map: &VarMap,
) -> (HashSet<VarId>, TokenStream) {
    LeafTextBuilder::new(tree_map, var_map).build(children)
}

struct LeafTextBuilder<'a> {
    tree_map: &'a TreeMap,
    var_map: &'a VarMap,
    buff: HashSet<VarId>,
    buff_expr: String,
    buff_args: Punctuated<Expr, Token![,]>,
}

impl<'a> LeafTextBuilder<'a> {
    fn new<'n>(tree_map: &'n TreeMap, var_map: &'n VarMap) -> LeafTextBuilder<'n> {
        LeafTextBuilder {
            tree_map,
            var_map,
            buff: Default::default(),
            buff_expr: "".into(),
            buff_args: Default::default(),
        }
    }

    fn build(mut self, children: &Document) -> (HashSet<VarId>, TokenStream) {
        self.init(children);

        let args = self.buff_args;
        let expr = self.buff_expr;
        (self.buff, quote!(format!(#expr, #args)))
    }

    fn init(&mut self, children: &Document) {
        for child in children {
            match child {
                Node::Elem(Element::Text(t)) => self
                    .buff_expr
                    .push_str(&t.replace("{", "{{").replace("}", "}}")),
                Node::Expr(e) => match e {
                    Expression::Safe(id, e) | Expression::Unsafe(id, e) => {
                        let vars = self.tree_map.get(id).expect("Expression to be defined");
                        self.buff.extend(vars);
                        self.buff_expr.push_str("{{}}");
                        self.buff_args.push(*e.clone());
                    }
                    Expression::Each(id, e) => todo!(),
                    Expression::IfElse(id, e) => todo!(),
                    Expression::Local(..) => todo!(),
                },
                _ => unreachable!(),
            }
        }
    }
}
