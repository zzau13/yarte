#![cfg(not(target_arch = "wasm32"))]

use std::default::Default;

use futures::channel::mpsc;
use futures::StreamExt;

use delorean::{run, App, Return, A};

/// Reusable Commands
#[derive(Default)]
struct Test {
    commands: Option<mpsc::UnboundedSender<Return<Msg>>>,
}

enum Msg {
    Init,
    Off,
}

// TODO: more coverage
impl App for Test {
    type BlackBox = ();
    type Output = usize;
    type Message = Msg;

    fn __hydrate(&mut self, addr: A<Self>) -> Return<Self::Output> {
        let (tx, mut rx) = mpsc::unbounded();
        self.commands.replace(tx);
        addr.send(Msg::Init);
        Box::pin(async move {
            while let Some(command) = rx.next().await {
                match command.await {
                    Msg::Off => break,
                    msg => addr.send(msg),
                }
            }
            0
        })
    }

    fn __dispatch(&mut self, msg: Self::Message, _addr: A<Self>) {
        match msg {
            Msg::Init => {
                eprintln!("Init");
                let _ = self
                    .commands
                    .as_ref()
                    .unwrap()
                    .unbounded_send(Box::pin(async { Msg::Off }));
            }
            a @ Msg::Off => {
                eprintln!("Off");
                let _ = self
                    .commands
                    .as_ref()
                    .unwrap()
                    .unbounded_send(Box::pin(async move { a }));
            }
        }
    }
}

#[tokio::test]
async fn test() {
    assert_eq!(run!(Test).await, 0);
}
