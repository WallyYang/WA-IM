use std::io::{self, BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::str;
use std::thread;

use client::Server;
use waim::*;

/// prompt user for username and password, return a User struct
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

/// send a user to server for registration, return true if no duplicate username
fn register(stream: &mut TcpStream, user: User) -> bool {
    send_req(stream, ReqType::Register, &user, &String::new());

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let result = str::from_utf8(&buffer).unwrap().trim();

    result == "True"
}

/// send a user to server for validation,
/// return true if username and password matches
fn validate(stream: &mut TcpStream, user: User) -> bool {
    send_req(stream, ReqType::Validate, &user, &String::new());

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let result = str::from_utf8(&buffer).unwrap().trim();

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
    eprintln!("Success!\n");

    loop {
        // spawn a thread to listen for incoming messages
        let c_stream = stream.try_clone().unwrap();
        thread::spawn(move || loop {
            if let Some(request) = recv_req(&c_stream) {
                match request.req_type {
                    ReqType::Message => println!(
                        "{}: {}",
                        request.user.username, request.message
                    ),
                    ReqType::List => {
                        let users: Vec<String> =
                            serde_json::from_str(&request.message).unwrap();
                        print!("Online users: ");
                        for user in users {
                            print!("{},", user);
                        }
                        io::stdout().flush().unwrap();
                    }
                    _ => (),
                }
            }
        });

        print!("> ");
        io::stdout().flush().unwrap();

        let mut msg = String::new();
        io::stdin().read_line(&mut msg).unwrap();
        let msg = msg.trim();

        if msg.len() > 0 {
            if msg == ":l" {
                send_req(&mut stream, ReqType::List, &user, &String::new());
            } else {
                send_req(&mut stream, ReqType::Message, &user, msg);
            }
        }
    }
}
