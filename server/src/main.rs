use std::net::{TcpListener, TcpStream};
use std::thread;

use std::io::{Error, Read, Write};

use waim::User;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
}
