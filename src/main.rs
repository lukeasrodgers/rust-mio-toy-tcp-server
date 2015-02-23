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
        conn: Option<TcpSocket>,
        sock: TcpAcceptor
    };

    impl TcpHandler {
        fn accept(&mut self, event_loop: &mut EventLoop<(), ()>) {
            let conn = self.sock.accept();
            let sock = conn.unwrap().unwrap();
            let tok = Token(2);
            self.conn = Some(sock);
            match self.conn {
                Some(ref c) => { event_loop.register_opt(c, tok, Interest::readable(), PollOpt::edge() | PollOpt::oneshot()); }
                None => { }
            }
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
                    let mut interest = Interest::readable();
                    match self.conn {
                        Some(ref c) => {
                            match c.read(&mut read_buf) {
                                Ok(NonBlock::WouldBlock) => {
                                    panic!("We just got readable, but were unable to read from the socket?");
                                }
                                Ok(NonBlock::Ready(r)) => {
                                    let mut buf = read_buf.flip();
                                    let mut sl = [0; 2048];
                                    buf.read_slice(&mut sl);
                                    print!("{}", String::from_utf8(sl.to_vec()).unwrap());
                                    // self.interest.remove(Interest::readable());
                                    // self.interest.insert(Interest::writable());
                                }
                                Err(e) => {
                                    event_loop.shutdown();
                                    // println!("not implemented; client err={:?}", e);
                                    // interest = Interest::hup();
                                    // self.interest.remove(Interest::readable());
                                }
                            }
                            event_loop.reregister(c, tok, interest, PollOpt::edge() | PollOpt::oneshot());
                        },
                        None => { }
                    }
                }
            }
        }
    }
    let mut tcp_server = TcpHandler {
        conn: None,
        sock: server
    };
    let _ = event_loop.run(&mut tcp_server);
}
