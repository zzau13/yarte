#![allow(warnings)]
use crate::wasm::client::WASMCodeGen;
use proc_macro2::TokenStream;
use quote::quote;
use yarte_dom::dom::Each;

impl<'a> WASMCodeGen<'a> {
    fn gen_each(&self, e: Box<Each>, insert_point: TokenStream) {
        let Each {
            args,
            body,
            expr,
            var,
        } = *e;
        // TODO: read attributes
        // TODO: attribute #[filter] on each arguments
        //  let row_len = self.data.iter().map(|_| 1).fold(0, |acc, x| acc + x);
        let data = TokenStream::new();
        let vdom = TokenStream::new();
        let data_len = TokenStream::new();
        let vdom_len = TokenStream::new();

        let parent = TokenStream::new();
        let new = TokenStream::new();
        let update = TokenStream::new();

        let build = quote! {
            let mut tbody_children = vec![];
            if let Some(mut curr) = #parent.first_child() {
                loop {
                    let id_node = curr.first_child().unwrap_throw();
                    let label_parent = id_node.next_sibling().unwrap_throw();
                    let label_node = label_parent.first_child().unwrap_throw();
                    let delete_parent = label_parent.next_sibling().unwrap_throw();
                    let delete_node = delete_parent.first_child().unwrap_throw();

                    curr = if let Some(new) = curr.next_sibling() {
                        tbody_children.push(RowDOM {
                            t_root: 0,
                            root: curr,
                            id_node,
                            label_node,
                            delete_node,
                            closure_select: None,
                            closure_delete: None,
                        });

                        new
                    } else {
                        tbody_children.push(RowDOM {
                            t_root: 0,
                            root: curr,
                            id_node,
                            label_node,
                            delete_node,
                            closure_select: None,
                            closure_delete: None,
                        });

                        break;
                    }
                }
            }
        };

        let render_elem = TokenStream::new();
        let new_elem = TokenStream::new();

        let inner = quote! {
            // select
            let (ord, min) = match data_len.cmp(&dom_len) {
                ord @ Ordering::Equal | ord @ Ordering::Greater => (ord, dom_len),
                ord @ Ordering::Less => (ord, row_len),
            };

            // Update
            for (dom, row) #vdom.iter_mut()
                .take(min)
                .zip(#data.take(min))
                .filter(|(dom, _)| dom.t_root != 0)
                {
                    #update
                }

            match ord {
                Ordering::Greater => {
                    // Add
                    for ele in #data.skip(dom_len) {
                        // TODO: select insert point for fragments and insert_before or append_child
                        #vdom.push(#new);
                    }
                }
                Ordering::Less => {
                    // Remove
                    for dom in #vdom.drain(data_len..) {
                        #parent.remove_child(&dom.root).unwrap_throw();
                    }
                }
                Ordering::Equal => (),
            }
        };

        // TODO: not in fragment
        let body = quote! {
            if data_len == 0 {
                // Clear
                #parent.set_text_content(None);
                #vdom.clear()
            } else {
                #inner
            }
        };

        let render = quote! {
            let dom_len = #vdom_len;
            let data_len = #data_len;
            #body
        };

        let hydrate_elem = quote! {};

        let hydrate = quote! {
            assert_eq!(#vdom_len, #data_len);

            // hydrate Each
            for (dom, elem) in #vdom.iter_mut().zip(#data) {
                #hydrate_elem
            }
        };
    }
}
