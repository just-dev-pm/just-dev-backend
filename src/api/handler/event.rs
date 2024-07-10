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
        model::{agenda::Event, util::Id},
    },
    usecase::util::auth_backend::AuthBackend,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateEventForAgendaRequest {
    pub name: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub participants: Vec<Id>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateEventForAgendaResponse {
    #[serde(flatten)]
    pub event: Event,
}

pub async fn create_event_for_agenda(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(agenda_id): Path<String>,
    Json(req): Json<CreateEventForAgendaRequest>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetEventsForAgendaResponse {
    pub events: Vec<Event>,
}

pub async fn get_events_for_agenda(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(agenda_id): Path<String>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchEventRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<Id>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchEventResponse {
    #[serde(flatten)]
    pub event: Event,
}

pub async fn patch_event(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((agenda_id, event_id)): Path<(String, String)>,
    Json(req): Json<PatchEventRequest>,
) -> impl IntoResponse {
    todo!()
}

pub async fn delete_event(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((agenda_id, event_id)): Path<(String, String)>,
) -> impl IntoResponse {
    todo!()
}
