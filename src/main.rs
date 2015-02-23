extern crate mio;

use mio::*;
use mio::net::{SockAddr};
use mio::net::tcp::{TcpSocket, TcpAcceptor};
use mio::buf::{ByteBuf, MutByteBuf, SliceBuf};
use mio::util::Slab;

fn main() {
    const SERVER: Token = Token(1);
    let addr = SockAddr::parse("127.0.0.1:13265").unwrap();
    let server = TcpSocket::v4().unwrap()
        .bind(&addr).unwrap()
        .listen(256).unwrap();

    let mut event_loop = EventLoop::<(), ()>::new().unwrap();
    event_loop.register(&server, SERVER).unwrap();

    struct TcpHandler {
        conns: Slab<TcpSocket>,
        sock: TcpAcceptor
    };

    impl TcpHandler {
        fn accept(&mut self, event_loop: &mut EventLoop<(), ()>) {
            let conn = self.sock.accept();
            let sock = conn.unwrap().unwrap();
            let tok = self.conns.insert(sock).ok().expect("could not add connection to slab");
            event_loop.register_opt(&self.conns[tok], tok, Interest::readable(), PollOpt::edge() | PollOpt::oneshot());
        }
    }

    impl Handler<(), ()> for TcpHandler {
        fn readable(&mut self, event_loop: &mut EventLoop<(), ()>, token: Token, _: ReadHint) {
            match token {
                SERVER => {
                    self.accept(event_loop);
                }
                tok => {
                    println!("tok: {}", tok.as_usize());
                    let mut read_buf = ByteBuf::mut_with_capacity(2048);
                    match self.conns[tok].read(&mut read_buf) {
                        Ok(NonBlock::WouldBlock) => {
                            panic!("We just got readable, but were unable to read from the socket?");
                        }
                        Ok(NonBlock::Ready(r)) => {
                            println!("CONN : we read {} bytes!", r);
                            // self.interest.remove(Interest::readable());
                            // self.interest.insert(Interest::writable());
                        }
                        Err(e) => {
                            println!("not implemented; client err={:?}", e);
                            // self.interest.remove(Interest::readable());
                        }
                    }
                }
            }
        }
    }
    let mut tcp_server = TcpHandler {
        conns: Slab::new_starting_at(Token(2), 128),
        sock: server
    };
    let _ = event_loop.run(&mut tcp_server);
}
