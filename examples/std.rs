
extern crate                         rand;
extern crate "websocket-protocol" as ws;

use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::error::FromError;
use std::io::Error;
//use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::string::FromUtf8Error;
// use std::string::String;
use std::sync::mpsc::{Receiver, SendError, SyncSender, sync_channel};
use std::thread;

use ws::protocol::{TryClone, WsError, WebSocketStream};

#[derive(Debug)]
enum TopFatalError { Fatal(String) }

// Macro this
impl FromError<Error> for TopFatalError {
    fn from_error(err: Error) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err.kind())) }
}
impl FromError<WsError> for TopFatalError {
    fn from_error(err: WsError) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
impl FromError<FromUtf8Error> for TopFatalError {
    fn from_error(err: FromUtf8Error) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}
impl FromError<SendError<ClientMessage>> for TopFatalError {
    fn from_error(err: SendError<ClientMessage>) -> TopFatalError { TopFatalError::Fatal(format!("{:?}", err)) }
}

fn handle_client(stream: TcpStream, client_id: ClientID, rx_from_mux: Receiver<MuxMessage>, tx_to_mux: SyncSender<ClientMessage>) -> Result<(), TopFatalError> {

    println!("Handling client");
    
    let ws = try!(WebSocketStream::new(stream));
    
    println!("Opened ws");

    let mut ws_send = try!(ws.try_clone());
    let mut ws_recv = ws;

    println!("Cloned both ws");

    try!(ws_send.send(format!("Hello, Client").as_bytes()));

    thread::spawn(move||{
        println!("Listening on broadcast");
        for mux_msg in rx_from_mux.iter() {
            match mux_msg {
                MuxMessage::Chat(msg) => {
                    match ws_send.send(&msg.into_bytes()[..]) {
                        Ok(_) => {},
                        Err(e) => panic!(format!("Failed sending to ws: {:?}", e))
                    }
                }
            }
        }
    });

    loop {
        println!("Listening on ws");
        let msg = try!(ws_recv.recv());
        try!(tx_to_mux.send(ClientMessage::Chat(client_id, try!(String::from_utf8(msg)))));
    }
    
}

type ClientID = i64;

enum ClientMessage { Connect(ClientID, SyncSender<MuxMessage>), Chat(ClientID, String), Disconnect(ClientID) }

#[derive(Clone)]
enum MuxMessage { Chat(String) }

fn run() -> Result<(), TopFatalError> {

    let listener = try!(TcpListener::bind("0.0.0.0:9000"));


    let (tx_to_mux, rx_from_clients) = sync_channel(0);

    thread::spawn(move||{

        let mut clients: HashMap<ClientID, SyncSender<MuxMessage>> = HashMap::new();

        for client_msg in rx_from_clients.iter() {
            match client_msg {
                ClientMessage::Connect(client_id, tx_to_client) => {
                    println!("Inserting client {}", client_id);
                    clients.insert(client_id, tx_to_client);
                }
                ClientMessage::Chat(client_id, s) => {
                    let broadcast_msg: String = format!("Got message from {}: {:?}", client_id, s);
                    println!("{}", broadcast_msg);
                    let broadcast_msg = MuxMessage::Chat(broadcast_msg);
                    
                    for (broadcast_client_id, tx_to_client) in clients.iter() {
                        println!("Broadcasting to {}", broadcast_client_id);
                        tx_to_client.send(broadcast_msg.clone());
                    }                                        
                }
                ClientMessage::Disconnect(client_id) => {
                    println!("Removing client {}", client_id);
                    clients.remove(&client_id);
                }
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
                try!(tx_to_mux.send(ClientMessage::Connect(client_id, tx_to_client)));
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