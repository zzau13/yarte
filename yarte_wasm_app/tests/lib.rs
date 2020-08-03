use yarte_wasm_app::*;

#[cfg(not(target_arch = "wasm32"))]
mod test_not_wasm {
    use super::*;
    #[derive(Default)]
    struct T;
    impl App for T {
        type BlackBox = ();
        type Message = ();
    }

    #[should_panic]
    #[test]
    fn test() {
        let _ = run!(T);
    }
}

#[cfg(target_arch = "wasm32")]
mod test {
    use super::*;
    use std::cell::Cell;
    use std::default::Default;
    use std::rc::Rc;

    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    #[derive(Default)]
    struct Test {
        c: Rc<Cell<usize>>,
        any: usize,
        it: Vec<usize>,
        black_box: <Self as App>::BlackBox,
    }

    impl App for Test {
        type BlackBox = BlackBox;
        type Message = Msg;
        fn __dispatch(&mut self, m: Self::Message, addr: &'static Addr<Self>) {
            match m {
                Msg::Msg(i) => msg(self, i, addr),
                Msg::Reset => reset(self, addr),
                Msg::Tree(i) => msg_tree(self, i, addr),
                Msg::Fut(i) => msg_fut(self, i, addr),
            }
        }
    }

    /// PoC tree builder and diff
    /// Tree is a tree of flow with static bool slices for expressions and IfElse flow
    ///  `app.black_box.t_root & [bool; N] != 0`
    /// For iterables vector of bool slices allocated in black box
    ///  `app.black_box.t_children_I[J] & [bool; M] != 0`
    /// management of children will control by `__render`
    /// and macros push or pop or replace
    /// or manual control of black box
    ///
    /// BlackBox trait Tree control
    ///     - render_all:
    ///     - ignore_next_render ?:
    /// public fields in BlackBox for Dom references
    ///
    /// Construct in base of used variables in templates
    ///
    /// this macro is construct in derive
    #[derive(Debug, PartialEq)]
    struct BlackBox {
        t_root: u8,
        t_children_0: Vec<bool>,
    }

    impl Default for BlackBox {
        fn default() -> BlackBox {
            BlackBox {
                t_root: 0xFF,
                t_children_0: vec![],
            }
        }
    }

    impl BlackBox {
        fn set_zero(&mut self) {
            self.t_root = 0;
            for child in self.t_children_0.iter_mut() {
                *child = false;
            }
        }
    }

    #[macro_export]
    macro_rules! set_any {
        ($app:ident, $value:expr) => {
            // fields, index will set in derive
            $app.black_box.t_root |= 1 << 1;
            $app.any = $value;
        };
    }

    #[macro_export]
    macro_rules! set_it {
        ($app:ident, $value:expr) => {
            // fields, index will set in derive
            let value = $value;
            $app.black_box.t_root |= 1 << 2;
            $app.black_box.t_children_0 = vec![true; value.len()];
            $app.it = value;
        };
    }

    // For template
    // ```
    // <h1>{{ c }} {{ any }}</h1>
    // <div>
    //     {{# each it }}
    //         {{ this + 2 }}
    //         <br>
    //     {{/ each }}
    // </div>
    // ```
    //
    // get a black box tree
    // ```
    // struct BlackBox {
    //     t_root: bool,
    //     t_children_0: Vec<bool>,
    //     ...
    // }
    // ```
    //
    #[macro_export]
    macro_rules! push_it {
        ($app:ident, $value:expr) => {
            // fields, index will set in derive
            $app.black_box.t_root |= 1 << 2;
            $app.black_box.t_children_0.push(true);
            $app.it.push($value);
        };
    }

    #[macro_export]
    macro_rules! pop_it {
        ($app:ident) => {{
            // fields, index will set in derive
            $app.black_box.t_root |= 1 << 2;
            $app.black_box.t_children_0.pop();
            $app.it.pop()
        }};
    }

    #[macro_export]
    macro_rules! set_it_index {
        ($app:ident [$i:expr] $value:expr) => {
            // fields, index will set in derive
            $app.black_box.t_root |= 1 << 2;
            $app.black_box
                .t_children_0
                .get_mut($i)
                .map(|x| *x = true)
                .and_then(|_| $app.it.get_mut($i).map(|x| *x = $value))
        };
    }

    // #[templates(..)]
    // #[msg enum Msg {
    //      #[fn(msg)]
    //      Msg,
    //      #[fn(reset)]
    //      Reset,
    //      #[fn(msg_tree)]
    //      MsgTree(usize),
    //      #[fn(msg_fut)]
    //      MsgFut(usize),
    // }]
    enum Msg {
        Msg(usize),
        Reset,
        Tree(usize),
        Fut(usize),
    }

    #[inline]
    fn msg_tree(app: &mut Test, msg: usize, _addr: &Addr<Test>) {
        app.black_box.set_zero();
        // after first render
        let expected = BlackBox {
            t_root: 0,
            t_children_0: vec![],
        };
        assert_eq!(app.black_box, expected);
        set_any!(app, msg);
        set_it!(app, vec![1, 2, 3, 4]);
        let expected = BlackBox {
            t_root: 6,
            t_children_0: vec![true, true, true, true],
        };
        assert_eq!(app.black_box, expected);
        app.black_box.set_zero();
        push_it!(app, 5);
        let expected = BlackBox {
            t_root: 4,
            t_children_0: vec![false, false, false, false, true],
        };
        assert_eq!(app.black_box, expected);
        app.black_box.set_zero();
        let _ = pop_it!(app);
        let expected = BlackBox {
            t_root: 4,
            t_children_0: vec![false, false, false, false],
        };
        assert_eq!(app.black_box, expected);
        app.black_box.set_zero();
        let expected = BlackBox {
            t_root: 0,
            t_children_0: vec![false, false, false, false],
        };
        assert_eq!(app.black_box, expected);
        set_it_index!(app [1] 6);
        let expected = BlackBox {
            t_root: 4,
            t_children_0: vec![false, true, false, false],
        };
        assert_eq!(app.black_box, expected)
    }

    #[inline]
    fn msg(app: &mut Test, msg: usize, _addr: &Addr<Test>) {
        app.c.replace(msg);
    }

    #[inline]
    fn reset(_app: &mut Test, addr: &'static Addr<Test>) {
        addr.send(Msg::Msg(0));
    }

    #[inline]
    fn msg_fut(_app: &mut Test, msg: usize, addr: &'static Addr<Test>) {
        addr.send(Msg::Reset);
        let work = unsafe {
            async_timer::Timed::platform_new_unchecked(
                async move { addr.send(Msg::Msg(msg)) },
                core::time::Duration::from_secs(1),
            )
        };
        spawn_local(async move {
            work.await.unwrap();
        });
    }

    #[wasm_bindgen_test]
    fn test() {
        use std::rc::Rc;
        let c = Rc::new(Cell::new(0));
        let c2 = Rc::clone(&c);
        let app = Test {
            c,
            ..Default::default()
        };
        let addr = unsafe { Addr::run(app) };
        addr.send(Msg::Msg(2));
        assert_eq!(c2.get(), 2);
        addr.send(Msg::Msg(3));
        addr.send(Msg::Msg(1));
        assert_eq!(c2.get(), 1);
        addr.send(Msg::Msg(1));
        addr.send(Msg::Msg(3));
        assert_eq!(c2.get(), 3);
        addr.send(Msg::Reset);
        assert_eq!(c2.get(), 0);
        addr.send(Msg::Msg(3));
        assert_eq!(c2.get(), 3);
        addr.send(Msg::Fut(7));
        assert_eq!(c2.get(), 0);
        let c3 = Rc::clone(&c2);
        let work = unsafe {
            async_timer::Timed::platform_new_unchecked(async {}, core::time::Duration::from_secs(1))
        };
        spawn_local(async move {
            work.await.unwrap();
            assert_eq!(c3.get(), 7);

            // Only use at end
            unsafe { addr.dealloc() };
            // TODO:
            //  #[should_panic]
            // addr.send(Msg::Reset);
        });
        addr.send(Msg::Reset);
        assert_eq!(c2.get(), 0);
        addr.send(Msg::Tree(0));
    }
}
