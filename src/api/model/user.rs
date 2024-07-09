use serde::{Deserialize, Serialize};

use super::status::StatusPool;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_pool: Option<StatusPool>,
}
