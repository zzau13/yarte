use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;

use yarte_helpers::config::Config;
use yarte_hir::{generate, visit_derive};
use yarte_parser::{
    emitter, parse,
    source_map::{clean, get_cursor},
};

use crate::CodeGen;

mod tree_diff;

use super::WASMCodeGen;

fn tokens(i: TokenStream) -> String {
    let config = &Config::new("");
    let der = parse2(i).unwrap();
    let s = visit_derive(&der, config);
    let mut src = BTreeMap::new();
    src.insert(s.path.clone(), s.src.clone());
    let sources = parse(get_cursor(&s.path, &s.src)).unwrap();
    let mut ctx = BTreeMap::new();
    ctx.insert(&s.path, sources);

    let ir = generate(config, &s, &ctx).unwrap_or_else(|e| emitter(&src, config, e.into_iter()));
    clean();

    WASMCodeGen::new(&s).gen(ir).to_string()
}

#[test]
fn test() {
    let src = r#"
    <!doctype html><html><body>
    <div>{{ foo }}</div>
    </body></html>"#;
    let der = quote! {
        #[derive(App)]
        #[template(src = #src, mode = "wasm")]
        #[msg(pub enum Msg {
            Foo,
        })]
        pub struct Test {
            black_box: <Self as App>::BlackBox,
        }
    };

    let expected = quote! {
        #[wasm_bindgen]
        extern "C" {
            fn get_state() -> String;
        }

        impl yarte_wasm_app::App for Test {
            type BlackBox = TestBlackBox;
            type Message = Msg;

            #[doc(hidden)]
            #[inline]
            fn __render (&mut self, __addr: &yarte_wasm_app::Addr<Self>) {
                if self.black_box.t_root == <u8 as yarte_wasm_app::YNumber>::zero() {
                    return;
                }

                if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & 1u8) {
                    self.black_box.__ynode__0.set_text_content(Some(&format!("{}", self.foo) ));
                }

                self.black_box.t_root = yarte_wasm_app::YNumber::zero();
            }

            #[doc(hidden)]
            #[inline]
            fn __hydrate (&mut self, __addr: &yarte_wasm_app::Addr<Self>) { }

            #[doc(hidden)]
            fn __dispatch(&mut self, __msg: Self::Message, __addr: &yarte_wasm_app::Addr<Self>) {
                use Msg::*;
                match __msg {
                    Foo => foo(self, __addr)
                }
            }
        }

        pub enum Msg {
            Foo,
        }

        #[derive(Default, serde::Deserialize)]
        struct TestInitialState { }

        #[doc = "Internal elements and difference tree"]
        pub struct TestBlackBox {
            #[doc = "Yarte Node element" ]
            pub __ynode__0: yarte_wasm_app::web::Element,
            #[doc = "Difference tree"]
            pub t_root: u8
        }

        impl std::default::Default for Test {
            fn default() -> Self {
                let TestInitialState { } = yarte_wasm_app::from_str(&get_state()).unwrap_or_default();
                let doc = yarte_wasm_app::web::window().unwrap_throw().document().unwrap_throw();
                let __ybody = doc.body().unwrap_throw();
                let __ynode__0 = __ybody.first_element_child().unwrap_throw();
                Self {
                    black_box: TestBlackBox {
                        __ynode__0: __ynode__0,
                        t_root: yarte_wasm_app::YNumber::zero()
                    }
                }
            }
        }
    }
    .to_string();

    assert_eq!(tokens(der), expected)
}
