use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{status::InvalidStatusCode, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_login::{AuthSession, AuthUser};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{status::StatusPool, user::User},
    },
    usecase::util::auth_backend::AuthBackend,
};

use super::util::{user_api_to_db, user_db_to_api};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetUserInfoResponse {
    #[serde(flatten)]
    user: User,
}

pub async fn get_user_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match auth_session.user {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(user) => match user.id().eq(&user_id) {
            true => (),
            false => return StatusCode::UNAUTHORIZED.into_response(),
        },
    };

    let state = state.lock().await;
    let db_user = state.user_repo.query_user_by_id(&user_id).await;

    let api_user = match db_user {
        Ok(db_user) => user_db_to_api(db_user),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match api_user {
        None => StatusCode::NO_CONTENT.into_response(),
        Some(user) => (StatusCode::OK, Json(GetUserInfoResponse { user })).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PatchUserInfoRequest {
    pub username: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub status_pool: Option<StatusPool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchUserInfoResponse {
    #[serde(flatten)]
    pub user: User,
}

pub async fn patch_user_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
    Json(req): Json<PatchUserInfoRequest>,
) -> impl IntoResponse {
    match auth_session.user {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(user) => match user.id().eq(&user_id) {
            true => (),
            false => return StatusCode::UNAUTHORIZED.into_response(),
        },
    };

    let user = state
        .lock()
        .await
        .user_repo
        .query_user_by_id(&user_id)
        .await;

    let user = match user {
        Ok(user) => user,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let password = user.clone().password;

    let user = user_db_to_api(user);

    let user = match user {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(user) => user,
    };

    let user = User {
        id: user.id,
        username: user.username,
        email: req.email.or(user.email),
        avatar: req.avatar.or(user.avatar),
        status_pool: req.status_pool.or(user.status_pool),
    };

    let returned_user = state
        .lock()
        .await
        .user_repo
        .update_user(&user.id.clone(), &user_api_to_db(user, &password))
        .await;

    let user = match returned_user {
        Ok(user) => user,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let user = user_db_to_api(user);

    let user = match user {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(user) => user,
    };

    (StatusCode::OK, Json(PatchUserInfoResponse { user })).into_response()
}
