use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Requirement {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
}

impl Requirement {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: None,
            name,
            description,
        }
    } 
}