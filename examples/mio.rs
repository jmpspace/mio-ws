
extern crate mio;

use mio::*;
use mio::tcp::TcpListener;
use std::error::FromError;
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;

// Setup some tokens to allow us to identify which event is
// for which socket.
const SERVER: Token = Token(0);

#[derive(Debug)]
enum TopFatalError { Fatal(String) }

// Macro this
impl FromError<Error> for TopFatalError {
    fn from_error(err: Error) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err.kind())) }
}

/*
impl FromError<WsError> for TopFatalError {
    fn from_error(err: WsError) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
impl FromError<FromUtf8Error> for TopFatalError {
    fn from_error(err: FromUtf8Error) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
*/

fn run() -> Result<(), TopFatalError> {

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 9000));

    // Setup the server socket
    let server = try!(tcp::listen(&addr));

    // Create an event loop
    let mut event_loop = EventLoop::new().unwrap();

    // Start listening for incoming connections
    event_loop.register(&server, SERVER).unwrap();

    // Define a handler to process the events
    struct MyHandler(NonBlock<TcpListener>);

    impl Handler for MyHandler {
        type Timeout = ();
        type Message = ();

        fn readable(&mut self, event_loop: &mut EventLoop<MyHandler>, token: Token, _: ReadHint) {
            match token {
                SERVER => {
                    let MyHandler(ref mut server) = *self;
                    let _ = server.accept();
                    println!("Accepted a socket");
                    // Accept and drop the socket immediately, this will close
                    // the socket and notify the client of the EOF.
                }
                _ => panic!("unexpected token"),
            }
        }
    }

    // Start handling events
    event_loop.run(&mut MyHandler(server)).unwrap();

    Ok(())
}

fn main() {
  match run() {
    Ok(_) => return,
    Err(e) => {
      println!("Got error {:?}", e);
    }
  }
}
