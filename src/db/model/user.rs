
use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use super::status::StatusPool;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Option<Thing>,
    pub username: String,
    pub avatar: String,
    pub email: String,
    pub password: String,
    pub status_pool: Option<Thing>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.as_ref().and_then(|thing| Some(thing.id.to_string())).unwrap_or_else(|| "not exist".to_string())
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

impl User {}
