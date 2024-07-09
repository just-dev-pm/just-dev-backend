use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_login::AuthSession;
use tokio::sync::Mutex;

use crate::{api::app::AppState, usecase::util::auth_backend::AuthBackend};

pub async fn get_projects_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
}
