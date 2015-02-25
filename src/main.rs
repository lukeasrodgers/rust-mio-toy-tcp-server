/*
 * Simple tcp listener, similar to `nc -l 13265`.
 * Will listen for TCP connections on port 13265, accept
 */
extern crate mio;

use mio::*;
use mio::net::{SockAddr};
use mio::net::tcp::{TcpSocket, TcpAcceptor};
use mio::buf::{ByteBuf};

fn main() {
    const SERVER: Token = Token(1);
    let addr = SockAddr::parse("127.0.0.1:13265").unwrap();
    // Listen on `addr`, with a connection backlog of just 1, since we only want to handle one
    // connection. See `man listen`.
    let server = TcpSocket::v4().unwrap()
        .bind(&addr).unwrap()
        .listen(1).unwrap();

    let mut event_loop = EventLoop::<(), ()>::new().unwrap();
    event_loop.register(&server, SERVER).unwrap();

    struct TcpHandler {
        conn: Option<TcpSocket>, // Will store our single socket connection, once established.
        sock: TcpAcceptor // Will store our server.
    };

    impl TcpHandler {
        fn accept(&mut self, event_loop: &mut EventLoop<(), ()>) {
            match self.conn {
                Some(_) => {
                    // If we already have a connection, do nothing.
                    return;
                },
                None => {}
            }
            let conn = self.sock.accept();
            let sock = conn.unwrap().unwrap();
            let tok = Token(2);
            // The following code will move `sock` into our `tcp_server` struct,
            // so we won't be able to access it directly anymore.
            self.conn = Some(sock);
            match self.conn {
                Some(ref c) => {
                    // Register an interest read events with the event_loop. the `edge` option
                    // stipulates that we will only be notified when a read event happens, rather
                    // than as long as reading is possible.
                    // See http://en.wikipedia.org/wiki/Epoll
                    let _ = event_loop.register_opt(c, tok, Interest::readable(), PollOpt::edge());
                }
                None => { }
            }
        }
    }

    impl Handler<(), ()> for TcpHandler {
        fn readable(&mut self, event_loop: &mut EventLoop<(), ()>, token: Token, _: ReadHint) {
            match token {
                SERVER => {
                    // Call `accept` on our `tcp_handler`.
                    self.accept(event_loop);
                }
                Token(2) => {
                    // Artificial limitation -- we'll only read up to 2048 bytes at a time.
                    let mut read_buf = ByteBuf::mut_with_capacity(2048);
                    match self.conn {
                        Some(ref c) => {
                            match c.read(&mut read_buf) {
                                Ok(NonBlock::WouldBlock) => {
                                    panic!("Read operation would block, bailing cuz this shouldn't happen.");
                                }
                                Ok(NonBlock::Ready(_)) => {
                                    // `_` would be the number of bytes read.
                                    // `flip` will return a `ByteBuf` on which we can call
                                    // `read_slice` to get the data available to be read.
                                    // See http://carllerche.github.io/bytes/bytes/struct.ByteBuf.html
                                    let mut buf = read_buf.flip();
                                    let mut sl = [0; 2048];
                                    buf.read_slice(&mut sl);
                                    // Assuming what was written was encoded as UTF8, print what
                                    // was read to STDOUT.
                                    print!("{}", String::from_utf8(sl.to_vec()).unwrap());
                                }
                                Err(e) => {
                                    if e.is_eof() {
                                        println!("Client closed connection, shutting down.");
                                        event_loop.shutdown();
                                    }
                                    else {
                                        panic!(e);
                                    }
                                }
                            }
                        },
                        None => { }
                    }
                },
                _ => { panic!("received token we can't handle".to_string()) }
            }
        }
    }
    let mut tcp_server = TcpHandler {
        conn: None,
        sock: server
    };
    let _ = event_loop.run(&mut tcp_server);
}
