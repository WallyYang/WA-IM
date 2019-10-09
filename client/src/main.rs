use std::io::{self, BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::str;

use client::Server;

fn main() {
    let server = Server::from_file("config.yaml");

    println!("server name = {}", server.name);
    println!("server address = {}", server.address);

    let socket_addr = SocketAddr::new(server.address, server.port);

    let mut stream = TcpStream::connect(socket_addr)
        .expect("Could not connect to the server");
}
