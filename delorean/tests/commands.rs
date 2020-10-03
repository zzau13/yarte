#![cfg(not(target_arch = "wasm32"))]

use std::default::Default;

use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::task::Context;
use futures::{Stream, StreamExt};

use tokio::macros::support::{Future, Pin, Poll};
use tokio::spawn;
use tokio::task::JoinHandle;

use delorean::{App, Return, A};

/// Reusable Commands
#[derive(Default)]
struct Test {
    commands: Option<mpsc::UnboundedSender<Msg>>,
}

enum Msg {
    Init,
    Any(usize),
    Off,
}

struct Commander {
    addr: A<Test>,
    threads: Vec<(JoinHandle<()>, oneshot::Receiver<Msg>)>,
    interface: mpsc::UnboundedReceiver<Msg>,
}

impl Commander {
    fn new(addr: A<Test>, interface: mpsc::UnboundedReceiver<Msg>) -> Commander {
        Commander {
            addr,
            interface,
            threads: vec![],
        }
    }
}

impl Drop for Commander {
    fn drop(&mut self) {
        for (j, _) in self.threads.drain(..) {
            let _ = block_on(j);
        }
    }
}

impl Stream for Commander {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Commander {
            addr,
            threads,
            interface,
        } = self.get_mut();
        match Pin::new(interface).poll_next(cx) {
            Poll::Pending => {
                let mut finished = vec![];
                for (i, (_, rx)) in threads.iter_mut().enumerate() {
                    if let Poll::Ready(msg) = Pin::new(rx).poll(cx) {
                        let msg = match msg {
                            Ok(msg) => msg,
                            Err(_) => panic!(),
                        };
                        finished.push(i);
                        addr.send(msg);
                    }
                }
                for i in finished {
                    let _ = threads.remove(i);
                }
                Poll::Pending
            }
            Poll::Ready(Some(msg)) => {
                if let Msg::Off = msg {
                    eprintln!("Commander Off by Msg");
                    return Poll::Ready(None);
                }
                let (tx, rx) = oneshot::channel();
                threads.push((spawn(worker(msg, tx)), rx));
                Poll::Ready(Some(()))
            }
            Poll::Ready(None) => {
                eprintln!("Commander Off by End Stream");
                Poll::Ready(None)
            }
        }
    }
}

async fn worker(msg: Msg, tx: oneshot::Sender<Msg>) {
    let msg = match msg {
        Msg::Any(n) => Msg::Any(n * 2),
        _ => Msg::Off,
    };
    let _ = tx.send(msg);
}

// TODO: more coverage
impl App for Test {
    type BlackBox = ();
    type Output = (usize, A<Self>);
    type Message = Msg;

    fn __hydrate(&mut self, addr: A<Self>) -> Return<Self::Output> {
        let (tx, rx) = mpsc::unbounded();
        self.commands.replace(tx);
        addr.send(Msg::Init);
        let mut commander = Commander::new(addr, rx);
        Box::pin(async move {
            while commander.next().await.is_some() {}
            (0, addr)
        })
    }

    fn __dispatch(&mut self, msg: Self::Message, _addr: A<Self>) {
        match msg {
            Msg::Init => {
                eprintln!("Init");
                let _ = self.commands.as_ref().unwrap().unbounded_send(Msg::Any(1));
            }
            Msg::Any(x) => {
                if x % 2 == 0 {
                    let _ = self.commands.take();
                }
                eprintln!("Any {}", x);
            }
            Msg::Off => {
                eprintln!("Off");
                let _ = self.commands.take();
            }
        }
    }
}

#[tokio::test]
async fn test() {
    let (ret, addr) = unsafe { A::run(Test::default()) }.await;
    assert_eq!(ret, 0);
    unsafe { addr.dealloc() }
}
