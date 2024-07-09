use std::io;

use crate::db::{db_context::DbContext, model::user::User};

#[derive(Clone, Debug)]
pub struct UserRepository {
    context: DbContext,
}

impl UserRepository {
    pub async fn new() -> UserRepository {
        UserRepository {
            context: DbContext::new().await,
        }
    }

    pub async fn query_user_by_name(&self, name: &str) -> Result<User, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!("SELECT * FROM user WHERE username == '{}'", name))
            .await
            .unwrap();

        let user: Option<User> = response.take(0).unwrap();

        user.ok_or(io::Error::new(io::ErrorKind::NotFound, "User not found"))
    }

    pub async fn query_user_by_id(&self, id: &str) -> Result<User, io::Error> {
        let user: Option<User> = self
            .context
            .db
            .select(("user", id))
            .await
            .unwrap_or_else(|_| None);
        user.ok_or(io::Error::new(io::ErrorKind::NotFound, "User not found"))
    }

    pub async fn insert_user(&self, user: &User) -> Result<User, io::Error> {
        let result: Option<User> = self
            .context
            .db
            .create("user")
            .content(user)
            .await
            .unwrap()
            .pop();
        result.ok_or(io::Error::new(io::ErrorKind::NotFound, "User insert fail"))
    }

    pub async fn update_user(&self, user_id:&str, user: &User) -> Result<User, io::Error> {
        let result: Option<User> = self
            .context
            .db
            .update(("user", user_id))
            .content(user)
            .await
            .unwrap();
        result.ok_or(io::Error::new(io::ErrorKind::NotFound, "User update fail"))
    }
}
