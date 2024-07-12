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

use super::util::{authorize_against_agenda_id, authorize_against_event_id, event_db_to_api};

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
    let ref state = state.lock().await;
    let ref user_repo = state.user_repo;
    let ref agenda_repo = state.agenda_repo;

    if let Some(value) =
        authorize_against_event_id(&auth_session, agenda_repo, user_repo, &agenda_id, &event_id)
            .await
    {
        return value;
    }
    let event_ref = match agenda_repo.query_event_by_id(&event_id).await {
        Ok(event) => event,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
    };

    let event = crate::db::model::agenda::Event {
        id: event_ref.id,
        name: req.name.unwrap_or(event_ref.name),
        description: req.description.unwrap_or(event_ref.description),
        start_time: req.start_time.unwrap_or(event_ref.start_time.0).into(),
        end_time: req.end_time.unwrap_or(event_ref.end_time.0).into(),
    };

    let event = agenda_repo.update_event(&event_id, &event).await;
    if let Err(err) = event {
        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
    }

    let assignees_ref = match agenda_repo.query_assignees_of_event(&event_id).await {
        Ok(assignees) => assignees,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
    };

    if let Some(assignees) = req.participants.clone() {
        let assignees: Vec<_> = assignees.into_iter().map(|a| a.id).collect();
        for assignee in &assignees {
            if !assignees_ref.contains(assignee) {
                match agenda_repo.assign_event_for_user(&event_id, assignee).await {
                    Ok(_) => {}
                    Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
                }
            }
        }
        for assignee in assignees_ref {
            if !assignees.contains(&assignee) {
                match agenda_repo.deassign_event_for_user(&event_id, &assignee).await {
                    Ok(_) => {}
                    Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
                }
            }
        }
    }  
    (
        StatusCode::OK,
        Json(PatchEventResponse {
            event: event_db_to_api(event.unwrap(), req.participants.unwrap_or_default()),
        }),
    )
        .into_response()
    
}

pub async fn delete_event(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((agenda_id, event_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref user_repo = state.user_repo;
    let ref agenda_repo = state.agenda_repo;

    if let Some(value) =
        authorize_against_event_id(&auth_session, agenda_repo, user_repo, &agenda_id, &event_id)
            .await
    {
        return value;
    }

    match agenda_repo.delete_event(&event_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
    }
}
