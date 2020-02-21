use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, punctuated::Punctuated, ExprCall, ExprPath, ExprStruct, Ident, Token};

use yarte_dom::dom::ExprId;

use super::{utils::get_vdom_ident, BlackBox, Parent, WASMCodeGen};

// TODO: attribute prevent default, event and event type
#[allow(unused_variables)]
fn get_closure(msg: &syn::Expr) -> (TokenStream, TokenStream) {
    use syn::Expr::*;
    let (msg, cloned) = match msg {
        Path(ExprPath { attrs, qself, path }) => (quote!(#msg), quote!()),
        Call(ExprCall {
            attrs, func, args, ..
        }) => {
            let mut new: Punctuated<Ident, Token![,]> = Punctuated::new();
            let mut cloned = TokenStream::new();
            for (count, arg) in args.iter().enumerate() {
                let ident = format_ident!("__cloned__{}", count);
                cloned.extend(quote!(let #ident = (#arg).clone();));
                new.push(ident);
            }
            (quote!(#func(#new)), cloned)
        }
        Struct(ExprStruct {
            attrs,
            path,
            fields,
            dot2_token,
            rest,
            ..
        }) => todo!("message struct"),
        _ => panic!("no valid expression at `on` attribute"),
    };
    (
        quote! {
            Closure::wrap(Box::new(move |__event: yarte_wasm_app::web::Event| {
                    __event.prevent_default();
                    __cloned__.send(#msg);
                }) as Box<dyn Fn(yarte_wasm_app::web::Event)>)
        },
        cloned,
    )
}

impl<'a> WASMCodeGen<'a> {
    pub(super) fn write_event(&mut self, id: ExprId, event: &str, msg: &syn::Expr) {
        let name = self.current_node_ident(0);
        assert_eq!(&event[..2], "on");
        let event = &event[2..];
        let vars = self.tree_map.get(&id).expect("registered expression");
        let vars_ident = vars
            .iter()
            .map(|x| {
                self.var_map
                    .get(x)
                    .expect("registered variable")
                    .ident
                    .clone()
            })
            .collect::<Vec<_>>();

        let (forget, dom) = match &last!(self).id {
            Parent::Body => {
                let ident = self.get_global_bb_ident();
                (vars.is_empty(), quote!(self.#ident))
            }
            Parent::Expr(i) => {
                let ident = get_vdom_ident(*i);
                (false, quote!(#ident))
            }
            Parent::Head => todo!(),
        };

        // Make closure expression
        let (closure_expr, clones) = get_closure(msg);

        if forget {
            let current = last_mut!(self);
            current.buff_hydrate.push(quote! {
                #clones
                let __cloned__ = __addr.clone();
                let __closure__ = #closure_expr;
                #name
                    .add_event_listener_with_callback(#event, yarte_wasm_app::JsCast::unchecked_ref(__closure__.as_ref()))
                .unwrap_throw();
                __closure__.forget();
            });
            current.path_events.push((name, current.steps.clone()));
        } else {
            let closure = format_ident!("__closure__{}", self.count);
            self.count += 1;
            let current = self.stack.last_mut().unwrap();
            current.black_box.push(BlackBox {
                doc: "".to_string(),
                name: closure.clone(),
                ty: parse2(quote!(Option<Closure<dyn Fn(yarte_wasm_app::web::Event)>>)).unwrap(),
            });
            current.buff_new.push(quote! {
                    #clones
                    let __cloned__ = __addr.clone();
                    let #closure = Some(#closure_expr);
                    #name
                        .add_event_listener_with_callback(#event, yarte_wasm_app::JsCast::unchecked_ref(#closure.as_ref().unwrap().as_ref()))
                    .unwrap_throw();
                });
            current
                .path_events
                .push((name.clone(), current.steps.clone()));
            current.buff_hydrate.push(quote! {
                    #clones
                    let __cloned__ = __addr.clone();
                    let __closure__ = #closure_expr;
                    #dom.#name
                        .add_event_listener_with_callback(#event, yarte_wasm_app::JsCast::unchecked_ref(__closure__.as_ref()))
                    .unwrap_throw();
                    #dom.#closure.replace(__closure__);
                });
            current.buff_render.push((
                vars.clone(),
                quote! {
                    #dom.#name
                    .remove_event_listener_with_callback(
                        #event,
                        yarte_wasm_app::JsCast::unchecked_ref(#dom
                            .#closure
                            .as_ref()
                            .unwrap_throw()
                            .as_ref()),
                    )
                    .unwrap_throw();
                    #clones
                    let __cloned__ = __addr.clone();
                    #dom.#closure.replace(#closure_expr);
                },
            ));
            current
                .path_nodes
                .push((name.clone(), current.steps.clone()));

            // TODO: duplicated node
            current.black_box.push(BlackBox {
                doc: "Yarte Node element".to_string(),
                name,
                ty: parse2(quote!(yarte_wasm_app::web::Element)).unwrap(),
            });
        };
    }
}
