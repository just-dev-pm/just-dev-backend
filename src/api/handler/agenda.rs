use std::{io, sync::Arc};

use axum::{
    extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, patch, post}, Json, Router
};
use axum_login::AuthSession;
use futures::future::try_join_all;
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

use super::{event::{create_event_for_agenda, delete_event, get_events_for_agenda, patch_event}, util::{
        agenda_db_to_api, authorize_against_agenda_id, authorize_against_project_id,
        authorize_against_user_id,
    }};

pub fn user_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new().route(
        "/agendas",
        get(get_agendas_for_user).post(create_agenda_for_user),
    )
}

pub fn project_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new().route(
        "/agendas",
        get(get_agendas_for_project).post(create_agenda_for_project),
    )
}

pub fn router() -> Router<Arc<Mutex<AppState>>> {
    let router = Router::new().route(
        "/events/:event_id",
        patch(patch_event).delete(delete_event),
    )
    .route(
        "/events",
        post(create_event_for_agenda).get(get_events_for_agenda),
    )
    .route(
        "/:agenda_id",
        get(get_agenda_info).delete(delete_agenda),
    );
    Router::new().nest("/:agenda_id", router)
}

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
    let ref state = state.lock().await;
    let ref agenda_repo = state.agenda_repo;
    let ref project_repo = state.project_repo;
    if let Some(value) =
        authorize_against_project_id(auth_session, &project_repo, &project_id).await
    {
        return value;
    }

    let returned_agendas = project_repo.query_agenda_by_id(&project_id).await;

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
pub struct GetAgendaInfoResponse {
    #[serde(flatten)]
    pub agenda: Agenda,
}

pub async fn get_agenda_info(
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

    let returned_agenda = agenda_repo.query_agenda_by_id(&agenda_id).await;

    let agenda = match returned_agenda {
        Ok(agenda) => {
            let events = agenda_repo.query_event_id_by_agenda_id(&agenda_id).await;
            if let Err(msg) = events {
                return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response();
            }
            Ok::<Agenda, io::Error>(Agenda {
                id: agenda_id,
                name: agenda.name,
                events: events
                    .unwrap()
                    .into_iter()
                    .map(|event| Id { id: event })
                    .collect(),
            })
        }
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
    };

    let agenda = match agenda {
        Ok(agenda) => agenda,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    (StatusCode::OK, Json(GetAgendaInfoResponse { agenda })).into_response()
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
    let ref state = state.lock().await;
    let ref agenda_repo = state.agenda_repo;

    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    };

    let returned_agenda = agenda_repo
        .insert_agenda_for_user(&user_id, &req.name)
        .await;
    
    match returned_agenda {
        Ok(agenda) => (
            StatusCode::OK,
            Json(CreateAgendaForProjectResponse {
                agenda: agenda_db_to_api(agenda, None),
            }),
        )
            .into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
    let ref state = state.lock().await;
    let ref agenda_repo = state.agenda_repo;
    let ref project_repo = state.project_repo;

    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let returned_agenda = agenda_repo
        .insert_agenda_for_project(&project_id, &req.name)
        .await;
    match returned_agenda {
        Ok(agenda) => (
            StatusCode::OK,
            Json(CreateAgendaForProjectResponse {
                agenda: agenda_db_to_api(agenda, None),
            }),
        )
            .into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_agenda(
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

    match agenda_repo.delete_agenda(&agenda_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
    }

}
