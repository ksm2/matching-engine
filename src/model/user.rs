use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct User {
    user_id: String,
}

impl User {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}
