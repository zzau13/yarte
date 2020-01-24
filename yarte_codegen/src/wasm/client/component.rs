use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use yarte_dom::dom::{Attribute, DOMBuilder, Document, Element, ExprId, Node, ExprOrText};

use super::{BlackBox, WASMCodeGen};
use std::mem;

pub fn get_component(id: ExprId, doc: &Document, builder: &mut WASMCodeGen) {
    ComponentBuilder::new(id, builder).build(doc)
}

const HEAD: &str = "n__";
struct ComponentBuilder<'a, 'b> {
    builder: &'a mut WASMCodeGen<'b>,
    id: ExprId,
    count: usize,
    tokens: TokenStream,
    children: Vec<Ident>,
}

impl<'a, 'b> ComponentBuilder<'a, 'b> {
    fn new<'n, 'm>(id: ExprId, builder: &'n mut WASMCodeGen<'m>) -> ComponentBuilder<'n, 'm> {
        ComponentBuilder {
            builder,
            id,
            count: 0,
            tokens: TokenStream::new(),
            children: vec![],
        }
    }

    fn build(mut self, doc: &Document) {
        let ident = format_ident!("component_{}", self.id);

        let doc: Vec<&Node> = Self::filter(doc).collect();

        if doc.len() == 1 {
            match &doc[0] {
                Node::Elem(Element::Node {
                    name,
                    attrs,
                    children,
                }) => {
                    let id = format_ident!("{}{}", HEAD, self.count);
                    let tag = name.1.to_string();
                    self.count += 1;
                    self.tokens.extend(quote! {
                        let #id = doc.create_element(#tag).unwrap_throw();
                    });
                    self.step(children, &id);
                    self.set_attrs(&id, attrs);
                    self.empty_buff(&id);
                    self.tokens.extend(quote!(#id))
                }
                _ => todo!("no node element"),
            }
        } else {
            todo!("len +1")
        }

        self.builder.buff_component.push((ident, self.tokens))
    }

    fn filter(doc: &Document) -> impl Iterator<Item = &Node> {
        doc.into_iter().filter(|x| match x {
            Node::Elem(Element::Text(t)) => {
                if t.chars().all(|x| x.is_whitespace()) {
                    false
                } else {
                    true
                }
            }
            _ => true,
        })
    }

    fn step(&mut self, doc: &Document, p_id: &Ident) {
        let doc: Vec<&Node> = Self::filter(doc).collect();
        for node in &doc {
            match node {
                Node::Elem(Element::Node {
                    name,
                    attrs,
                    children,
                }) => {
                    let id = format_ident!("{}{}", HEAD, self.count);
                    self.count += 1;
                    let tag = name.1.to_string();
                    let old = mem::take(&mut self.children);

                    self.tokens.extend(quote! {
                        let #id = doc.create_element(#tag).unwrap_throw();
                        #p_id.append_child(&#id).unwrap_throw();
                    });

                    self.step(children, &id);

                    self.set_attrs(&id, attrs);
                    self.empty_buff(&id);
                    self.children = old;
                    self.children.push(id)
                }
                Node::Elem(Element::Text(s)) => {
                    if doc.len() == 1 {
                        self.tokens.extend(quote! {
                            #p_id.set_text_content(Some(#s));
                        })
                    } else {
                        todo!()
                    }
                }
                _ => (),
            }
        }
    }

    fn set_attrs(&mut self, id: &Ident, attrs: &[Attribute]) {
        for attr in attrs {
            let value = attr.value.iter().fold(String::new(), |mut acc, x| {
                if let ExprOrText::Text(t) = x {
                    acc.push_str(t)
                }

                acc
            });
            let name = &attr.name;
            self.tokens.extend(quote!(#id.set_attribute(#name, #value).unwrap_throw();));

        }
    }

    fn empty_buff(&mut self, base: &Ident) {
        for child in self.children.drain(..) {
        }
    }
}
