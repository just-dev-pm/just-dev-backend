use std::{io, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use futures::{future::try_join_all, stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{agenda::Agenda, util::Id},
    },
    db::repository::utils::unwrap_thing,
    usecase::util::auth_backend::AuthBackend,
};

use super::util::authorize_against_user_id;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAgendasForUserResponse {
    pub agendas: Vec<Agenda>,
}

pub async fn get_agendas_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }
    let ref state = state.lock().await;
    let ref user_repo = state.user_repo;
    let ref agenda_repo = state.agenda_repo;
    let returned_agendas = user_repo.query_agenda_by_id(&user_id).await;

    let agendas = match returned_agendas {
        Ok(agendas) => agendas
            .into_iter()
            .map(|id| async move { agenda_repo.query_agenda_by_id(id.as_str()).await })
            .collect::<Vec<_>>(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let agendas = match try_join_all(agendas).await {
        Ok(agendas) => agendas
            .into_iter()
            .map(|agenda| {
                let agenda_id = unwrap_thing(agenda.id.clone().unwrap());
                async move {
                    let events = agenda_repo.query_event_id_by_agenda_id(&agenda_id).await?;
                    Ok::<Agenda, io::Error>(Agenda {
                        id: agenda_id,
                        name: agenda.name,
                        events: events.into_iter().map(|event| Id { id: event }).collect(),
                    })
                }
            })
            .collect::<Vec<_>>(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let agendas = match try_join_all(agendas).await {
        Ok(agendas) => agendas,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    (StatusCode::OK, Json(GetAgendasForUserResponse { agendas })).into_response()
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
