use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Requirement {
    pub id: String,
    pub name: String,
    pub content: String,
}
