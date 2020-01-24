use proc_macro2::TokenStream;
use quote::{format_ident};

use yarte_dom::dom::{Document, ExprId, Element, Node, Attribute};

use super::{BlackBox, WASMCodeGen};

impl<'a> WASMCodeGen<'a> {
    pub(super) fn component(&mut self, id: ExprId, doc: &Document) {
        let tokens = TokenStream::new();
        let ident = format_ident!("component_{}", id);

        if doc.len() == 1 {
            match &doc[0] {
                Node::Elem(Element::Node { name, attrs, children }) => {
                    let component = self.component_node(name.1.to_string(), attrs, children);

                }
                _ => (),
            }

        } else {
            ()
        }

        self.buff_component.push((ident, tokens))
    }

    fn component_node(&self, name: String, attrs: &[Attribute], children: &Document) {

    }
}
