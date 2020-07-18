use quote::quote;

use super::tokens;

#[test]
fn test() {
    let der = quote! {
        #[derive(App)]
        #[template(path = "fortune")]
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
    fn __render(&mut self, __addr: &yarte_wasm_app::Addr<Self>) {
        if self.black_box.t_root == <u8 as yarte_wasm_app::YNumber>::zero() {
            return;
        }
        if yarte_wasm_app::YNumber::neq_zero(self.black_box.t_root & 1u8) {
            let __dom_len__ = self.black_box.__ytable__0.len();
            let __data_len__ = ((&(self.fortunes)).__into_citer()).size_hint().0;
            for (__dom__0, __key___0x00000000) in self
                .black_box
                .__ytable__0
                .iter_mut()
                .zip(((&(self.fortunes)).__into_citer()))
                .filter(|(__d__, _)| yarte_wasm_app::YNumber::neq_zero(__d__.t_root))
            {
                if yarte_wasm_app::YNumber::neq_zero(__dom__0.t_root & 4u8) {
                    __dom__0
                        .__ynode__0
                        .set_text_content(Some(&format!("{}", __key___0x00000000.id)));
                }
                if yarte_wasm_app::YNumber::neq_zero(__dom__0.t_root & 1u8) {
                    __dom__0
                        .__ynode__1
                        .set_text_content(Some(&format!("{}", __key___0x00000000.message)));
                }
                __dom__0.t_root = yarte_wasm_app::YNumber::zero();
            }
            if __dom_len__ < __data_len__ {
                let __cached__ = self
                    .black_box
                    .__ytable_dom__0
                    .children()
                    .item(2u32 + __dom_len__ as u32)
                    .map(yarte_wasm_app::JsCast::unchecked_into::<yarte_wasm_app::web::Node>);
                for __key___0x00000000 in ((&(self.fortunes)).__into_citer()).skip(__dom_len__) {
                    self.black_box.__ytable__0.push({
                        let __tmp__ = yarte_wasm_app::JsCast::unchecked_into::<yarte_wasm_app::web::Element>(
                            self.black_box.component_0.clone_node_with_deep(true).unwrap_throw()
                        );
                        let __ynode__0 = __tmp__.first_element_child().unwrap_throw();
                        let __ynode__1 = __ynode__0.next_element_sibling().unwrap_throw();
                        __ynode__0.set_text_content(Some(&format!("{}", __key___0x00000000.id)));
                        __ynode__1.set_text_content(Some(&format!("{}", __key___0x00000000.message)));
                        let __dom__0 = YComponent0 {
                            __ynode__0: __ynode__0,
                            __ynode__1: __ynode__1,
                            t_root: yarte_wasm_app::YNumber::zero(),
                            __root: __tmp__
                        };
                        self.black_box
                            .__ytable_dom__0
                            .insert_before(&__dom__0.__root, __cached__.as_ref())
                            .unwrap_throw();
                        __dom__0
                    });
                }
            } else {
                self.black_box.__ytable__0.drain(__data_len__..);
            }
        }
        self.black_box.t_root = yarte_wasm_app::YNumber::zero();
    }
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, __addr: &yarte_wasm_app::Addr<Self>) {}
    #[doc(hidden)]
    fn __dispatch(&mut self, __msg: Self::Message, __addr: &yarte_wasm_app::Addr<Self>) {
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
    #[doc = "Each DOM Element"]
    pub __ytable_dom__0: yarte_wasm_app::web::Element,
    #[doc = "Difference tree"]
    pub t_root: u8,
    #[doc = "Component"]
    pub component_0: yarte_wasm_app::web::Element
}
#[doc = "Internal elements and difference tree"]
pub struct YComponent0 {
    #[doc = "Yarte Node element"]
    pub __ynode__0: yarte_wasm_app::web::Element,
    #[doc = "Yarte Node element"]
    pub __ynode__1: yarte_wasm_app::web::Element,
    #[doc = "Difference tree"]
    pub t_root: u8,
    #[doc = "root dom element"]
    pub __root: yarte_wasm_app::web::Element
}
impl Drop for YComponent0 {
    fn drop(&mut self) {
        self.__root.remove();
    }
}
impl std::default::Default for Test {
    fn default() -> Self {
        let TestInitialState {} = yarte_wasm_app::from_str(&get_state()).unwrap_or_default();
        let doc = yarte_wasm_app::web::window().unwrap_throw().document().unwrap_throw();
        let __ybody = doc.body().unwrap_throw();
        let __ytable_dom__0 = __ybody.first_element_child().unwrap_throw();
        let mut __ytable__0: Vec<YComponent0> = vec![];
        for __key___0x00000000 in ((&(fortunes)).__into_citer()) {
            let __dom__0 = __ytable__0
                .last()
                .map(|__x__| __x__.__root.next_element_sibling().unwrap_throw())
                .unwrap_or_else(|| __ytable_dom__0.children().item(1u32).unwrap_throw());
            let __ynode__0 = __dom__0.first_element_child().unwrap_throw();
            let __ynode__1 = __ynode__0.next_element_sibling().unwrap_throw();
            __ytable__0.push(YComponent0 {
                __ynode__0: __ynode__0,
                __ynode__1: __ynode__1,
                t_root: yarte_wasm_app::YNumber::zero(),
                __root: __dom__0
            });
        }
        Self {
            black_box: TestBlackBox {
                __ytable__0: __ytable__0,
                __ytable_dom__0: __ytable_dom__0,
                t_root: yarte_wasm_app::YNumber::zero(),
                component_0: {
                    let __n__0 = doc.create_element("tr").unwrap_throw();
                    let __n__1 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__1).unwrap_throw();
                    let __n__2 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__2).unwrap_throw();
                    __n__0
                }
            }
        }
    }
}

    }.to_string();

    let c = tokens(der, false);
    assert_eq!(c, expected)
}
