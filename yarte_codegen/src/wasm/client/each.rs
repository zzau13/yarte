use std::mem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, punctuated::Punctuated, Expr, ExprField, FieldValue, Ident, Token};

use yarte_dom::dom::{Each, ExprId};

use crate::wasm::client::{component::get_component, InsertPath, InsertPoint, Len, Parent, Step};

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
        insert_point: InsertPoint,
    ) {
        // TODO: add Each to tree map
        let vars = self.tree_map.get(&id).cloned().unwrap_or_default();
        let parent = self.get_parent_node();
        let current_bb = self.get_current_black_box();
        let froot = Self::get_field_root_ident();

        let on = self.on.replace(Parent::Expr(id));
        let path = self.steps[..parent].to_vec();
        let old_steps = mem::take(&mut self.steps);
        let old_b = mem::take(&mut self.black_box);
        let mut old_buff = mem::take(&mut self.buff_render);

        self.add_black_box_t_root();
        self.black_box.push(BlackBox {
            doc: "root dom element".to_string(),
            name: froot.clone(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        self.step(body);
        let component = get_component(id, body, self);

        let ty = Self::get_component_ty_ident(&id);
        let name = Self::get_table_ident(&id);
        let name_elem = Self::get_table_dom_ident(&id);
        let dom = Self::get_vdom_ident(&id);
        let bb_name = self.get_global_bbox_ident();
        let fields = self.get_black_box_fields(&dom);
        let tmp = format_ident!("__tmp__");
        let fields_new = self.get_black_box_fields(&tmp);

        let black_box = self.empty_black_box(&ty);

        // TODO get current black_box
        let table = quote!(#current_bb.#name);
        let elem = quote!(#current_bb.#name_elem);

        // Remove marker
        eprintln!("{:?}", self.path_nodes);

        let new_build = self.write_steps(quote!(#current_bb.#froot));
        let new = self.write_steps(quote!(#tmp));

        self.build_each(args, new_build, &name, &dom, &name_elem, &ty, &fields);
        let new = self.new_each(
            new,
            &component,
            fields_new,
            tmp,
            &dom,
            &ty,
            &current_bb,
            &elem,
            insert_point,
        );

        old_buff.push((
            vars.clone(),
            self.render_each(table, args, expr, elem, dom, new, fragment),
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
            name: name_elem.clone(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        self.buff_render = old_buff;
        self.path_nodes.push((name_elem, path));
        self.on = on;
        self.steps = old_steps;
    }

    fn new_each(
        &self,
        new: TokenStream,
        component: &Ident,
        fields: Punctuated<FieldValue, Token![,]>,
        tmp: Ident,
        elem: &Ident,
        c_name: &Ident,
        vdom: &TokenStream,
        parent: &TokenStream,
        insert_point: InsertPoint,
    ) -> TokenStream {
        let bb = self.get_global_bbox_ident();
        let froot = Self::get_field_root_ident();
        let insert_point = match insert_point {
            InsertPoint::Append => quote!(#parent.append_child(&#elem.#froot).unwrap_throw();),
            InsertPoint::LastBefore(head, tail) => {
                let mut tokens = TokenStream::new();
                let h_len: Len = head.into();
                let t_len: Len = tail.into();
                if t_len.base == 0 {
                    let mut len = quote!(0);
                    for i in &h_len.expr {
                        let ident = Self::get_table_ident(i);
                        len.extend(quote!(+ #vdom.#ident.len() as u32))
                    }
                    let base = h_len.base as u32 + 1;
                    let mut tokens = quote!(#base);
                    for i in &h_len.expr {
                        let ident = Self::get_table_ident(i);
                        tokens.extend(quote!(+ #vdom.#ident.len() as u32))
                    }
                    quote! {
                        let __len = #len;
                        if __len == 0 {
                            #parent.append_child(&#elem.#froot).unwrap_throw();
                        } else {
                            #parent.insert_before(&#elem.#froot, Some(&#parent.children().item(#tokens).unwrap_throw()));
                        }
                    }
                } else {
                    let base = h_len.base as u32 + 1;
                    let mut tokens = quote!(#base);
                    for i in &h_len.expr {
                        let ident = Self::get_table_ident(i);
                        tokens.extend(quote!(+ #vdom.#ident.len() as u32))
                    }
                    quote!(#parent.insert_before(&#elem.#froot, Some(&#parent.children().item(#tokens).unwrap_throw()));)
                }
            }
        };
        let render = self.buff_render.iter().map(|(_, x)| x);
        quote! {
             let #tmp = yarte::JsCast::unchecked_into::<yarte::web::Element>(self.#bb.#component
                 .clone_node_with_deep(true)
                 .unwrap_throw());
             #new
             let #elem = #c_name { #fields };
             #(#render)*
             #insert_point
             #elem
        }
    }

    fn build_each(
        &mut self,
        args: &Expr,
        new: TokenStream,
        table: &Ident,
        dom: &Ident,
        elem: &Ident,
        c_name: &Ident,
        fields: &Punctuated<FieldValue, Token![,]>,
    ) {
        return;
        // TODO: init element
        // TODO
        let args: TokenStream = quote!(#args)
            .to_string()
            .replace("self .", "")
            .parse()
            .unwrap();
        let init = quote!(first_element_child().unwrap_throw());
        self.buff_build.extend(quote! {
            let mut #table = vec![];
            let mut _iter = #args;
            if let Some(_) = _iter.next() {
                let #dom = #elem.#init;
                #new
                #table.push(#c_name { #fields });
            }
            for _ in _iter {
                let #dom = #table.last().unwrap().root.next_element_sibling().unwrap_throw();
                #new
                #table.push(#c_name { #fields });
            }
        });
    }

    fn render_each(
        &mut self,
        table: TokenStream,
        args: &Expr,
        expr: &Expr,
        elem: TokenStream,
        dom: Ident,
        new: TokenStream,
        fragment: bool,
    ) -> TokenStream {
        let render = self.write_render();
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
                for #expr in #args.skip(dom_len) {
                    #table.push({ #new });
                }
            } else {
                for d in #table.drain(data_len..) {
                    d.root.remove()
                }
            }
        };

        // TODO: #[filter] or child is `if`
        let data_len = if true {
            quote!(let data_len = #args.size_hint().0;)
        } else {
            quote!(let data_len = #args.fold(0. |acc, _| acc + 1);)
        };
        if fragment {
            quote! {
                let dom_len = #table.len();
                #data_len
                #body
            }
        } else {
            quote! {
            let dom_len = #table.len();
            #data_len;
            if data_len == 0 {
                #elem.set_text_content(None);
                #table.clear()
            } else { #body }
            }
        }
    }
}
