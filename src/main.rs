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
            if msg.as_slice() == "done\n" {
                event_loop.shutdown();
            }
        }
    }
    let mut event_loop = EventLoop::new().unwrap();
    let sender = event_loop.channel();
    // Send the notification from another thread
    Thread::spawn(move || {
        let mut stdin = old_io::stdin();
        loop {
            for line in stdin.lock().lines() {
                let _ = sender.send(line.unwrap());
            }
        }
    });
    let _ = event_loop.run(EchoHandler);
}
