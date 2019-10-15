use std::collections::BTreeSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;
use std::vec::Vec;

extern crate serde;

use waim::Message;
use waim::ReqType;
use waim::Request;
use waim::User;

struct Session {
    users: Vec<User>,
    // conns: BTreeSet<TcpStream>,
    messages: Vec<Message>,
}

impl Session {
    fn create() -> Session {
        let users = Path::new("users.json");

        let mut file = match File::open(&users) {
            Ok(file) => file,
            Err(e) => {
                panic!("Could not open users info file: {}", e.description())
            }
        };

        let users: Vec<User>;
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => {
                users = serde_json::from_str(&s).unwrap_or_else(|e| {
                    panic!("Error in parsing users.json: {}", e.description())
                })
            }
            Err(e) => {
                panic!("Unable to read from users.json: {}", e.description())
            }
        };

        Session {
            users: users,
            // conns: BTreeSet::new(),
            messages: Vec::new(),
        }
    }

    fn handle_client(&mut self, stream: &mut TcpStream) {
        println!("Incoming connection from: {}", stream.peer_addr().unwrap());

        let mut buffer: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(stream);
        reader.read_until(b'\n', &mut buffer).unwrap();

        let s = str::from_utf8(&buffer).unwrap();
        let request: Request = serde_json::from_str(&s).unwrap();

        let stream = reader.into_inner();

        match request.req_type {
            ReqType::Register => self.register(stream, request.user),
            ReqType::Validate => self.validate(stream, request.user),
            _ => (),
        }
    }

    fn register(&mut self, stream: &mut TcpStream, user: User) {
        eprintln!("Registering");
        if self.users.contains(&user) {
            stream.write("False".as_bytes()).unwrap();
        } else {
            stream.write("True".as_bytes()).unwrap();
            let mut file = File::create("users.json")
                .expect("Unable to open users.json for writing");
            file.write(
                serde_json::to_string_pretty(&self.users)
                    .unwrap()
                    .as_bytes(),
            )
            .expect("Failed to write to user file");
        }
    }

    fn validate(&self, stream: &mut TcpStream, user: User) {
        eprintln!("Validating");
        if self.users.contains(&user) {
            stream.write("True".as_bytes()).unwrap();
        } else {
            stream.write("False".as_bytes()).unwrap();
        }
    }
}

fn main() {
    let mut session = Session::create();
    println!("{:?}", session.users);

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(mut stream) => session.handle_client(&mut stream),
        }
    }
}
