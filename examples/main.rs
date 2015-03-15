// unstable libraries
#![feature(io,net,rand)]

extern crate "websocket-protocol" as ws;

use std::collections::{HashMap};
use std::error::FromError;
use std::io::{Error};
use std::net::{TcpListener, TcpStream};
use std::rand::{thread_rng};
use std::rand::Rng;
use std::string::{FromUtf8Error};
use std::sync::mpsc::{Receiver, SendError, SyncSender, sync_channel};
use std::thread;

use ws::protocol::{TryClone, WsError, WebSocketStream};

#[derive(Debug)]
enum TopFatalError { Fatal(String) }

// Macro this
impl FromError<WsError> for TopFatalError {
    fn from_error(err: WsError) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
impl FromError<FromUtf8Error> for TopFatalError {
    fn from_error(err: FromUtf8Error) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
impl<T> FromError<SendError<T>> for TopFatalError {
    fn from_error(err: SendError<T>) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}

fn handle_client(stream: TcpStream, client_id: ClientID, rx_from_mux: Receiver<MuxMessage>, tx_to_mux: SyncSender<ClientMessage>) -> Result<(), TopFatalError> {

    println!("Handling client");
    
    let mut ws = try!(WebSocketStream::new(stream));
    
    println!("Opened ws");

    try!(ws.send(format!("Hello, Client").as_bytes()));
    
    let mut ws_send = try!(ws.try_clone());
    let mut ws_recv = try!(ws.try_clone());

    thread::spawn(move||{
        for mux_msg in rx_from_mux.iter() {
            match mux_msg {
                MuxMessage::Chat(msg) => {
                    match ws_send.send(&msg.into_bytes()[..]) {
                        Ok(_) => {},
                        Err(e) => panic!("Failed sending to ws")
                    }
                }
            }
        }
    });

    loop {
        let msg = try!(ws_recv.recv());
        try!(tx_to_mux.send(ClientMessage::Chat(client_id, try!(String::from_utf8(msg)))));
    }
    
}

type ClientID = i64;

enum ClientMessage { Chat(ClientID, String), Disconnect(ClientID) }

enum MuxMessage { Chat(String) }

fn run() -> Result<(), Error> {

    let listener = try!(TcpListener::bind("0.0.0.0:9000"));

    let mut clients: HashMap<ClientID, SyncSender<MuxMessage>> = HashMap::new();

    let (tx_to_mux, rx_from_clients) = sync_channel(0);

    thread::spawn(move||{
        for client_msg in rx_from_clients.iter() {
            match client_msg {
                ClientMessage::Chat(client_id, s) => {
                    println!("Got message from {}, {:?}", client_id, s);
                }
                ClientMessage::Disconnect(client_id) => {}
            }
        }
    });

    // Handle incoming clients
    for stream in listener.incoming() {
        
        match stream {
            Err(e) => { 
                println!("Accept error: {:?}", e); 
            }
            Ok(stream) => {
                let client_id: ClientID = thread_rng().gen();
                let (tx_to_client, rx_from_mux) = sync_channel(0);
                clients.insert(client_id, tx_to_client);
                let local_tx_to_mux = tx_to_mux.clone();
                thread::spawn(move|| {
                    match handle_client(stream, client_id, rx_from_mux, local_tx_to_mux) {
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