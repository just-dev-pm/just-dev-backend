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
    api::{app::AppState, model::draft::Draft},
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDraftInfoResponse {
    #[serde(flatten)]
    pub draft: Draft,
}

pub async fn get_draft_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(draft_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchDraftInfoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchDraftInfoResponse {
    #[serde(flatten)]
    pub draft: Draft,
}

pub async fn patch_draft_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(draft_id): Path<String>,
    Json(req): Json<PatchDraftInfoRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDraftsForUserResponse {
    drafts: Vec<Draft>,
}

pub async fn get_drafts_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateDraftForUserRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateDraftForUserResponse {
    #[serde(flatten)]
    pub draft: Draft,
}

pub async fn create_draft_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateDraftForUserRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDraftsForProjectResponse {
    drafts: Vec<Draft>,
}

pub async fn get_drafts_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateDraftForProjectRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateDraftForProjectResponse {
    #[serde(flatten)]
    pub draft: Draft,
}

pub async fn create_draft_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateDraftForProjectRequest>,
) -> impl IntoResponse {
    todo!()
}
