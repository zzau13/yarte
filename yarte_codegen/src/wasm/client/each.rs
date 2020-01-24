use std::mem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Expr, Ident};

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
        fragment: bool,
        insert_point: TokenStream,
    ) {
        // TODO: add Each to tree map
        let vars = self.tree_map.get(&id).cloned().unwrap_or_default();
        let parent = self.parent_node();

        let node = self.on.unwrap();
        let v = &self.steps[..parent];
        let old_b = mem::take(&mut self.black_box);
        let mut old_buff = mem::take(&mut self.buff_render);

        self.add_black_box_t_root();
        self.black_box.push(BlackBox {
            doc: "root dom element".to_string(),
            name: format_ident!("root"),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        self.do_step(body, id);
        self.component(id, body);

        let ty = format_ident!("Component{}", id);
        let name = format_ident!("ytable_{}", id);
        let name_elem = format_ident!("ytable_dom_{}", id);

        let black_box = self.black_box(&ty);

        // TODO get current black_box
        let bb_name = self.get_black_box_ident();
        let c_bb = quote!(self.#bb_name);
        let table = quote!(#c_bb.#name);
        let elem = quote!(#c_bb.#name_elem);
        let dom = format_ident!("dom_{}", id);

        old_buff.push((
            vars.clone(),
            self.render_each(table, args, expr, elem, dom, fragment),
        ));

        self.helpers.extend(black_box);

        self.black_box = old_b;
        self.black_box.push(BlackBox {
            doc: "Each root dom element".to_string(),
            name,
            ty: parse2(quote!(Vec<#ty>)).unwrap(),
        });
        self.black_box.push(BlackBox {
            doc: "Each dom elements".to_string(),
            name: name_elem,
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        self.buff_render = old_buff;
    }

    fn render_each(
        &mut self,
        table: TokenStream,
        args: &Expr,
        expr: &Expr,
        elem: TokenStream,
        dom: Ident,
        fragment: bool,
    ) -> TokenStream {
        let new = TokenStream::new();
        let render = self.empty_buff();
        // TODO get parents dependency
        let check = quote!(d.t_root != 0);

        // TODO: remove for fragments
        let body = quote! {
            for (#dom, #expr) in #table
                .iter_mut()
                .zip(#args)
                .filter(|(d, _)| #check)
                { #render }

            if dom_len < data_len {
                for row in #args.skip(dom_len) {
                    #table.push(#new);
                }
            } else {
                for d in #table.drain(data_len..) {
                    d.root.remove()
                }
            }
        };

        if fragment {
            quote! {
                let dom_len = #table.len();
                let data_len = #args.size_hint().0;
                #body
            }
        } else {
            quote! {
            let dom_len = #table.len();
            let data_len = #args.size_hint().0;
            if data_len == 0 {
                #elem.set_text_content(None);
                #table.clear()
            } else { #body }
            }
        }
    }
}
