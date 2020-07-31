use quote::quote;

use super::tokens;

#[test]
fn test() {
    let der = quote! {
        #[derive(App)]
        #[template(src = "<!DOCTYPE html><html><body>{{#if check }}foo{{/if }}</body></html>")]
        #[msg(pub enum Msg {})]
        pub struct Test {
            black_box: <Self as App>::BlackBox,
        }
    };

    let expected = quote! {
#[allow(unused_imports)]
use yarte_wasm_app::*;
#[wasm_bindgen]
extern "C" {
    fn get_state() -> String;
}
impl yarte_wasm_app::App for Test {
    type BlackBox = TestBlackBox;
    type Message = Msg;
    #[doc(hidden)]
    #[inline]
    fn __render(&mut self, __addr: &'static yarte_wasm_app::Addr<Self>) {
        if self.black_box.t_root == <u8 as yarte_wasm_app::YNumber>::zero() {
            return;
        }
        if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & 1u8) {
        }
        self.black_box.t_root = yarte_wasm_app::YNumber::zero();
    }
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, __addr: &'static yarte_wasm_app::Addr<Self>) {}
    #[doc(hidden)]
    fn __dispatch(&mut self, __msg: Self::Message, __addr: &'static yarte_wasm_app::Addr<Self>) {
        use Msg::*;
        match __msg {}
    }
}
pub enum Msg {}
#[derive(Default, serde :: Deserialize)]
struct TestInitialState {}
#[doc = "Internal elements and difference tree"]
pub struct TestBlackBox {
    #[doc = "Each Virtual DOM node"]
    pub __ytable__0: Vec<YComponent0>,
    #[doc = "Difference tree"]
    pub t_root: u8,
}
impl std::default::Default for Test {
    fn default() -> Self {
        let TestInitialState {} = yarte_wasm_app::from_str(&get_state()).unwrap_or_default();
        let doc = yarte_wasm_app::web::window().unwrap_throw().document().unwrap_throw();
        let __ybody = doc.body().unwrap_throw();
        Self {
            black_box: TestBlackBox {
                __ytable__0: __ytable__0,
                t_root: yarte_wasm_app::YNumber::zero(),
            }
        }
    }
}

    }.to_string();

    let c = tokens(der, true);
    assert_eq!(c, expected)
}
