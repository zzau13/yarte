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
// TODO: lifecycle ?
pub trait App: Default + Sized + Unpin + 'static {
    type BlackBox: Default;
    /// empty for overridden in derive
    #[doc(hidden)]
    // TODO: derive
    fn __render(&mut self, _ctx: &Mailbox<Self>) {}

    /// Start a new asynchronous app, returning its address.
    fn start(self) -> Addr<Self>
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
        Self::default().start()
    }
}

/// The address of App
pub struct Addr<A: App>(Rc<Context<A>>);

impl<A: App> Addr<A> {
    fn push(&self, env: Envelope<A>) {
        self.0.q.push(env);
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        self.push(Envelope::new(msg));
        self.update();
    }

    fn update(&self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);

            let mailbox = self.mailbox();
            while let Some(mut env) = self.0.q.pop() {
                env.handle(&mut self.0.app.borrow_mut(), &mailbox)
            }

            self.0.ready.replace(true);
            self.render();
        }
    }

    /// Render app
    ///
    /// if app if not ready will render when it's ready
    pub fn render(&self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);
            self.0.app.borrow_mut().__render(&self.mailbox());
            self.0.ready.replace(true);
            if !self.0.q.is_empty() {
                self.update()
            }
        }
    }

    /// Get new mailbox
    fn mailbox(&self) -> Mailbox<A> {
        let cloned = self.clone();
        Mailbox::new(move |env| {
            cloned.push(env);
            cloned.update();
        })
    }
}

impl<A: App> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr(Rc::clone(&self.0))
    }
}

// TODO: benchmark queue find other options
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
            ready: Cell::new(true),
        }
    }
}

/// MailBox of messages enveloped with App type
pub struct Mailbox<A: App>(Rc<Box<dyn Fn(Envelope<A>) + 'static>>);

impl<A: App> Mailbox<A> {
    fn new(f: impl Fn(Envelope<A>) + 'static) -> Self {
        Mailbox(Rc::new(Box::new(f)))
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Message,
    {
        (self.0)(Envelope::new(msg))
    }
}

impl<A: App> Clone for Mailbox<A> {
    fn clone(&self) -> Self {
        Mailbox(Rc::clone(&self.0))
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

    fn handle(&mut self, act: &mut Self::App, ctx: &Mailbox<Self::App>);
}

impl<A: App> EnvelopeProxy for Envelope<A> {
    type App = A;

    fn handle(&mut self, act: &mut Self::App, ctx: &Mailbox<Self::App>) {
        self.0.handle(act, ctx)
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

    fn handle(&mut self, act: &mut Self::App, ctx: &Mailbox<A>) {
        if let Some(msg) = self.msg.take() {
            <Self::App as Handler<M>>::handle(act, msg, ctx);
        }
    }
}

pub trait Message: 'static {}

pub trait Handler<M>
where
    Self: App,
    M: Message,
{
    fn handle(&mut self, msg: M, ctx: &Mailbox<Self>);
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
        black_box: <Self as App>::BlackBox,
    }

    impl App for Test {
        type BlackBox = ();
    }

    struct Msg(usize);

    impl Message for Msg {}

    impl Handler<Msg> for Test {
        fn handle(&mut self, msg: Msg, _ctx: &Mailbox<Self>) {
            self.c.store(msg.0, Ordering::Relaxed);
        }
    }

    struct Reset;

    impl Message for Reset {}

    impl Handler<Reset> for Test {
        fn handle(&mut self, _: Reset, ctx: &Mailbox<Self>) {
            ctx.send(Msg(0));
        }
    }

    struct MsgFut(usize);

    impl Message for MsgFut {}

    impl Handler<MsgFut> for Test {
        fn handle(&mut self, msg: MsgFut, ctx: &Mailbox<Self>) {
            let ctx = ctx.clone();
            let work = unsafe {
                async_timer::Timed::platform_new_unchecked(
                    async move { ctx.send(Msg(msg.0)) },
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
        let addr = app.start();
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
        })
    }
}
