use std::vec::Vec;

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl std::cmp::PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        return self.username == other.username
            && self.password == other.password;
    }
}

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

#[derive(Debug, Serialize, Deserialize)]
pub enum ReqType {
    Register,
    Validate,
    Message,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
