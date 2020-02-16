use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Expr, Ident};

use yarte_dom::dom::{Each, ExprId, VarId, VarInner};

use crate::wasm::client::{component::get_component, InsertPath, Len, Parent, State, Step};

use super::{BlackBox, WASMCodeGen};

impl<'a> WASMCodeGen<'a> {
    #[inline]
    pub(super) fn gen_each(
        &mut self,
        id: ExprId,
        Each {
            args,
            body,
            expr,
            var,
        }: Each,
        fragment: bool,
        last: bool,
        insert_point: &[InsertPath],
    ) {
        // Get current state
        let current_bb = self.get_current_black_box();
        let old_on = last!(self).id;

        // Get bases
        let (key, index) = var;
        let var_id = vec![key];
        let mut var_id_index = vec![key];
        let mut bases = HashSet::new();
        bases.insert(key);
        if let Some(index) = index {
            var_id_index.push(index);
            bases.insert(index);
        }

        // Push
        self.stack.push(State {
            id: Parent::Expr(id),
            bases,
            ..Default::default()
        });

        let vdom = Self::get_vdom_ident(id);
        let component_ty = Self::get_component_ty_ident(id);
        let table = Self::get_table_ident(id);
        // TODO: Path to Dom is registered, use old
        let table_dom = Self::get_table_dom_ident(id);

        // Do steps
        let component = get_component(id, &body, self);
        self.step(body);

        // Update state
        let (base, _) = self.get_black_box_t_root(var_id.into_iter());
        last_mut!(self).black_box.push(BlackBox {
            doc: "Difference tree".to_string(),
            name: format_ident!("t_root"),
            ty: parse2(base).unwrap(),
        });

        // TODO: Multiple root
        let roots = vec![Self::get_field_root_ident()];
        last_mut!(self).black_box.push(BlackBox {
            doc: "root dom element".to_string(),
            name: Self::get_field_root_ident(),
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });

        // Write component
        self.helpers.extend(self.get_black_box(&component_ty));
        self.helpers.extend(Self::get_drop(&component_ty, &roots));

        // TODO:
        {
            let current = last_mut!(self);
            for (_, path) in current
                .path_nodes
                .iter_mut()
                .chain(current.path_events.iter_mut())
            {
                if path.starts_with(&[Step::FirstChild, Step::FirstChild]) {
                    // Remove marker
                    path.remove(0);
                } else {
                    todo!("multi node expressions");
                }
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
            &expr,
            &component_ty,
            &insert_point,
            &vdom,
            &table,
            &table_dom,
        );

        let parent = match old_on {
            Parent::Expr(id) => {
                let ident = Self::get_vdom_ident(id);
                quote!(#ident)
            }
            Parent::Body | Parent::Head => quote!(#current_bb.#table_dom),
        };
        let (new, cached) = self.new_each(
            &component,
            &component_ty,
            last,
            insert_point,
            &vdom,
            quote!(#current_bb.#table_dom),
            Some(parent),
        );
        let render = self.render_each(
            new,
            cached,
            &args,
            &expr,
            fragment,
            &vdom,
            quote!(#current_bb.#table),
            quote!(#current_bb.#table_dom),
            key,
        );
        let (new, cached) = self.new_each(
            &component,
            &component_ty,
            last,
            &insert_point,
            &vdom,
            quote!(#table_dom),
            None,
        );
        // Pops
        let current = self.stack.pop().unwrap();
        let mut vars = self
            .tree_map
            .get(&id)
            .cloned()
            .expect("expression registered in tree map");

        for (i, _) in &current.buff_render {
            for j in i {
                let VarInner { base, .. } = self.var_map.get(j).unwrap();
                if !var_id_index.contains(base) {
                    vars.insert(*j);
                }
            }
        }

        // TODO: Expressions in path
        let parent = if fragment {
            self.get_parent_node()
        } else {
            last!(self).steps.len()
        };
        let last = last_mut!(self);
        last.buff_render.push((vars, render));
        last.buff_build.push(build);
        last.buff_new.push(if let Some(cached) = cached {
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
        if !current.path_events.is_empty() {
            let root = Self::get_field_root_ident();
            let steps = Self::get_steps(current.path_events.iter(), quote!(#vdom.#root));
            let hydrate = current.buff_hydrate;
            let hydrate = quote! {
                for (#vdom, #expr) in #current_bb.#table
                        .iter_mut()
                        .zip(#args)
                    {
                        #steps
                        #(#hydrate)*
                    }
            };
            last.buff_hydrate.push(hydrate);
        }
        last.path_nodes
            .push((table_dom.clone(), last.steps[..parent].to_vec()));
        last.black_box.push(BlackBox {
            doc: "Each Virtual DOM node".to_string(),
            name: table,
            ty: parse2(quote!(Vec<#component_ty>)).unwrap(),
        });
        last.black_box.push(BlackBox {
            doc: "Each DOM Element".to_string(),
            name: table_dom,
            ty: parse2(quote!(yarte::web::Element)).unwrap(),
        });
    }

    fn new_each(
        &self,
        component: &Ident,
        component_ty: &Ident,
        last: bool,
        insert_point: &[InsertPath],
        vdom: &Ident,
        table_dom: TokenStream,
        parent: Option<TokenStream>,
    ) -> (TokenStream, Option<TokenStream>) {
        let bb = self.get_global_bbox_ident();
        let tmp = format_ident!("__tmp__");
        let froot = Self::get_field_root_ident();
        let steps = {
            let current = last!(self);
            Self::get_steps(
                current.path_nodes.iter().chain(current.path_events.iter()),
                quote!(#tmp),
            )
        };
        let fields = self.get_black_box_fields(&tmp, false);

        let (insert_point, cached) = if last {
            (
                quote!(#table_dom.append_child(&#vdom.#froot).unwrap_throw();),
                None,
            )
        } else {
            let len: Len = insert_point.into();
            let base = len.base as u32 + 1;
            let mut tokens = quote!(#base);
            for i in &len.expr {
                let ident = Self::get_table_ident(*i);
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
        };

        let build = &last!(self).buff_new;
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

    #[inline]
    fn build_each(
        &mut self,
        args: TokenStream,
        expr: &Expr,
        component_ty: &Ident,
        insert_point: &[InsertPath],
        vdom: &Ident,
        table: &Ident,
        table_dom: &Ident,
    ) -> TokenStream {
        let froot = Self::get_field_root_ident();
        let steps = Self::get_steps(last!(self).path_nodes.iter(), quote!(#vdom));
        let fields = self.get_black_box_fields(vdom, true);
        let build = &last!(self).buff_build;

        let insert_point = {
            let len: Len = insert_point.into();
            let base = len.base as u32;
            let mut tokens = quote!(#base);
            for i in &len.expr {
                let ident = Self::get_table_ident(*i);
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

    #[inline]
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
        each_base: VarId,
    ) -> TokenStream {
        let froot = Self::get_field_root_ident();

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
        let render = if last!(self).buff_render.is_empty() {
            quote!()
        } else {
            let parents = self.get_render_hash().into_iter().any(|(i, _)| {
                for j in i {
                    let base = self.var_map.get(&j).unwrap().base;
                    if base != each_base {
                        return true;
                    }
                }
                false
            });

            let render = self.get_render();
            assert!(!render.is_empty());
            if parents {
                quote! {
                    for (#vdom, #expr) in #table
                        .iter_mut()
                        .zip(#args)
                    {
                        #render
                        #vdom.t_root = yarte::YNumber::zero();
                    }
                }
            } else {
                quote! {
                    for (#vdom, #expr) in #table
                        .iter_mut()
                        .zip(#args)
                        .filter(|(__d__, _)| yarte::YNumber::neq_zero(__d__.t_root))
                        {
                            #render
                            #vdom.t_root = yarte::YNumber::zero();
                        }
                }
            }
        };
        let body = quote! {
            #render
            if __dom_len__ < __data_len__ { #new_block } else {
                #table.drain(__data_len__..);
            }
        };

        // TODO: #[filter] or child is `if`
        let data_len = if true {
            quote!(let __data_len__ = #args.size_hint().0;)
        } else {
            quote!(let __data_len__ = #args.count();)
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
