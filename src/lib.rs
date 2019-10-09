use std::vec::Vec;

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
