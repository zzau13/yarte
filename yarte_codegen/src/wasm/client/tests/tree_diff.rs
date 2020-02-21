use quote::quote;

use super::tokens;

#[test]
fn test_diff_u16() {
    let src = r#"
    <!doctype html><html><body>
    <div>{{ foo }}{{f1}}{{f2}}{{f3}}{{f4}}{{f5}}{{f6}}{{f7}}{{f8}}</div>
    </body></html>"#;

    let diff = 0b0000_0001_1111_1111u16;
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
                if self.black_box.t_root == <u16 as yarte_wasm_app::YNumber>::zero() {
                    return;
                }

                if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & #diff) {
                    self.black_box.__ynode__0.set_text_content(Some(&format!("{}{}{}{}{}{}{}{}{}", self.foo, self.f1, self.f2, self.f3, self.f4, self.f5, self.f6, self.f7, self.f8)));
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
            pub t_root: u16
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

    assert_eq!(tokens(src), expected)
}

#[test]
fn test_diff_u16_1() {
    let src = r#"
    <!doctype html><html><body>
    <div>{{ foo }}{{f1}}{{f2}}{{f3}}{{f4}}{{f5}}{{f6}}{{f7}}{{f8}}</div>
    <div>{{f9}}</div>
    </body></html>"#;

    let diff_0 = 959u16;
    let diff_1 = 64u16;
    assert_eq!(diff_0 & diff_1, 0);
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
                if self.black_box.t_root == <u16 as yarte_wasm_app::YNumber>::zero() {
                    return;
                }

                if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & #diff_0) {
                    self.black_box.__ynode__0.set_text_content(Some(&format!("{}{}{}{}{}{}{}{}{}", self.foo, self.f1, self.f2, self.f3, self.f4, self.f5, self.f6, self.f7, self.f8)));
                }

                if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & #diff_1) {
                    self.black_box.__ynode__1.set_text_content(Some(&format!("{}", self.f9)));
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
            #[doc = "Yarte Node element" ]
            pub __ynode__1: yarte_wasm_app::web::Element,
            #[doc = "Difference tree"]
            pub t_root: u16
        }

        impl std::default::Default for Test {
            fn default() -> Self {
                let TestInitialState { } = yarte_wasm_app::from_str(&get_state()).unwrap_or_default();
                let doc = yarte_wasm_app::web::window().unwrap_throw().document().unwrap_throw();
                let __ybody = doc.body().unwrap_throw();
                let __ynode__0 = __ybody.first_element_child().unwrap_throw();
                let __ynode__1 = __ynode__0.next_element_sibling().unwrap_throw();
                Self {
                    black_box: TestBlackBox {
                        __ynode__0: __ynode__0,
                        __ynode__1: __ynode__1,
                        t_root: yarte_wasm_app::YNumber::zero()
                    }
                }
            }
        }
    }
        .to_string();

    assert_eq!(tokens(src), expected)
}
