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
    api::{
        app::AppState,
        model::{
            task::{TaskRelation, TaskRelationType},
            util::Id,
        },
    },
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetLinksForTaskResponse {
    task_links: Vec<TaskRelation>,
}

pub async fn get_links_for_task(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskLinkForUserRequest {
    from: Id,
    to: Id,
    #[serde(flatten)]
    category: TaskRelationType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskLinkForUserResponse {
    #[serde(flatten)]
    relation: TaskRelation,
}

pub async fn create_task_link_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateTaskLinkForUserRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskLinkForProjectRequest {
    from: Id,
    to: Id,
    #[serde(flatten)]
    category: TaskRelationType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskLinkForProjectResponse {
    #[serde(flatten)]
    relation: TaskRelation,
}

pub async fn create_task_link_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateTaskLinkForProjectRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskLinksForProjectResponse {
    links: Vec<TaskRelation>,
}

pub async fn get_task_links_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskLinksForUserResponse {
    links: Vec<TaskRelation>,
}

pub async fn get_task_links_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

pub async fn delete_task_link(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}
