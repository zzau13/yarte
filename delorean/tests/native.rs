#![cfg(not(target_arch = "wasm32"))]

use std::cell::Cell;
use std::default::Default;
use std::rc::Rc;
use std::thread::{sleep, spawn};
use std::time::Duration;

use futures::channel::oneshot::{self, Sender};
use futures::executor::block_on;
use futures::future::join;

use delorean::{App, Return, A};

#[derive(Default)]
struct Test {
    c: Rc<Cell<usize>>,
    off: Option<Sender<(usize, A<Test>)>>,
}

enum Msg {
    Set(usize),
    Off,
}

// TODO: more coverage
impl App for Test {
    type BlackBox = ();
    type Output = (usize, A<Self>);
    type Message = Msg;

    fn __hydrate(&mut self, addr: A<Self>) -> Return<Self::Output> {
        let (tx, rx) = oneshot::channel();
        // Join in local with channels
        let _ = spawn(|| {
            sleep(Duration::new(1, 0));
            let _ = tx.send(());
        });
        let work2 = async move {
            rx.await.unwrap();
            addr.send(Msg::Set(0xFF));
        };

        let (tx, rx) = oneshot::channel();
        let _ = spawn(|| {
            sleep(Duration::new(2, 0));
            let _ = tx.send(());
        });
        let work = async move {
            rx.await.unwrap();
            addr.send(Msg::Set(1));
        };

        // TODO: more coverage but is so easy
        let (tx, rx) = oneshot::channel();
        self.off.replace(tx);
        let fut = join(rx, join(work, work2));
        Box::pin(async { fut.await.0.unwrap() })
    }

    fn __dispatch(&mut self, msg: Self::Message, addr: A<Self>) {
        match msg {
            Msg::Set(x) => {
                if x == 0xFF {
                    eprintln!("First");
                    assert_eq!(self.c.get(), 0);
                }
                if x == 1 {
                    eprintln!("Last");
                    assert_eq!(self.c.get(), 0xFF);
                    addr.send(Msg::Off);
                }
                eprintln!("Set {}", x);
                self.c.set(x)
            }
            Msg::Off => {
                let _ = self.off.take().unwrap().send((0, addr));
            }
        }
    }
}

#[test]
fn test() {
    let c = Default::default();
    let c2 = Rc::clone(&c);

    let app = Test {
        c,
        ..Default::default()
    };
    let (ret, addr) = block_on(unsafe { A::run(app) });
    assert_eq!(ret, 0);
    unsafe { addr.dealloc() }

    assert_eq!(c2.get(), 1);
}
