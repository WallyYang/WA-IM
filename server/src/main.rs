use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::vec::Vec;

extern crate serde;

use waim::Message;
use waim::ReqType;
use waim::Request;
use waim::User;

struct Session {
    users: Vec<User>,
    messages: Vec<Message>,
    active_conns: HashMap<User, TcpStream>,
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
            active_conns: HashMap::new(),
        }
    }

    fn register(&mut self, stream: &mut TcpStream, user: User) -> bool {
        eprintln!("Registering");
        if self.users.contains(&user) {
            stream.write("False".as_bytes()).unwrap();
            return false;
        } else {
            stream.write("True".as_bytes()).unwrap();

            self.users.push(user);

            let mut file = File::create("users.json")
                .expect("Unable to open users.json for writing");
            file.write(
                serde_json::to_string_pretty(&self.users)
                    .unwrap()
                    .as_bytes(),
            )
            .expect("Failed to write to user file");
            return true;
        }
    }

    fn validate(&self, stream: &mut TcpStream, user: User) -> bool {
        eprintln!("Validating");
        if self.users.contains(&user) {
            stream.write("True\n".as_bytes()).unwrap();
            return true;
        } else {
            stream.write("False\n".as_bytes()).unwrap();
            return false;
        }
    }

    fn recv_msg(&mut self, user: User, content: String) {
        eprintln!("Received Message");
        eprintln!("{:?}", self.messages);
        self.messages.push(Message { user, content });
    }
}

fn handle_client(session: Arc<Mutex<Session>>, stream: TcpStream) {
    println!("Incoming connection from: {}", stream.peer_addr().unwrap());

    let mut buffer: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(stream);
    reader.read_until(b'\n', &mut buffer).unwrap();

    let s = str::from_utf8(&buffer).unwrap();
    let request: Request = serde_json::from_str(&s).unwrap();

    let mut stream = reader.into_inner();

    let result = match request.req_type {
        ReqType::Register => (*session)
            .lock()
            .unwrap()
            .register(&mut stream, request.user.clone()),
        ReqType::Validate => (*session)
            .lock()
            .unwrap()
            .validate(&mut stream, request.user.clone()),
        ReqType::Message => false,
    };

    if result {
        (*session)
            .lock()
            .unwrap()
            .active_conns
            .insert(request.user.clone(), stream.try_clone().unwrap());
    }
}

fn main() {
    let mut session = Arc::new(Mutex::new(Session::create()));

    println!("{:?}", (*session).lock().unwrap().users);

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(stream) => {
                let session_ref = Arc::clone(&session);
                thread::spawn(move || {
                    handle_client(session_ref, stream);
                });
            }
        }
    }

    println!("exit");
}
