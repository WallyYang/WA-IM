use std::vec::Vec;

use std::error::Error;
use std::fs::File;
use std::path::Path;

use std::net::{TcpListener, TcpStream};
// use std::thread;

use std::io;
use std::io::{Read, Write};

extern crate serde;

use waim::Message;
use waim::User;

struct Session {
    users: Vec<User>,
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
            messages: Vec::new(),
        }
    }
}

fn handle_client(mut stream: TcpStream) -> Result<(), io::Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            return Ok(());
        }
        stream.write(&buf[..bytes_read])?;
    }
}

fn main() {
    let session = Session::create();
    println!("{:?}", session.users);

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(stream) => {
                handle_client(stream).unwrap_or_else(|e| eprintln!("{:?}", e));
            }
        }
    }
}
