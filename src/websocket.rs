
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use mio::net::tcp::{TcpStream};
use serialize::base64::{Config, Newline, Standard, ToBase64};
use std::collections::HashMap;
use std::error::FromError;
use std::io::{BufRead, BufStream, Error, Write};

macro_rules! some { ($e:expr,$none:expr) => (match $e { Some(e) => e, None => return $none } )}
macro_rules! tryWith { ($e:expr,$none:expr) => (match $e { Ok(x) => x, Err(_) => return $none } )}
//macro_rules! genericError { ($msg:expr) => (Err(IoError { kind: OtherIoError, desc: $msg, detail: None }))}

static WS_MAGIC_GUID : &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/*
pub struct WebSocket {
    stream: TcpStream
}
*/

pub enum MioWsError { Io(Error), Parse(String), Handshake(String) }

impl FromError<Error> for MioWsError {
    fn from_error(err: Error) -> MioWsError { MioWsError::Io(err) }
}

struct HttpHeader {
    name: String,
    value: String
}

peg! http_grammar (r#"
use websocket::HttpHeader;

name -> String
    = [^:]+ { match_str.to_string() }

value -> String
    = .+ { match_str.to_string() }

#[pub]
header -> HttpHeader
    = n:name ": " v:value { HttpHeader{ name: n, value: v } }
"#);

fn parse_header(line: &String) -> Result<HttpHeader, MioWsError> {
    match http_grammar::header(&line.trim()[..]) {
        Err(err) => Err(MioWsError::Handshake(err)),
        Ok(header) => Ok(header)
    }
}

pub fn perform_handshake(stream: TcpStream) -> Result<(), MioWsError> {

    let mut buf_stream = BufStream::new(stream);
    
    let mut line = String::new();
    
    try!(buf_stream.read_line(&mut line));
    
    let mut headers = HashMap::new();
    
    loop {
        try!(buf_stream.read_line(&mut line));
        if line.trim().len() == 0 { break }
        let header = try!(parse_header(&line));
        headers.insert(header.name, header.value);
    }
    
    let key = String::from_str("Sec-WebSocket-Key");
    
    let challenge_response: String = match headers.get(&key) {
        None => return Err(MioWsError::Handshake(String::from_str("No WebSocket Challenge Header"))),
        Some(challenge) => {
            let mut hasher = Sha1::new();
            let mut local = challenge.clone();
            local.push_str(WS_MAGIC_GUID);
            println!("Using {}", local);
            hasher.input(local.as_bytes());
            let mut output = [0;20];
            hasher.result(&mut output);
            output.to_base64(Config {
                char_set: Standard,
                line_length: None,
                newline: Newline::CRLF,
                pad: true,
            })
        }
    };
    
    println!("Using {}", challenge_response);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", challenge_response);
    
    try!(buf_stream.write(response.as_bytes()));

    Ok(())

}

/*

impl WebSocket {

    pub fn new(stream: TcpStream) -> IoResult<WebSocket> {
        try!(perform_handshake(stream.clone()));
        Ok(WebSocket { stream: stream.clone() })
    }
    
    pub fn send(&mut self, msg: &[u8]) -> IoResult<()> {

        try!(self.stream.write([129]));
            
        let len: uint = msg.len();
        let l1: u8 = some!(FromPrimitive::from_uint(len >> 24), genericError!("Conversion error"));
        let l2: u8 = some!(FromPrimitive::from_uint(len >> 16), genericError!("Conversion error"));
        let l3: u8 = some!(FromPrimitive::from_uint(len >>  8), genericError!("Conversion error"));
        let l4: u8 = some!(FromPrimitive::from_uint(len      ), genericError!("Conversion error"));
        
        try!(match len {
            _ if len <= 125 => 
                self.stream.write(&[l4]),
            _ if len > 125 && len <= 65535 => 
                self.stream.write(&[126u8, l3, l4]),
            _ => 
                // HMM, looks like really 8 bytes are required
                self.stream.write(&[127u8, l1, l2, l3, l4])
        });

        return self.stream.write(msg)
    }

    pub fn recv(&mut self) -> IoResult<String> {
        
        let _text_type = try!(self.stream.read_byte());
        
        let len1 = 0x7F & try!(self.stream.read_byte());
        
        let length: uint = match len1 {
            _ if len1 <= 125 =>
                some!(FromPrimitive::from_u8(len1), genericError!("Conversion error")),
            _ if len1 == 126 => {
                    let mut l: [u8;2] = [0;2];
                    try!(self.stream.read(l));
                    let high: uint = some!(FromPrimitive::from_u8(l[0]), genericError!("Conversion error"));
                    let low: uint = some!(FromPrimitive::from_u8(l[1]), genericError!("Conversion error"));
                    (high << 8) | low
                }
            _ =>
                return genericError!("TODO message length > 65535")
        };
        
        println!("Receiving message with {} bytes", length);
        
        let mut mask: [u8;4] = [0;4];
        try!(self.stream.read(mask));
        
        let mut data: Vec<u8> = try!(self.stream.read_exact(length));
        
        for i in range(0, length) {
            *data.get_mut(i) = data[i] ^ mask[i % 4];
        }
        
        let text = tryWith!(String::from_utf8(data), genericError!("Invalid unicode"));
        
        Ok(text)
        
    }
}

*/
