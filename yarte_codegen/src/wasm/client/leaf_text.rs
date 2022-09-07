use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Expr, Token};

use yarte_dom::dom::{Document, Element, Expression, Node, VarId};

use crate::wasm::client::solver::Solver;

pub fn get_leaf_text(children: Document, solver: &Solver) -> (BTreeSet<VarId>, TokenStream) {
    LeafTextBuilder::new(solver).build(children)
}

struct LeafTextBuilder<'a> {
    solver: &'a Solver,
    buff: BTreeSet<VarId>,
    buff_expr: String,
    buff_args: Punctuated<Expr, Token![,]>,
}

// TODO: #[str] alone expression for no reallocate string
impl<'a> LeafTextBuilder<'a> {
    fn new(solver: &Solver) -> LeafTextBuilder {
        LeafTextBuilder {
            solver,
            buff: Default::default(),
            buff_expr: Default::default(),
            buff_args: Default::default(),
        }
    }

    fn build(mut self, children: Document) -> (BTreeSet<VarId>, TokenStream) {
        self.init(children);

        let args = self.buff_args;
        let expr = self.buff_expr;
        (self.buff, quote!(format!(#expr, #args)))
    }

    fn init(&mut self, children: Document) {
        for child in children {
            match child {
                Node::Elem(Element::Text(t)) => self
                    .buff_expr
                    .push_str(&t.replace('{', "{{").replace('}', "}}")),
                Node::Expr(e) => match e {
                    // TODO
                    Expression::Safe(id, e) | Expression::Unsafe(id, e) => {
                        let vars = self.solver.expr_inner_var(&id);
                        self.buff.extend(vars);
                        self.buff_expr.push_str("{}");
                        self.buff_args.push(*e);
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
