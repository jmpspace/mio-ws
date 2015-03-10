
// Unstable libraries
#![feature(collections, core, io, unicode)]

#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate                      crypto;
extern crate "rustc-serialize" as serialize;

pub mod websocket;

#[test]
fn it_works() {
}
