use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::task::TaskList},
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListInfoResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn get_task_list_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListsForProjectResponse {
    task_lists: Vec<TaskList>,
}

pub async fn get_task_lists_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListsForUserResponse {
    task_lists: Vec<TaskList>,
}

pub async fn get_task_lists_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(uer_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateTaskListForProjectRequest {
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateTaskListForProjectResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn create_task_list_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateTaskListForProjectRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskListForUserRequest {
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskListForUserResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn create_task_list_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(uer_id): Path<String>,
    Json(req): Json<CreateTaskListForUserRequest>,
) -> impl IntoResponse {
    todo!()
}

pub async fn delete_task_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}
