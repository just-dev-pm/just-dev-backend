use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use octocrate_webhooks::{WebhookPullRequestClosed, WebhookPullRequestClosedPullRequest};
use tokio::sync::Mutex;

use crate::{api::{app::AppState, model::pr::PullRequest}, db::repository::utils::unwrap_thing};

use super::task::IoErrorWrapper;

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
) -> Result<impl IntoResponse, IoErrorWrapper> {
    if let Ok(req) = serde_json::from_value::<WebhookPullRequestClosed>(value) {
        if let WebhookPullRequestClosedPullRequest::PullRequest(_pr) = req.pull_request {
            if !_pr.merged {
                return Ok(StatusCode::OK.into_response());
            }
        } else {
            return Ok(StatusCode::OK.into_response());
        };
        let state = state.lock().await;
        let tasks = state.task_repo.query_task_by_pr_number(req.number).await?;
        
        for mut task in tasks {
            if task.pr.repo != req.repository.name {
                continue;
            }
            task.complete = true;
            let _ = state
                .task_repo
                .update_task_by_id(&unwrap_thing(task.id.clone().unwrap()), &task)
                .await?;
        }

        Ok(StatusCode::OK.into_response())
    } else {
        Ok(StatusCode::BAD_REQUEST.into_response())
    }
}
