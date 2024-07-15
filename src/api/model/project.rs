use serde::{Deserialize, Serialize};

use super::status::StatusPool;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_pool: Option<StatusPool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<i64>,
}



