use std::mem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Expr, Ident};

use yarte_dom::dom::{Each, ExprId};

use crate::wasm::client::{component::get_component, InsertPoint, Len, Parent, Step};

use super::{BlackBox, WASMCodeGen};

impl<'a> WASMCodeGen<'a> {
    pub(super) fn gen_each(
        &mut self,
        id: ExprId,
        Each {
            args, body, expr, ..
        }: &Each,
        fragment: bool,
        insert_point: InsertPoint,
    ) {
        // Get current state
        // TODO: add Each to tree map
        let vars = self.tree_map.get(&id).cloned().unwrap_or_default();
        let current_bb = self.get_current_black_box();

        // Push
        let old_bb = mem::take(&mut self.black_box);
        let old_build = mem::take(&mut self.buff_build);
        let old_new = mem::take(&mut self.buff_new);
        let old_on = self.on.replace(Parent::Expr(id));
        let old_paths = mem::take(&mut self.path_nodes);
        let old_render = mem::take(&mut self.buff_render);
        let old_steps = mem::take(&mut self.steps);

        // Do steps
        self.step(body);

        // Update state
        let component = get_component(id, body, self);
        let component_ty = Self::get_component_ty_ident(&id);

        let vdom = Self::get_vdom_ident(&id);
        let table = Self::get_table_ident(&id);
        let table_dom = Self::get_table_dom_ident(&id);
        self.add_black_box_t_root();
        // TODO: Multiple root
        self.black_box.push(BlackBox {
            doc: "root dom element".to_string(),
            name: Self::get_field_root_ident(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        let black_box = self.get_black_box(&component_ty);

        // TODO:
        for (_, path) in self.path_nodes.iter_mut() {
            if path.starts_with(&[Step::FirstChild, Step::FirstChild]) {
                // Remove marker
                path.remove(0);
            } else {
                todo!("multi node expressions");
            }
        }

        // TODO: remove self
        let build_args: TokenStream = quote!(#args)
            .to_string()
            .replace("self .", "")
            .parse()
            .unwrap();
        let build = self.build_each(
            build_args,
            expr,
            &component_ty,
            &insert_point,
            &vdom,
            &table,
            &table_dom,
        );

        let parent = match old_on.unwrap() {
            Parent::Expr(id) => {
                let ident = Self::get_vdom_ident(&id);
                quote!(#ident)
            }
            Parent::Body | Parent::Head => quote!(#current_bb.#table_dom),
        };
        let (new, cached) = self.new_each(
            &component,
            &component_ty,
            &insert_point,
            &vdom,
            quote!(#current_bb.#table_dom),
            Some(parent),
        );
        let render = self.render_each(
            new,
            cached,
            args,
            expr,
            fragment,
            &vdom,
            quote!(#current_bb.#table),
            quote!(#current_bb.#table_dom),
        );
        let (new, cached) = self.new_each(
            &component,
            &component_ty,
            &insert_point,
            &vdom,
            quote!(#table_dom),
            None,
        );
        // Pops
        self.buff_render = old_render;
        self.buff_render.push((vars.clone(), render));

        self.buff_build = old_build;
        self.buff_build.push(build);

        self.buff_new = old_new;

        self.buff_new.push(if let Some(cached) = cached {
            quote! {
                let __cached__ = #cached;
                let mut #table: Vec<#component_ty> = vec![];
                for #expr in #args.skip(__dom_len__) {
                    #table.push({ #new });
                }
            }
        } else {
            quote! {
                let mut #table: Vec<#component_ty> = vec![];
                for #expr in #args.skip(__dom_len__) {
                        #table.push({ #new });
                }
            }
        });
        self.helpers.extend(black_box);

        self.black_box = old_bb;
        self.black_box.push(BlackBox {
            doc: "Each Virtual DOM node".to_string(),
            name: table,
            ty: parse2(quote!(Vec<#component_ty>)).unwrap(),
        });
        self.black_box.push(BlackBox {
            doc: "Each DOM Element".to_string(),
            name: table_dom.clone(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        self.on = old_on;
        self.steps = old_steps;
        self.path_nodes = old_paths;
        let parent = self.get_parent_node();
        self.path_nodes
            .push((table_dom, self.steps[..parent].to_vec()));
    }

    fn new_each(
        &self,
        component: &Ident,
        component_ty: &Ident,
        insert_point: &InsertPoint,
        vdom: &Ident,
        table_dom: TokenStream,
        parent: Option<TokenStream>,
    ) -> (TokenStream, Option<TokenStream>) {
        let bb = self.get_global_bbox_ident();
        let tmp = format_ident!("__tmp__");
        let froot = Self::get_field_root_ident();
        let steps = self.get_steps(quote!(#tmp));
        let fields = self.get_black_box_fields(&tmp);

        let (insert_point, cached) = match insert_point {
            InsertPoint::Append(_) => (
                quote!(#table_dom.append_child(&#vdom.#froot).unwrap_throw();),
                None,
            ),
            InsertPoint::LastBefore(head, _tail) => {
                let len: Len = head.to_vec().into();
                let base = len.base as u32 + 1;
                let mut tokens = quote!(#base);
                for i in &len.expr {
                    let ident = Self::get_table_ident(i);
                    if let Some(parent) = &parent {
                        tokens.extend(quote!(+ #parent.#ident.len() as u32))
                    } else {
                        tokens.extend(quote!(+ #ident.len() as u32))
                    }
                }

                (
                    quote!(#table_dom.insert_before(&#vdom.#froot, __cached__.as_ref()).unwrap_throw();),
                    Some(if parent.is_some() {
                        quote!(#table_dom.children().item(#tokens + __dom_len__ as u32).map(yarte::JsCast::unchecked_into::<yarte::web::Node>))
                    } else {
                        quote!(#table_dom.children().item(#tokens).map(yarte::JsCast::unchecked_into::<yarte::web::Node>))
                    }),
                )
            }
        };

        let build = &self.buff_new;
        (
            quote! {
                 let #tmp = yarte::JsCast::unchecked_into::<yarte::web::Element>(self.#bb.#component
                     .clone_node_with_deep(true)
                     .unwrap_throw());
                 #steps
                 #(#build)*
                 let #vdom = #component_ty { #fields };
                 #insert_point
                 #vdom
            },
            cached,
        )
    }

    fn build_each(
        &mut self,
        args: TokenStream,
        expr: &Expr,
        component_ty: &Ident,
        insert_point: &InsertPoint,
        vdom: &Ident,
        table: &Ident,
        table_dom: &Ident,
    ) -> TokenStream {
        let froot = Self::get_field_root_ident();
        let steps = self.get_steps(quote!(#vdom));
        let fields = self.get_black_box_fields(vdom);
        let build = &self.buff_build;
        // TODO: simplify
        let head = match insert_point {
            InsertPoint::Append(head) => head,
            InsertPoint::LastBefore(head, _) => head,
        };
        let insert_point = {
            let len: Len = head.to_vec().into();
            let base = len.base as u32;
            let mut tokens = quote!(#base);
            for i in &len.expr {
                let ident = Self::get_table_ident(i);
                tokens.extend(quote!(+ #ident.len() as u32))
            }

            quote!(#table_dom.children().item(#tokens).unwrap_throw())
        };

        quote! {
            let mut #table: Vec<#component_ty> = vec![];
            for #expr in #args {
                let #vdom = #table.last().map(|__x__| __x__.#froot.next_element_sibling().unwrap_throw()).unwrap_or_else(|| #insert_point);
                #steps
                #(#build)*
                #table.push(#component_ty { #fields });
            }
        }
    }

    fn render_each(
        &self,
        new: TokenStream,
        cached: Option<TokenStream>,
        args: &Expr,
        expr: &Expr,
        fragment: bool,
        vdom: &Ident,
        table: TokenStream,
        table_dom: TokenStream,
    ) -> TokenStream {
        let render = self.get_render();
        let froot = Self::get_field_root_ident();
        // TODO get parents dependency
        let check = quote!(|(d, _)| d.t_root != 0);

        // TODO: remove for fragments
        // TODO: remove on drop
        // TODO: remove component method
        let new_block = if let Some(cached) = &cached {
            quote! {
                let __cached__ = #cached;
                for #expr in #args.skip(__dom_len__) {
                    #table.push({ #new });
                }
            }
        } else {
            quote! {
                for #expr in #args.skip(__dom_len__) {
                        #table.push({ #new });
                }
            }
        };
        let body = quote! {
            for (#vdom, #expr) in #table
                .iter_mut()
                .zip(#args)
                .filter(#check)
                {
                    #render
                    #vdom.t_root = 0;
                }

            if __dom_len__ < __data_len__ { #new_block } else {
                for __d__ in #table.drain(__data_len__..) {
                    __d__.#froot.remove()
                }
            }
        };

        // TODO: #[filter] or child is `if`
        let data_len = if true {
            quote!(let __data_len__ = #args.size_hint().0;)
        } else {
            quote!(let __data_len__ = #args.fold(0, |acc, _| acc + 1);)
        };
        if fragment {
            quote! {
                let __dom_len__ = #table.len();
                #data_len
                #body
            }
        } else {
            quote! {
                let __dom_len__ = #table.len();
                #data_len;
                if __data_len__ == 0 {
                    #table_dom.set_text_content(None);
                    #table.clear()
                } else { #body }
            }
        }
    }
}
