use std::sync::Arc;

use crate::db::model::user::{Credentials, User};
use crate::db::repository::user::UserRepository;
use axum::async_trait;
use axum_login::{AuthnBackend, AuthzBackend, UserId};

#[derive(Clone, Debug)]
pub struct AuthBackend {
    user_repo: Arc<UserRepository>,
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials { username, password }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match self.user_repo.query_user_by_name(&username).await {
            Some(user) => {
                if user.password == password {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        match self.user_repo.query_user_by_id(user_id).await {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl AuthzBackend for AuthBackend {
    type Permission = ();
}

impl AuthBackend {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        AuthBackend { user_repo }
    }
}
