use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Status {
    pub name: String,
    pub description: String,
    pub number: String,
}

#[derive(Deserialize, Clone, Serialize, Debug, Default)]
pub struct StatusPool {
    pub incomplete: Vec<Status>,
    pub complete: Status,
}

impl Status {
    pub fn new() -> Self {
        Status {
            name: "complete".to_string(),
            description: "description".to_owned(),
            number: String::new(),
        }
    }
}

impl StatusPool {
    pub fn new() -> Self {
        StatusPool {
            incomplete: vec![Status::new()],
            complete: Status::new(),
        }
    }
}
