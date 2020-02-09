use std::collections::{BTreeSet, HashMap};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Expr, Token};

use yarte_dom::dom::{Document, Element, Expression, Node, TreeMap, VarId, VarInner};

pub fn get_leaf_text(
    children: &Document,
    tree_map: &TreeMap,
    var_map: &HashMap<VarId, VarInner>,
) -> (BTreeSet<VarId>, TokenStream) {
    LeafTextBuilder::new(tree_map, var_map).build(children)
}

struct LeafTextBuilder<'a> {
    tree_map: &'a TreeMap,
    var_map: &'a HashMap<VarId, VarInner>,
    buff: BTreeSet<VarId>,
    buff_expr: String,
    buff_args: Punctuated<Expr, Token![,]>,
}

// TODO: #[str] alone expression for no reallocate string
impl<'a> LeafTextBuilder<'a> {
    fn new<'n>(
        tree_map: &'n TreeMap,
        var_map: &'n HashMap<VarId, VarInner>,
    ) -> LeafTextBuilder<'n> {
        LeafTextBuilder {
            tree_map,
            var_map,
            buff: Default::default(),
            buff_expr: "".into(),
            buff_args: Default::default(),
        }
    }

    fn build(mut self, children: &Document) -> (BTreeSet<VarId>, TokenStream) {
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
                        self.buff_expr.push_str("{}");
                        self.buff_args.push(*e.clone());
                    }
                    Expression::Each(_id, _e) => todo!(),
                    Expression::IfElse(_id, _e) => todo!(),
                    Expression::Local(..) => todo!(),
                },
                _ => unreachable!(),
            }
        }
    }
}
