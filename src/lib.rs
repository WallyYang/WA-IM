use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::vec::Vec;

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub user: User,
    pub content: String,
}

pub fn serialize_users(users: Vec<User>) -> String {
    serde_json::to_string(&users)
        .unwrap_or_else(|_| panic!("Error while serializing users"))
}

pub fn deserialize_users(s: &str) -> Vec<User> {
    let users: Vec<User> = serde_json::from_str(s)
        .unwrap_or_else(|_| panic!("Error while deserializing users"));

    users
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub req_type: ReqType,
    pub user: User,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ReqType {
    Register,
    Validate,
    Message,
    List,
}

/// Create a JSON string from request field and write to TCP stream
pub fn send_req(
    stream: &mut TcpStream,
    req_type: ReqType,
    user: &User,
    message: &str,
) {
    let request = Request {
        req_type,
        user: user.clone(),
        message: message.clone().to_string(),
    };

    let mut s = serde_json::to_string(&request)
        .expect("Error converting request to JSON string");

    s.push(0xAu8 as char);

    let mut writer =
        BufWriter::new(stream.try_clone().expect("Unable to clone TCP stream"));

    writer
        .write(s.as_bytes())
        .expect("Unable to write to TCP stream");
    writer.flush().expect("Error while writing to TCP stream");
}

/// Retrieve a request from TCP stream, return None if no request available
pub fn recv_req(stream: &TcpStream) -> Option<Request> {
    let mut reader =
        BufReader::new(stream.try_clone().expect("Unable to clone TCP stream"));

    let mut buffer = String::new();
    reader
        .read_line(&mut buffer)
        .expect("Unable to read from TCP stream");
    if buffer.len() > 0 {
        serde_json::from_str(&buffer).expect("Error parsing incoming request")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
