use tracing::log::info;
use crate::db::{db_context::DbContext, model::user::User};

#[derive(Clone, Debug)]
pub struct UserRepository {
    context: DbContext
}

impl UserRepository {
    pub async fn new() -> UserRepository {
        UserRepository {
            context: DbContext::new().await
        }
    }

    pub async fn query_user_by_name(&self, name: &str) -> Option<User> {
        self.context.db.select(("user", name)).await.unwrap_or_else(|_| None)

    }
    
    pub async fn query_user_by_id(&self, id: &str) -> Option<User> {
        self.context.db.select(("user:id", id)).await.unwrap_or_else(|_| None)
    } 
}
