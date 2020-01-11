/// Adapted from [`actix`](https://github.com/actix/actix) and [`draco`](https://github.com/utkarshkukreti/draco)
use std::{
    cell::{Cell, RefCell},
    default::Default,
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

mod queue;

use self::queue::Queue;

/// App are object which encapsulate state and behavior
///
///
/// App communicate exclusively by directional exchanging messages
/// The sender can't wait the response since it never answer
// TODO: derive
pub trait App: Default + Sized + Unpin + 'static {
    type BlackBox: Default;
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    fn __render(&mut self, _mb: &Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    fn __hydrate(&mut self, _mb: &Addr<Self>) {}

    /// Private: Start a new asynchronous app, returning its address.
    #[doc(hidden)]
    fn __start(self) -> Addr<Self>
    where
        Self: App,
    {
        Addr(Rc::new(Context::new(self)))
    }

    /// Construct and start a new asynchronous app, returning its
    /// address.
    ///
    /// This is constructs a new app using the `Default` trait, and
    /// invokes its `start` method.
    fn start_default() -> Addr<Self>
    where
        Self: App,
    {
        Self::default().__start()
    }
}

/// The address of App
pub struct Addr<A: App>(Rc<Context<A>>);

impl<A: App> Addr<A> {
    /// Enqueue message
    fn push(&self, env: Envelope<A>) {
        self.0.q.push(env);
        self.update();
    }

    /// Sends a message
    ///
    /// The message is always queued
    #[inline]
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        self.push(Envelope::new(msg));
    }

    #[inline]
    fn update(&self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);
            while let Some(mut env) = self.0.q.pop() {
                env.handle(&mut self.0.app.borrow_mut(), &self)
            }
            self.0.ready.replace(true);
            self.render();
        }
    }

    /// Render app
    ///
    /// if app if not ready will render when it's ready
    #[inline]
    fn render(&self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);
            self.0.app.borrow_mut().__render(&self);
            self.0.ready.replace(true);
            if !self.0.q.is_empty() {
                self.update()
            }
        }
    }

    /// Hydrate app
    ///
    /// Link events and get nodes
    pub fn hydrate(&self) {
        assert!(!self.0.ready.get());
        self.0.app.borrow_mut().__hydrate(&self);
        self.0.ready.replace(true);
        if !self.0.q.is_empty() {
            self.update()
        }
    }
}

impl<A: App> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr(Rc::clone(&self.0))
    }
}

/// Encapsulate inner context of the App
pub struct Context<A: App> {
    app: RefCell<A>,
    q: Queue<Envelope<A>>,
    ready: Cell<bool>,
}

impl<A: App> Context<A> {
    fn new(app: A) -> Self {
        Self {
            app: RefCell::new(app),
            q: Queue::new(),
            ready: Cell::new(false),
        }
    }
}

/// Envelope `Message` in a `App` type
pub struct Envelope<A: App>(Box<dyn EnvelopeProxy<App = A>>);

impl<A: App> Envelope<A> {
    pub fn new<M>(msg: M) -> Self
    where
        A: Handler<M>,
        M: Message,
    {
        Envelope(Box::new(SyncEnvelopeProxy {
            msg: Some(msg),
            act: PhantomData,
        }))
    }
}

trait EnvelopeProxy {
    type App: App;

    fn handle(&mut self, act: &mut Self::App, addr: &Addr<Self::App>);
}

impl<A: App> EnvelopeProxy for Envelope<A> {
    type App = A;

    #[inline]
    fn handle(&mut self, act: &mut Self::App, addr: &Addr<Self::App>) {
        self.0.handle(act, addr)
    }
}

struct SyncEnvelopeProxy<A, M>
where
    M: Message,
{
    act: PhantomData<A>,
    msg: Option<M>,
}

impl<A, M> EnvelopeProxy for SyncEnvelopeProxy<A, M>
where
    M: Message,
    A: App + Handler<M>,
{
    type App = A;

    #[inline]
    fn handle(&mut self, act: &mut Self::App, addr: &Addr<A>) {
        if let Some(msg) = self.msg.take() {
            <Self::App as Handler<M>>::handle(act, msg, addr);
        }
    }
}

/// Represent message that can be handled by the app.
pub trait Message: 'static {}

/// Describes how to handle messages of a specific type.
///
/// Implementing `Handler` is a way to handle incoming messages
///
/// The type `M` is a message which can be handled by the app.
pub trait Handler<M>
where
    Self: App,
    M: Message,
{
    /// This method is called for every message type `M` received by this app
    fn handle(&mut self, msg: M, mb: &Addr<Self>);
}

/// Allow users to use `Arc<M>` as a message without having to re-impl `Message`
impl<M> Message for Arc<M> where M: Message {}

/// Allow users to use `Box<M>` as a message without having to re-impl `Message`
impl<M> Message for Box<M> where M: Message {}

#[cfg(test)]
mod test {
    #![allow(dead_code)]

    use super::*;
    use std::{
        default::Default,
        rc::Rc,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    #[derive(Default)]
    struct Test {
        c: Rc<AtomicUsize>,
        any: usize,
        it: Vec<usize>,
        black_box: <Self as App>::BlackBox,
    }

    impl App for Test {
        type BlackBox = BlackBox;
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

    struct MsgTree(usize);

    impl Message for MsgTree {}

    impl Handler<MsgTree> for Test {
        fn handle(&mut self, msg: MsgTree, _mb: &Addr<Self>) {
            self.black_box.set_zero();
            // after first render
            let expected = BlackBox {
                t_root: 0,
                t_children_0: vec![],
            };
            assert_eq!(self.black_box, expected);
            set_any!(self, msg.0);
            set_it!(self, vec![1, 2, 3, 4]);
            let expected = BlackBox {
                t_root: 6,
                t_children_0: vec![true, true, true, true],
            };
            assert_eq!(self.black_box, expected);
            self.black_box.set_zero();
            push_it!(self, 5);
            let expected = BlackBox {
                t_root: 4,
                t_children_0: vec![false, false, false, false, true],
            };
            assert_eq!(self.black_box, expected);
            self.black_box.set_zero();
            let _ = pop_it!(self);
            let expected = BlackBox {
                t_root: 4,
                t_children_0: vec![false, false, false, false],
            };
            assert_eq!(self.black_box, expected);
            self.black_box.set_zero();
            let expected = BlackBox {
                t_root: 0,
                t_children_0: vec![false, false, false, false],
            };
            assert_eq!(self.black_box, expected);
            set_it_index!(self [1] 6);
            let expected = BlackBox {
                t_root: 4,
                t_children_0: vec![false, true, false, false],
            };
            assert_eq!(self.black_box, expected)
        }
    }

    struct Msg(usize);

    impl Message for Msg {}

    impl Handler<Msg> for Test {
        fn handle(&mut self, msg: Msg, _mb: &Addr<Self>) {
            self.c.store(msg.0, Ordering::Relaxed);
        }
    }

    struct Reset;

    impl Message for Reset {}

    impl Handler<Reset> for Test {
        fn handle(&mut self, _: Reset, mb: &Addr<Self>) {
            mb.send(Msg(0));
        }
    }

    struct MsgFut(usize);

    impl Message for MsgFut {}

    impl Handler<MsgFut> for Test {
        fn handle(&mut self, msg: MsgFut, mb: &Addr<Self>) {
            mb.send(Reset);
            let mb = mb.clone();
            let work = unsafe {
                async_timer::Timed::platform_new_unchecked(
                    async move { mb.send(Msg(msg.0)) },
                    core::time::Duration::from_secs(1),
                )
            };
            spawn_local(async move {
                work.await.unwrap();
            });
        }
    }

    #[wasm_bindgen_test]
    fn test() {
        let c = Rc::new(AtomicUsize::new(0));
        let c2 = Rc::clone(&c);
        let app = Test {
            c,
            ..Default::default()
        };
        let addr = app.__start();
        addr.hydrate();
        let addr2 = addr.clone();
        addr.send(Msg(2));
        assert_eq!(c2.load(Ordering::Relaxed), 2);
        addr2.send(Msg(3));
        addr.send(Msg(1));
        assert_eq!(c2.load(Ordering::Relaxed), 1);
        addr.send(Msg(1));
        addr2.send(Msg(3));
        assert_eq!(c2.load(Ordering::Relaxed), 3);
        addr2.send(Reset);
        assert_eq!(c2.load(Ordering::Relaxed), 0);
        addr2.send(Msg(3));
        assert_eq!(c2.load(Ordering::Relaxed), 3);
        addr2.send(MsgFut(7));
        assert_eq!(c2.load(Ordering::Relaxed), 0);
        let c3 = Rc::clone(&c2);
        let work = unsafe {
            async_timer::Timed::platform_new_unchecked(async {}, core::time::Duration::from_secs(1))
        };
        spawn_local(async move {
            work.await.unwrap();
            assert_eq!(c3.load(Ordering::Relaxed), 7);
            addr2.send(Reset);
            assert_eq!(c3.load(Ordering::Relaxed), 0);
        });
        addr.send(Reset);
        assert_eq!(c2.load(Ordering::Relaxed), 0);
        addr.send(MsgTree(0))
    }
}
