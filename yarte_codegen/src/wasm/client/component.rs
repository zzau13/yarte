use std::{cell::RefCell, collections::HashMap};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use yarte_dom::dom::{Attribute, Element, ExprId, ExprOrText, Node};

use super::WASMCodeGen;

thread_local!(
    static CACHE: RefCell<HashMap<String, Ident>> = RefCell::new(HashMap::new());
);

pub fn clean() {
    CACHE.with(|c| c.borrow_mut().clear())
}

pub fn get_component<'a, I: Iterator<Item = &'a Node>>(
    id: ExprId,
    doc: I,
    builder: &mut WASMCodeGen,
) -> Ident {
    ComponentBuilder::new(id, builder).build(doc)
}

const HEAD: &str = "__n__";

struct ComponentBuilder<'a, 'b> {
    builder: &'a mut WASMCodeGen<'b>,
    id: ExprId,
    count: usize,
    tokens: TokenStream,
}

impl<'a, 'b> ComponentBuilder<'a, 'b> {
    fn new<'n, 'm>(id: ExprId, builder: &'n mut WASMCodeGen<'m>) -> ComponentBuilder<'n, 'm> {
        ComponentBuilder {
            builder,
            id,
            count: 0,
            tokens: TokenStream::new(),
        }
    }

    fn build<'c, I: Iterator<Item = &'c Node>>(mut self, doc: I) -> Ident {
        let ident = format_ident!("component_{}", self.id);

        let doc: Vec<&Node> = Self::filter(doc).collect();

        if doc.len() == 1 {
            match &doc[0] {
                Node::Elem(Element::Node {
                    name,
                    attrs,
                    children,
                }) => {
                    let id = self.get_ident();
                    let tag = match &name.1 {
                        ExprOrText::Text(name) => name,
                        _ => todo!("unsafe tag expression"),
                    };

                    self.tokens.extend(quote! {
                        let #id = doc.create_element(#tag).unwrap_throw();
                    });
                    self.step(children.iter(), &id);
                    self.set_attrs(&id, attrs);

                    self.tokens.extend(quote!(#id))
                }
                _ => todo!("no node element"),
            }
        } else {
            todo!("len +1")
        }

        let tokens = self.tokens.to_string();
        let cached = CACHE.with(|c| {
            if !c.borrow().contains_key(&tokens) {
                c.borrow_mut().insert(tokens, ident.clone());
                return None;
            }
            c.borrow().get(&tokens).cloned()
        });

        cached.unwrap_or_else(|| {
            self.builder.component.push((ident.clone(), self.tokens));
            ident
        })
    }

    fn filter<'c, I: Iterator<Item = &'c Node>>(doc: I) -> impl Iterator<Item = &'c Node> {
        doc.filter(|x| match x {
            Node::Elem(Element::Text(t)) => !t.chars().all(|x| x.is_whitespace()),
            _ => true,
        })
    }

    fn step<'c, I: Iterator<Item = &'c Node>>(&mut self, doc: I, p_id: &Ident) {
        let doc: Vec<&Node> = Self::filter(doc).collect();
        for node in &doc {
            match node {
                Node::Elem(Element::Node {
                    name,
                    attrs,
                    children,
                }) => {
                    let id = self.get_ident();
                    let tag = match &name.1 {
                        ExprOrText::Text(name) => name,
                        _ => todo!("unsafe tag expression"),
                    };

                    self.tokens.extend(quote! {
                        let #id = doc.create_element(#tag).unwrap_throw();
                        #p_id.append_child(&#id).unwrap_throw();
                    });
                    self.set_attrs(&id, attrs);

                    self.step(children.iter(), &id);
                }
                Node::Elem(Element::Text(s)) => {
                    if doc.len() == 1 {
                        self.tokens.extend(quote! {
                            #p_id.set_text_content(Some(#s));
                        })
                    } else {
                        todo!("text +1")
                    }
                }
                _ => (),
            }
        }
    }

    fn set_attrs(&mut self, id: &Ident, attrs: &[Attribute]) {
        for attr in attrs {
            let all_text = attr.value.iter().all(|x| matches!(x, ExprOrText::Text(_)));
            if all_text {
                let value = attr.value.iter().fold(String::new(), |mut acc, x| {
                    if let ExprOrText::Text(t) = x {
                        acc.push_str(t)
                    }

                    acc
                });
                let name = match &attr.name {
                    ExprOrText::Text(name) => name,
                    _ => todo!("Attribute expression"),
                };
                self.tokens
                    .extend(quote!(#id.set_attribute(#name, #value).unwrap_throw();));
            }
        }
    }

    fn get_ident(&mut self) -> Ident {
        let id = format_ident!("{}{}", HEAD, self.count);
        self.count += 1;
        id
    }
}
