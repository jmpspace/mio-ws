// unstable libraries
#![feature(io,net)]

extern crate "websocket-protocol" as ws;

use std::io::{Error};
use std::net::{TcpListener, TcpStream};
use std::thread;
use ws::protocol::{WsError, WebSocketStream};

fn handle_client(stream: TcpStream) -> Result<(), WsError> {

    println!("Handling client");
    
    let mut ws = try!(WebSocketStream::new(stream));
    
    println!("Opened ws");

    try!(ws.send(format!("Test message").as_bytes()));
    
    loop {
        let msg = try!(ws.recv());
        try!(ws.send(&msg[..]));
    }
    
}

fn run() -> Result<(), Error> {

    let listener = try!(TcpListener::bind("0.0.0.0:9000"));

    for stream in listener.incoming() {
        
        match stream {
            Err(e) => { 
                println!("Accept error: {:?}", e); 
            }
            Ok(stream) => {
              thread::spawn(move|| {
                match handle_client(stream) {
                    Ok(()) => println!("Client quit without error"),
                    Err(e) => println!("Client exited with error: {:?}", e)
                };
              });
            }
        }
    }
    
    return Ok(());
}

fn main() {
  match run() {
    Ok(_) => return,
    Err(e) => {
      println!("Got error {:?}", e);
    }
  }
}