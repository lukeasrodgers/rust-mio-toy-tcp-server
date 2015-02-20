extern crate mio;


use mio::*;
use mio::net::{SockAddr};
use mio::net::tcp::{TcpSocket, TcpAcceptor};
use std::thread::Thread;
use std::old_io;

fn main() {
    struct EchoHandler;

    impl Handler<(), String> for EchoHandler {
        fn notify(&mut self, event_loop: &mut EventLoop<(), String>, msg: String) {
            print!("{}", msg);
            if msg.as_slice() == "done" {
                event_loop.shutdown();
            }
        }
    }
    let mut event_loop = EventLoop::new().unwrap();
    let sender = event_loop.channel();

    Thread::spawn(move || {
        for line in old_io::stdin().lock().lines() {
            let _ = sender.send(line);
        }
    });

    let _ = event_loop.run(EchoHandler);
}
