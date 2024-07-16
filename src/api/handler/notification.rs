use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use axum_login::AuthSession;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::notification::Notification},
    db::repository::utils::unwrap_thing,
    usecase::{notification::query_notif_by_id, util::auth_backend::AuthBackend},
};

use super::{
    task::IoErrorWrapper,
    util::{authorize_against_user_id, notif_db_to_api},
};

pub fn user_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route(
            "/notifications/:notification_id",
            patch(handle_notification),
        )
        .route("/notifications", get(get_notifications))
}

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

    let ref state = state.lock().await;
    let notifs = notif_ids
        .into_iter()
        .map(|id| async move {
            query_notif_by_id(
                &state.notif_repo,
                &state.task_repo,
                &state.agenda_repo,
                id.as_str(),
            )
            .await
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
) -> Result<impl IntoResponse, IoErrorWrapper> {
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return Ok(value);
    }

    let ref state = state.lock().await;
    let ref notif_repo = state.notif_repo;

    let notif = notif_repo.handle_notif_by_id(&notification_id).await?;

    let (notif, source) = query_notif_by_id(
        &state.notif_repo,
        &state.task_repo,
        &state.agenda_repo,
        &unwrap_thing(notif.id.unwrap()),
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(HandleNotificationResponse {
            notification: notif_db_to_api(notif, source),
        }),
    )
        .into_response())
}
