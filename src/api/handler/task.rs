use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{status::Status, task::Task, util::Id},
    },
    usecase::util::auth_backend::AuthBackend,
};

use super::user::PatchUserInfoRequest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTasksForList {
    pub tasks: Vec<Task>,
}

pub async fn get_tasks_for_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskForListRequest {
    pub name: String,
    pub description: String,
    pub assignees: Vec<Id>,
    pub status: Status,
    pub deadline: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskForListResponse {
    #[serde(flatten)]
    pub task: Task,
}

pub async fn create_task_for_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
    Json(req): Json<CreateTaskForListRequest>,
) -> impl IntoResponse {
    todo!()
}

pub async fn delete_task_from_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((task_list_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<Id>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskResponse {
    #[serde(flatten)]
    pub task: Task,
}

pub async fn patch_task(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((task_list_id, task_id)): Path<(String, String)>,
    Json(req): Json<PatchTaskRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAssignedTasksReponse {
    pub tasks: Vec<Task>,
}

pub async fn get_assigned_tasks_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((task_list_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    todo!()
}
