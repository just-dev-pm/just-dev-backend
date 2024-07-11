use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{asset::Asset, notification::Notification},
    },
    db::{model::notification::NotificationSource, repository::utils::unwrap_thing},
    usecase::util::auth_backend::AuthBackend,
};

use super::util::{authorize_against_user_id, notif_db_to_api};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetNotificationsResponse {
    notifications: Vec<Notification>,
}

pub async fn get_notifications(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let returned_notif_ids = state
        .lock()
        .await
        .user_repo
        .query_notif_by_user_id(&user_id)
        .await;
    let notif_ids = match returned_notif_ids {
        Ok(ids) => ids,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let state = state.lock().await;
    let notifs = notif_ids
        .into_iter()
        .map(|id| {
            let notif_repo = &state.notif_repo;
            async move { notif_repo.query_notif_by_id(id.as_str()).await }
        })
        .collect::<Vec<_>>();

    let notifs = match try_join_all(notifs).await {
        Ok(notifs) => notifs,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    (
        StatusCode::OK,
        Json(GetNotificationsResponse {
            notifications: notifs
                .into_iter()
                .map(|(notif, source)| notif_db_to_api(notif, source))
                .collect(),
        }),
    )
        .into_response()
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
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let ref notif_repo = state.lock().await.notif_repo;

    let returned_notif = notif_repo.handle_notif_by_id(&notification_id).await;

    let notif = match returned_notif {
        Ok(notif) => notif,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let returned_notif = notif_repo
        .query_notif_by_id(&unwrap_thing(notif.id.unwrap()))
        .await;
    let (notif, source) = match returned_notif {
        Ok(value) => value,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    (
        StatusCode::OK,
        Json(HandleNotificationResponse {
            notification: notif_db_to_api(notif, source),
        }),
    )
        .into_response()
}
