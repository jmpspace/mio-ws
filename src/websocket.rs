
use crypto::digest::Digest;
use crypto::sha1::Sha1;
//use mio::net::tcp::{TcpStream};
use serialize::base64::{Config, Newline, Standard, ToBase64};
use std::collections::HashMap;
use std::error::FromError;
use std::io::{BufRead, BufStream, Error, Read, Write};
use std::num::FromPrimitive;
//use std::option::Option;

static WS_MAGIC_GUID : &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WebSocketStream<S> {
    stream: BufStream<S>
}

pub enum WsError { Bitwise, Io(Error), Parse(String), Handshake(String), Protocol(String) }

impl FromError<Error> for WsError {
    fn from_error(err: Error) -> WsError { WsError::Io(err) }
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

fn parse_header(line: &String) -> Result<HttpHeader, WsError> {
    match http_grammar::header(&line.trim()[..]) {
        Err(err) => Err(WsError::Handshake(err)),
        Ok(header) => Ok(header)
    }
}

impl <S: Read + Write> WebSocketStream<S> {

    pub fn new(stream: S) -> Result<WebSocketStream<S>, WsError> {

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
            None => return Err(WsError::Handshake(String::from_str("No WebSocket Challenge Header"))),
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

        Ok(WebSocketStream{stream: buf_stream})

    }

    pub fn send(&mut self, msg: &[u8]) -> Result<(), WsError> {

        try!(self.stream.write(&[129]));

        let len: usize = msg.len();
        let l1: u8 = try!(FromPrimitive::from_usize(len >> 24).ok_or(WsError::Bitwise));
        let l2: u8 = try!(FromPrimitive::from_usize(len >> 16).ok_or(WsError::Bitwise));
        let l3: u8 = try!(FromPrimitive::from_usize(len >>  8).ok_or(WsError::Bitwise));
        let l4: u8 = try!(FromPrimitive::from_usize(len      ).ok_or(WsError::Bitwise));

        try!(match len {
            _ if len <= 125 => 
            self.stream.write(&[l4]),
            _ if len > 125 && len <= 65535 => 
            self.stream.write(&[126u8, l3, l4]),
            _ => 
            // HMM, looks like really 8 bytes are required
            self.stream.write(&[127u8, l1, l2, l3, l4])
        });

        try!(self.stream.write(&msg));

        Ok(())
    }

    /*
       pub fn recv(&mut self) -> IoResult<String> {

       let _text_type = try!(self.stream.read_byte());

       let len1 = 0x7F & try!(self.stream.read_byte());

       let length: uint = match len1 {
       _ if len1 <= 125 =>
       try!(FromPrimitive::from_u8(len1)),
       _ if len1 == 126 => {
       let mut l: [u8;2] = [0;2];
       try!(self.stream.read(l));
       let high: uint = try!(FromPrimitive::from_u8(l[0]));
       let low: uint = try!(FromPrimitive::from_u8(l[1]));
       (high << 8) | low
       }
       _ =>
       return Err(WsError::Protocol(String::from_str("Message too big, not implemented")))
       };

       println!("Receiving message with {} bytes", length);

       let mut mask: [u8;4] = [0;4];
       try!(self.stream.read(mask));

       let mut data: Vec<u8> = try!(self.stream.read_exact(length));

       for i in range(0, length) {
     *data.get_mut(i) = data[i] ^ mask[i % 4];
     }

     let text = try!(String::from_utf8(data));

     Ok(text)

     }
     */
}

