use std::mem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse2;

use yarte_dom::dom::{Each, ExprId};

use super::{BlackBox, WASMCodeGen};

impl<'a> WASMCodeGen<'a> {
    pub(super) fn gen_each(
        &mut self,
        id: ExprId,
        Each {
            args,
            body,
            expr,
            var,
        }: &Each,
        insert_point: TokenStream,
    ) {
        let vars = self.tree_map.get(&id).cloned().unwrap_or_default();
        let parent = self.parent_node();

        let node = self.on.unwrap();
        let v = &self.steps[..parent];
        let old_b = mem::take(&mut self.black_box);
        let mut old_buff = mem::take(&mut self.buff_render);

        self.do_step(body, id);

        let ty = format_ident!("Component{}", id);
        let name = format_ident!("ytable_{}", id);
        let name_elem = format_ident!("ytable_dom_{}", id);
        self.add_black_box_t_root();
        self.black_box.push(BlackBox {
            doc: "Each root dom element".to_string(),
            name: name_elem.clone(),
            ty: parse2(quote!(Vec<#ty>)).unwrap(),
        });
        self.black_box.push(BlackBox {
            doc: "Each dom elements".to_string(),
            name: name.clone(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });
        let black_box = self.black_box(&ty);
        self.helpers.extend(black_box);
        self.black_box = old_b;
        let bb_name = self.get_black_box_ident();
        let table = quote!(self.#bb_name.#name);
        let render = self.empty_buff();
        let dom = format_ident!("dom_{}", id);
        let new = TokenStream::new();
        old_buff.push((
            vars.clone(),
            quote! {
            let dom_len = #table.len();
            let data_len = #args.size_hint().0;
            if data_len == 0 {
                self.#name_elem.set_text_content(None);
                #table.clear()
            } else {
                for (#dom, #expr) in #table
                    .iter_mut()
                    .zip(#args)
                    .filter(|(dom, _)| dom.t_root != 0) { #render }

                if dom_len < data_len {
                    for row in self.data.iter().skip(dom_len) {
                        #table.push(new_row!(row, self.tr, mb, self.tbody));
                    }
                } else {
                    for dom in #table.drain(data_len..) {
                        dom.root.remove()
                    }
                }
            }
            },
        ));
        self.buff_render = old_buff;
    }
}
