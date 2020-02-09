# Work in progress

## Build 
```bash
wasm-pack build --release --target web 
```


## Generate App Documentation
You can generate the documentation on the BlackBox 
to be able to modify it outside the automatic render cycle
by message
```bash
cargo doc --target wasm32-unknown-unknown --open --no-deps
```

## Client Generated Code
```rust
#[wasm_bindgen]
extern "C" {
    fn get_state() -> String;
}

impl yarte::Template for Test {
    type BlackBox = TestBlackBox;
    type Message = Msg;
    #[doc(hidden)]
    #[inline]
    fn __render(&mut self, __addr: &yarte::Addr<Self>) {
        if self.black_box.t_root == <u8 as yarte::YNumber>::zero() {
            return;
        }
        if yarte::YNumber::neq_zero(self.black_box.t_root & 3u8) {
            let __dom_len__ = self.black_box.__ytable__1.len();
            let __data_len__ = ((&(self.fortunes)).into_iter().enumerate()).size_hint().0;
            for (__dom__1, (__index___0x00000001, __key___0x00000000)) in self
                .black_box
                .__ytable__1
                .iter_mut()
                .zip(((&(self.fortunes)).into_iter().enumerate()))
            {
                if yarte::YNumber::neq_zero(self.black_box.t_root & 2u8)
                    || yarte::YNumber::neq_zero(__dom__1.t_root & 1u8)
                {
                    __dom__1.__ynode__3.set_text_content(Some(&format!(
                        "{} {} {}",
                        self.head, __key___0x00000000.message, __index___0x00000001
                    )));
                    __dom__1
                        .__ynode__4
                        .set_text_content(Some(&format!("{} {}", self.head, __key___0x00000000.message)));
                }
                if yarte::YNumber::neq_zero(__dom__1.t_root & 4u8) {
                    __dom__1
                        .__ynode__2
                        .set_text_content(Some(&format!("{}", __key___0x00000000.id)));
                }
                __dom__1.t_root = yarte::YNumber::zero();
            }
            if __dom_len__ < __data_len__ {
                for (__index___0x00000001, __key___0x00000000) in
                    ((&(self.fortunes)).into_iter().enumerate()).skip(__dom_len__)
                {
                    self.black_box.__ytable__1.push({
                        let __tmp__ = yarte::JsCast::unchecked_into::<yarte::web::Element>(
                            self.black_box.component_1.clone_node_with_deep(true).unwrap_throw(),
                        );
                        let __ynode__1 = __tmp__.first_element_child().unwrap_throw();
                        let __ynode__2 = __ynode__1.next_element_sibling().unwrap_throw();
                        let __ynode__3 = __ynode__2.next_element_sibling().unwrap_throw();
                        let __ynode__4 = __ynode__3
                            .next_element_sibling()
                            .unwrap_throw()
                            .next_element_sibling()
                            .unwrap_throw();
                        __ynode__1.set_text_content(Some(&format!("{}", (__index___0x00000001 + 1))));
                        __ynode__2.set_text_content(Some(&format!("{}", __key___0x00000000.id)));
                        __ynode__3.set_text_content(Some(&format!(
                            "{} {} {}",
                            self.head, __key___0x00000000.message, __index___0x00000001
                        )));
                        __ynode__4.set_text_content(Some(&format!("{} {}", self.head, __key___0x00000000.message)));
                        let __dom__1 = YComponent1 {
                            __ynode__1: __ynode__1,
                            __ynode__2: __ynode__2,
                            __ynode__3: __ynode__3,
                            __ynode__4: __ynode__4,
                            t_root: yarte::YNumber::zero(),
                            __root: __tmp__,
                        };
                        self.black_box
                            .__ytable_dom__1
                            .append_child(&__dom__1.__root)
                            .unwrap_throw();
                        __dom__1
                    });
                }
            } else {
                for __d__ in self.black_box.__ytable__1.drain(__data_len__..) {
                    __d__.__root.remove()
                }
            }
        }
        if yarte::YNumber::neq_zero(self.black_box.t_root & 2u8) {
            self.black_box
                .__ynode__0
                .set_text_content(Some(&format!("{}", self.head)));
        }
        self.black_box.t_root = yarte::YNumber::zero();
    }
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, __addr: &yarte::Addr<Self>) {
        let body = yarte::web::window()
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
            Clear => clear(self, __addr),
        }
    }
}

pub enum Msg {
    Clear,
}

#[derive(Default, serde :: Deserialize)]
struct TestInitialState {
    #[serde(default)]
    fortunes: Vec<Fortune>,
    #[serde(default)]
    head: String,
}

#[doc = "Internal elements and difference tree"]
pub struct TestBlackBox {
    #[doc = "Difference tree"]
    pub t_root: u8,
    #[doc = "Yarte Node element\n\n```\nformat ! (\"{}\", self . head)\n```"]
    pub __ynode__0: yarte::web::Element,
    #[doc = "Each Virtual DOM node"]
    pub __ytable__1: Vec<YComponent1>,
    #[doc = "Each DOM Element"]
    pub __ytable_dom__1: yarte::web::Element,
    #[doc = "Component"]
    pub component_1: yarte::web::Element,
}

#[doc = "Internal elements and difference tree"]
pub struct YComponent1 {
    #[doc = "Yarte Node element\n\n```\nformat ! (\"{}\", (__index___0x00000001 + 1))\n```"]
    pub __ynode__1: yarte::web::Element,
    #[doc = "Yarte Node element\n\n```\nformat ! (\"{}\", __key___0x00000000 . id)\n```"]
    pub __ynode__2: yarte::web::Element,
    #[doc = "Yarte Node element\n\n```\nformat !\n(\"{} {} {}\", self . head, __key___0x00000000 . message, \
             __index___0x00000001)\n```"]
    pub __ynode__3: yarte::web::Element,
    #[doc = "Yarte Node element\n\n```\nformat ! (\"{} {}\", self . head, __key___0x00000000 . message)\n```"]
    pub __ynode__4: yarte::web::Element,
    #[doc = "Difference tree"]
    pub t_root: u8,
    #[doc = "root dom element"]
    pub __root: yarte::web::Element,
}

impl std::default::Default for Test {
    fn default() -> Self {
        let TestInitialState { fortunes, head } = yarte::from_str(&get_state()).unwrap_or_default();
        let doc = yarte::web::window().unwrap_throw().document().unwrap_throw();
        let __ybody = doc.body().unwrap_throw();
        let __ynode__0 = __ybody
            .first_element_child()
            .unwrap_throw()
            .first_element_child()
            .unwrap_throw()
            .next_element_sibling()
            .unwrap_throw();
        let __ytable_dom__1 = __ybody
            .first_element_child()
            .unwrap_throw()
            .next_element_sibling()
            .unwrap_throw()
            .first_element_child()
            .unwrap_throw();
        let mut __ytable__1: Vec<YComponent1> = vec![];
        for (__index___0x00000001, __key___0x00000000) in ((&(fortunes)).into_iter().enumerate()) {
            let __dom__1 = __ytable__1
                .last()
                .map(|__x__| __x__.__root.next_element_sibling().unwrap_throw())
                .unwrap_or_else(|| __ytable_dom__1.children().item(1u32).unwrap_throw());
            let __ynode__1 = __dom__1.first_element_child().unwrap_throw();
            let __ynode__2 = __ynode__1.next_element_sibling().unwrap_throw();
            let __ynode__3 = __ynode__2.next_element_sibling().unwrap_throw();
            let __ynode__4 = __ynode__3
                .next_element_sibling()
                .unwrap_throw()
                .next_element_sibling()
                .unwrap_throw();
            __ytable__1.push(YComponent1 {
                __ynode__1: __ynode__1,
                __ynode__2: __ynode__2,
                __ynode__3: __ynode__3,
                __ynode__4: __ynode__4,
                t_root: yarte::YNumber::zero(),
                __root: __dom__1,
            });
        }
        Self {
            fortunes,
            head,
            black_box: TestBlackBox {
                t_root: yarte::YNumber::zero(),
                __ynode__0: __ynode__0,
                __ytable__1: __ytable__1,
                __ytable_dom__1: __ytable_dom__1,
                component_1: {
                    let __n__0 = doc.create_element("tr").unwrap_throw();
                    let __n__1 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__1).unwrap_throw();
                    __n__1.set_attribute("class", "col-index").unwrap_throw();
                    let __n__2 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__2).unwrap_throw();
                    __n__2.set_attribute("class", "col-id").unwrap_throw();
                    let __n__3 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__3).unwrap_throw();
                    __n__3.set_attribute("class", "col-msg").unwrap_throw();
                    let __n__4 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__4).unwrap_throw();
                    __n__4.set_attribute("class", "another").unwrap_throw();
                    let __n__5 = doc.create_element("a").unwrap_throw();
                    __n__4.append_child(&__n__5).unwrap_throw();
                    __n__5.set_attribute("class", "button").unwrap_throw();
                    __n__5.set_text_content(Some("Delete"));
                    let __n__6 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__6).unwrap_throw();
                    __n__6.set_attribute("class", "col-msg").unwrap_throw();
                    let __n__7 = doc.create_element("td").unwrap_throw();
                    __n__0.append_child(&__n__7).unwrap_throw();
                    __n__7.set_attribute("class", "another").unwrap_throw();
                    let __n__8 = doc.create_element("a").unwrap_throw();
                    __n__7.append_child(&__n__8).unwrap_throw();
                    __n__8.set_attribute("class", "button").unwrap_throw();
                    __n__8.set_text_content(Some("Fol"));
                    __n__0
                },
            },
        }
    }
}
```
