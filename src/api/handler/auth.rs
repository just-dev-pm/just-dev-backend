use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::user::{Credential, User},
    },
    db::model::user::Credentials,
    usecase::{user::insert_user, util::auth_backend::AuthBackend},
};

use super::util::{credential_api_to_user_db, user_db_to_api};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LoginRequest {
    #[serde(flatten)]
    pub credential: Credential,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LoginResponse {
    #[serde(flatten)]
    pub user: User,
}

pub async fn login(
    mut auth_session: AuthSession<AuthBackend>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let creds = Credentials {
        username: req.credential.username,
        password: req.credential.password,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let api_user = user_db_to_api(user);
    match api_user {
        Some(user) => (StatusCode::OK, Json(LoginResponse { user })).into_response(),
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SignupRequest {
    #[serde(flatten)]
    pub credential: Credential,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SignupResponse {
    #[serde(flatten)]
    pub user: User,
}

pub async fn signup(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    let db_user = match credential_api_to_user_db(req.credential) {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(user) => {
            insert_user(
                &state.lock().await.user_repo,
                &state.lock().await.task_repo,
                &user,
            )
            .await
        }
    };

    let return_api_user = match db_user {
        Ok(user) => user_db_to_api(user),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match return_api_user {
        Some(user) => (StatusCode::OK, Json(SignupResponse { user })).into_response(),
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn logout(mut auth_session: AuthSession<AuthBackend>) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => (StatusCode::OK, "log out successfully").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
