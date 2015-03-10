
// Unstable libraries
#![feature(collections, core, io, net, unicode)]

#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate                      crypto;
extern crate                      mio;
extern crate "rustc-serialize" as serialize;

pub mod websocket;

#[test]
fn it_works() {
}
