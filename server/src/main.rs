use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::vec::Vec;

extern crate serde;

use waim::*;

struct Session {
    users: Vec<User>,
    messages: Vec<Message>,
    active_conns: HashMap<User, TcpStream>,
}

impl Session {
    /// create a session and read user info from file
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

    /// register a user to the list, write False to stream if duplicate
    fn register(
        &mut self,
        writer: &mut BufWriter<TcpStream>,
        user: User,
    ) -> bool {
        eprintln!("Registering");
        if self.users.contains(&user) {
            writer.write("False\n".as_bytes()).unwrap();
            writer.flush().unwrap();
            return false;
        } else {
            writer.write("True\n".as_bytes()).unwrap();
            writer.flush().unwrap();

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

    /// validate the user with list, write True if username and password match
    fn validate(&self, writer: &mut BufWriter<TcpStream>, user: User) -> bool {
        eprintln!("Validating");
        if self.users.contains(&user) {
            writer.write("True\n".as_bytes()).unwrap();
            writer.flush().unwrap();
            return true;
        } else {
            writer.write("False\n".as_bytes()).unwrap();
            writer.flush().unwrap();
            return false;
        }
    }

    /// add a message to list of messages
    fn recv_msg(&mut self, user: &User, content: &String) {
        eprintln!("Received Message");
        self.messages.push(Message {
            user: user.clone(),
            content: content.clone(),
        });
        eprintln!("{}", serde_json::to_string_pretty(&self.messages).unwrap());

        for active_conn in &mut self.active_conns {
            if active_conn.0 != user {
                send_req(active_conn.1, ReqType::Message, user, &content);
            }
        }
    }

    fn list(&mut self, user: &User) {
        eprintln!("List online users");

        let mut users: Vec<String> = Vec::new();
        for active_conn in &mut self.active_conns {
            users.push(active_conn.0.username.clone());
        }

        send_req(
            &mut self.active_conns.get_mut(&user).unwrap(),
            ReqType::List,
            &user,
            &serde_json::to_string(&users).unwrap(),
        );
    }
}

fn handle_client(session: Arc<Mutex<Session>>, stream: TcpStream) {
    println!("Incoming connection from: {}", stream.peer_addr().unwrap());

    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    let user: User;
    loop {
        // register or validate user
        if let Some(request) = recv_req(&stream) {
            eprintln!("Matching");
            let result = match request.req_type {
                ReqType::Register => session
                    .lock()
                    .unwrap()
                    .register(&mut writer, request.user.clone()),
                ReqType::Validate => session
                    .lock()
                    .unwrap()
                    .validate(&mut writer, request.user.clone()),
                _ => false,
            };

            if result {
                eprintln!("Validation success");
                (*session)
                    .lock()
                    .unwrap()
                    .active_conns
                    .insert(request.user.clone(), stream.try_clone().unwrap());
                user = request.user;
                break;
            }
        } else {
            eprintln!("Empty request");
        }
        eprintln!("Validation failed");
    }

    loop {
        // received messages
        if let Some(request) = recv_req(&stream) {
            match request.req_type {
                ReqType::Message => {
                    session.lock().unwrap().recv_msg(&user, &request.message)
                }
                ReqType::List => session.lock().unwrap().list(&user),
                _ => (),
            }
        // if request.req_type == ReqType::Message {
        //     session.lock().unwrap().recv_msg(&user, request.message);
        // } else {
        //     panic!("Error, expected message from clients");
        // }
        } else {
            break;
        }
    }
}

fn main() {
    let session = Arc::new(Mutex::new(Session::create()));

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
