use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use octocrate_webhooks::WebhookPullRequestClosed;
use tokio::sync::Mutex;

use crate::{api::app::AppState, db::repository::utils::unwrap_thing};

pub async fn filter_github_webhook_requests(
    header: HeaderMap,
    req: Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    if let Some(event) = header.get("X-GitHub-Event") {
        if event != "pull_request" {
            return StatusCode::BAD_REQUEST.into_response();
        }
    } else {
        return StatusCode::BAD_REQUEST.into_response();
    }
    next.run(req).await
}

pub async fn handle_pull_request_event(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Ok(req) = serde_json::from_value::<WebhookPullRequestClosed>(value) {
        let state = state.lock().await;
        let tasks = match state.task_repo.query_task_by_pr_number(req.number).await {
            Ok(tasks) => tasks,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
        };

        for mut task in tasks {
            task.complete = true;
            if let Err(err) = state
                .task_repo
                .update_task_by_id(&unwrap_thing(task.id.clone().unwrap()), &task)
                .await
            {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        }

        StatusCode::OK.into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}
