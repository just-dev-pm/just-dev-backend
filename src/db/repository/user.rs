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

    pub async fn query_user_by_name(&self, name: &str) -> Option<User> {
        let mut response = self
            .context
            .db
            .query(format!("SELECT * FROM user WHERE username == '{}'", name))
            .await
            .unwrap();

        let user: Option<User> = response.take(0).unwrap();

        user
    }

    pub async fn query_user_by_id(&self, id: &str) -> Option<User> {
        self.context
            .db
            .select(("user", id))
            .await
            .unwrap_or_else(|_| None)
    }
}
