use std::collections::BTreeMap;

use quote::quote;
use syn::parse2;

use yarte_config::Config;
use yarte_hir::{generate, visit_derive};
use yarte_parser::{parse, source_map::get_cursor};

use crate::CodeGen;

use super::WASMCodeGen;

fn tokens(src: &str) -> String {
    let config = &Config::new("");
    let path = config.get_dir().join("Test.hbs");
    let der = parse2(quote! {
        #[derive(Template)]
        #[template(src = #src, mode = "wasm")]
        #[msg(pub enum Msg {
            Foo,
        })]
        pub struct Test {
            black_box: <Self as Template>::BlackBox,
        }
    })
    .unwrap();
    let s = visit_derive(&der, config);
    assert_eq!(s.path, path);
    assert_eq!(s.src, src);
    let sources = parse(get_cursor(&path, src)).unwrap();
    let mut ctx = BTreeMap::new();
    ctx.insert(&path, sources);

    let ir = generate(config, &s, &ctx).unwrap_or_else(|e| panic!());

    WASMCodeGen::new(&s).gen(ir).to_string()
}

#[test]
fn test() {
    let src = r#"
    <!doctype html><html><body>
    <div>{{ foo }}</div>
    </body></html>"#;

    let expected = quote! {
        #[wasm_bindgen]
        extern "C" {
            fn get_state() -> String;
        }

        impl yarte::Template for Test {
            type BlackBox = TestBlackBox;
            type Message = Msg;

            #[doc(hidden)]
            #[inline]
            fn __render (&mut self, __addr: &yarte::Addr<Self>) {
                if self.black_box.t_root == <u8 as yarte::YNumber>::zero() {
                    return;
                }

                if yarte::YNumber::neq_zero(self.black_box.t_root & 1u8) {
                    self.black_box.__ynode__0.set_text_content(Some(&format!("{}", self.foo) ));
                }

                self.black_box.t_root = yarte::YNumber::zero();
            }

            #[doc(hidden)]
            #[inline]
            fn __hydrate (&mut self, __addr: &yarte::Addr<Self>) {
                let __ybody = yarte::web::window()
                    .unwrap_throw()
                    .document()
                    .unwrap_throw()
                    .body()
                    .unwrap_throw();
            }

            #[doc(hidden)]
            fn __dispatch(&mut self, __msg: Self::Message, __addr: &yarte::Addr<Self>) {
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
            #[doc = "Yarte Node element\n\n```\nformat ! ( \"{}\" , self . foo )\n```" ]
            pub __ynode__0: yarte::web::Element,
            #[doc = "Difference tree"]
            pub t_root: u8
        }

        impl std::default::Default for Test {
            fn default() -> Self {
                let TestInitialState { } = yarte::from_str(&get_state()).unwrap_or_default();
                let doc = yarte::web::window().unwrap_throw().document().unwrap_throw();
                let __ybody = doc.body().unwrap_throw();
                let __ynode__0 = __ybody.first_element_child().unwrap_throw();
                Self {
                    black_box: TestBlackBox {
                        __ynode__0: __ynode__0,
                        t_root: yarte::YNumber::zero()
                    }
                }
            }
        }
    }
    .to_string();

    assert_eq!(tokens(src), expected)
}
