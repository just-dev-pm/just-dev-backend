use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusPool {
    pub incomplete: Vec<IndexedStatusContent>,
    pub complete: StatusContent,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusContent {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexedStatusContent {
    pub id: String,
    pub status: StatusContent,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "category")]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Incomplete { id: String },
    Complete,
}
