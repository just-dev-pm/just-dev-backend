use axum_login::AuthUser;
use serde::{Deserialize, Serialize};



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}


impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

impl User {

}
