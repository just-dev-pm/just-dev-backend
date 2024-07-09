use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "source")]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    Task { id: String },
    Draft { id: String },
    Event { id: String },
}
