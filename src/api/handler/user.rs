use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::{AuthSession, AuthUser};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::user::User},
    usecase::util::auth_backend::AuthBackend,
};

use super::util::user_db_to_api;

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
