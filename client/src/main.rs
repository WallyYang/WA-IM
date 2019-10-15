use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::str;

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
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    send_req(&mut writer, ReqType::Register, &user, &String::new());

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let result = str::from_utf8(&buffer).unwrap().trim();
    println!("{}", result);

    result == "True"
}

/// send a user to server for validation,
/// return true if username and password matches
fn validate(stream: &mut TcpStream, user: User) -> bool {
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    send_req(&mut writer, ReqType::Validate, &user, &String::new());

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

    let mut writer = BufWriter::new(stream);
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut msg = String::new();
        io::stdin().read_line(&mut msg).unwrap();

        send_req(&mut writer, ReqType::Message, &user, msg.trim());
    }
}
