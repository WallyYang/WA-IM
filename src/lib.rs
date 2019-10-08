pub struct User {
    pub username: String,
    pub password: String,
}

pub struct Message {
    pub user: User,
    pub content: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
