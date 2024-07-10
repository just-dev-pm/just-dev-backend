use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::notification::Notification},
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetNotificationsResponse {
    notifications: Vec<Notification>,
}

pub async fn get_notifications(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HandleNotificationResponse {
    #[serde(flatten)]
    notification: Notification,
}

pub async fn handle_notification(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((user_id, notification_id)): Path<(String, String)>,
) -> impl IntoResponse {
    todo!()
}
