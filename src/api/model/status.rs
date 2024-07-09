use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Status {
    pub pool: StatusPool,
    pub status_item: ActualStatusItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusPool {
    pub incomplete: Vec<IndexedStatusItem>,
    pub complete: StatusItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusItem {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexedStatusItem {
    pub id: String,
    pub status: StatusItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "category")]
#[serde(rename_all = "snake_case")]
pub enum ActualStatusItem {
    Incomplete { id: String },
    Complete,
}
