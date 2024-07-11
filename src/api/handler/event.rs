use std::sync::Arc;

use crate::api::model::agenda::Event as ApiEvent;
use crate::db::model::agenda::Event as DbEvent;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth;
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{agenda::Event, util::Id},
    },
    usecase::util::auth_backend::AuthBackend,
};

use super::util::{authorize_against_agenda_id, event_db_to_api};

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
    let ref state = state.lock().await;
    let ref agenda_repo = state.agenda_repo;
    let ref user_repo = state.user_repo;
    if let Some(value) = authorize_against_agenda_id(&auth_session, user_repo, &agenda_id).await {
        return value;
    }
    match agenda_repo
        .insert_event_for_agenda(&DbEvent::from_create_request(req), &agenda_id)
        .await
    {
        Ok(event) => (
            StatusCode::OK,
            Json(CreateEventForAgendaResponse {
                event: event_db_to_api(event, vec![]),
            }),
        )
            .into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
    let ref state = state.lock().await;
    let ref agenda_repo = state.agenda_repo;
    let ref user_repo = state.user_repo;
    if let Some(value) = authorize_against_agenda_id(&auth_session, user_repo, &agenda_id).await {
        return value;
    }
    match agenda_repo.query_events_by_agenda_id(&agenda_id).await {
        Ok(events) => (
            StatusCode::OK,
            Json(GetEventsForAgendaResponse {
                events: events
                    .into_iter()
                    .map(|event| event_db_to_api(event, vec![]))
                    .collect(),
            }),
        )
            .into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
