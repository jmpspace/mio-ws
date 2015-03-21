# websocket-protocol

I'm implementing just the websocket protocol. It is stack agnostic - you need to provide a stream which can read, write, and be cloned.

## Non goals

* SSL - use a reverse-proxy instead

## Example Platforms

Rust std::net::TcpStream

TODO - https://github.com/carllerche/mio#platforms
