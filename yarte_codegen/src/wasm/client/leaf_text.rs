#![allow(warnings)]
use proc_macro2::TokenStream;

use yarte_dom::dom::{Document, VarId};

pub fn get_leaf_text(children: &Document) -> (Vec<VarId>, TokenStream) {
    LeadTextBuilder::default().build(children)
}

#[derive(Default)]
struct LeadTextBuilder {}

impl LeadTextBuilder {
    fn build(mut self, children: &Document) -> (Vec<VarId>, TokenStream) {
        let tokens = TokenStream::new();

        (vec![], tokens)
    }
}
