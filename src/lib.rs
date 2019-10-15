use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::ops::Add;
use std::vec::Vec;

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
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
}

/// Create a JSON string from request field and write to TCP stream
pub fn send_req(
    writer: &mut BufWriter<TcpStream>,
    req_type: ReqType,
    user: &User,
    message: &str,
) {
    let request = Request {
        req_type,
        user: user.clone(),
        message: message.clone().to_string(),
    };

    let s = serde_json::to_string(&request)
        .expect("Error converting request to JSON string")
        .add("\n");
    eprintln!("{}", s);

    writer
        .write(s.as_bytes())
        .expect("Unable to write to TCP stream");
    writer.flush().expect("Error while writing to TCP stream");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
