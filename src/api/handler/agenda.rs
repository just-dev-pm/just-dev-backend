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
        model::{agenda::Agenda, util::Id},
    },
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAgendasForUserResponse {
    pub agendas: Vec<Agenda>,
}

pub async fn get_agendas_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAgendasForProjectResponse {
    pub agendas: Vec<Agenda>,
}

pub async fn get_agendas_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAgendaInfoResponse {
    #[serde(flatten)]
    pub agenda: Agenda,
}

pub async fn get_agenda_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(agenda_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateAgendaForUserRequest {
    name: String,
    events: Vec<Id>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateAgendaForUserResponse {
    #[serde(flatten)]
    pub agenda: Agenda,
}

pub async fn create_agenda_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateAgendaForUserRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateAgendaForProjectRequest {
    name: String,
    events: Vec<Id>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateAgendaForProjectResponse {
    #[serde(flatten)]
    pub agenda: Agenda,
}

pub async fn create_agenda_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateAgendaForProjectRequest>,
) -> impl IntoResponse {
    todo!()
}

pub async fn delete_agenda(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(agenda_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}
