use std::io::{self, BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::ops::Add;
use std::str;

use client::Server;
use waim::ReqType;
use waim::Request;
use waim::User;

fn prompt_for_user() -> User {
    print!("Enter your username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("Enter your password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();

    User {
        username: username.trim().to_string(),
        password: password.trim().to_string(),
    }
}

fn register(stream: &mut TcpStream, user: User) -> bool {
    let request = Request {
        req_type: ReqType::Register,
        user: user,
        message: String::new(),
    };

    let s = serde_json::to_string(&request).unwrap().add("\n");

    stream.write(s.as_bytes()).unwrap();

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let result = str::from_utf8(&buffer).unwrap().trim();
    println!("{}", result);

    result == "True"
}

fn validate(stream: &mut TcpStream, user: User) -> bool {
    let request = Request {
        req_type: ReqType::Validate,
        user: user,
        message: String::new(),
    };

    let s = serde_json::to_string(&request).unwrap().add("\n");

    stream.write(s.as_bytes()).unwrap();

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let result = str::from_utf8(&buffer).unwrap().trim();
    println!("{}", result);

    result == "True"
}

fn main() {
    let server = Server::from_file("config.yaml");

    println!("server name = {}", server.name);
    println!("server address = {}", server.address);

    let socket_addr = SocketAddr::new(server.address, server.port);

    let mut stream = TcpStream::connect(socket_addr)
        .expect("Could not connect to the server");

    print!("Register for a new user?[y/n] ");
    io::stdout().flush().unwrap();
    let mut reg = String::new();
    io::stdin().read_line(&mut reg).unwrap();
    let reg = reg.trim();

    let mut user = prompt_for_user();

    if reg == "y" {
        eprintln!("Register");
        while !register(&mut stream, user.clone()) {
            println!("Error, username exists");
            user = prompt_for_user();
        }
    } else {
        eprintln!("Validate");
        while !validate(&mut stream, user.clone()) {
            println!("Error, invalid username or password");
            user = prompt_for_user();
        }
    }

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut msg = String::new();
        io::stdin().read_line(&mut msg).unwrap();

        let request = Request {
            req_type: ReqType::Message,
            user: user.clone(),
            message: msg.trim().to_string(),
        };

        stream
            .write(
                serde_json::to_string(&request)
                    .unwrap()
                    .add("\n")
                    .as_bytes(),
            )
            .unwrap();
    }
}
