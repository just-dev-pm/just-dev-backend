use axum_ycrdt_websocket::{
    broadcast::BroadcastGroup,
    ws::{AxumSink, AxumStream},
};
use futures::{future::try_join_all, StreamExt};
use std::{io, sync::Arc};
use yrs::{
    updates::encoder::{Encoder, EncoderV1},
    ReadTxn, Transact,
};

use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::draft::Draft},
    db::{model::draft::DraftPayload, repository::draft::DraftRepository},
    usecase::{draft_collaboration::DraftCollaborationManager, util::auth_backend::AuthBackend},
};

use super::util::{authorize_against_project_id, authorize_against_user_id, draft_db_to_api};

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
    // TODO authorize draft against user
    let state = state.lock().await;
    let db_draft = state.draft_repo.query_draft_by_id(&draft_id).await;

    let db_draft = match db_draft {
        Ok(draft) => draft,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let draft = draft_db_to_api(db_draft);

    let draft = match draft {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(draft) => draft,
    };

    (StatusCode::OK, Json(GetDraftInfoResponse { draft })).into_response()
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
    // TODO authorize draft against user
    let state = state.lock().await;
    let db_draft = state.draft_repo.query_draft_by_id(&draft_id).await;

    let db_draft = match db_draft {
        Ok(draft) => draft,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match req.name {
        None => {
            let draft = draft_db_to_api(db_draft);

            let draft = match draft {
                None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                Some(draft) => draft,
            };

            (StatusCode::OK, Json(PatchDraftInfoResponse { draft })).into_response()
        }

        Some(name) => {
            let new_draft = DraftPayload { name, ..db_draft };
            let returned_db_draft = state.draft_repo.update_draft(new_draft).await;
            match returned_db_draft {
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                Ok(db_draft) => {
                    let api_draft = draft_db_to_api(db_draft);
                    match api_draft {
                        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                        Some(draft) => {
                            (StatusCode::OK, Json(PatchDraftInfoResponse { draft })).into_response()
                        }
                    }
                }
            }
        }
    }
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
    let state = state.lock().await;

    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let db_drafts_id = state.user_repo.query_draft_by_id(&user_id).await;

    let db_drafts_id = match db_drafts_id {
        Ok(drafts) => drafts,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let db_drafts_futures: Vec<_> = db_drafts_id
        .into_iter()
        .map(|id| {
            let draft_repo = &state.draft_repo;
            async move { draft_repo.query_draft_by_id(&id).await }
        })
        .collect();

    let db_drafts = try_join_all(db_drafts_futures).await;

    let db_drafts = match db_drafts {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(drafts) => drafts,
    };

    let api_drafts: Option<Vec<_>> = db_drafts
        .into_iter()
        .map(|draft| draft_db_to_api(draft))
        .collect();

    let api_drafts = match api_drafts {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(drafts) => drafts,
    };

    (
        StatusCode::OK,
        Json(GetDraftsForUserResponse { drafts: api_drafts }),
    )
        .into_response()
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
    let state = state.lock().await;
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let returned_draft_payload = state
        .draft_repo
        .insert_draft_for_user(&req.name, &user_id)
        .await;

    match returned_draft_payload {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(draft_payload) => {
            let api_draft = draft_db_to_api(draft_payload);
            match api_draft {
                None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                Some(draft) => {
                    (StatusCode::OK, Json(CreateDraftForUserResponse { draft })).into_response()
                }
            }
        }
    }
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
    let state = state.lock().await;

    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return value;
    }

    let db_drafts_id = state.project_repo.query_draft_by_id(&project_id).await;

    let db_drafts_id = match db_drafts_id {
        Ok(drafts) => drafts,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let db_drafts_futures: Vec<_> = db_drafts_id
        .into_iter()
        .map(|id| {
            let draft_repo = &state.draft_repo;
            async move { draft_repo.query_draft_by_id(&id).await }
        })
        .collect();

    let db_drafts = try_join_all(db_drafts_futures).await;

    let db_drafts = match db_drafts {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(drafts) => drafts,
    };

    let api_drafts: Option<Vec<_>> = db_drafts
        .into_iter()
        .map(|draft| draft_db_to_api(draft))
        .collect();

    let api_drafts = match api_drafts {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(drafts) => drafts,
    };

    (
        StatusCode::OK,
        Json(GetDraftsForProjectResponse { drafts: api_drafts }),
    )
        .into_response()
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
    let state = state.lock().await;
    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return value;
    }

    let returned_draft_payload = state
        .draft_repo
        .insert_draft_for_project(&req.name, &project_id)
        .await;

    match returned_draft_payload {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(draft_payload) => {
            let api_draft = draft_db_to_api(draft_payload);
            match api_draft {
                None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                Some(draft) => {
                    (StatusCode::OK, Json(CreateDraftForUserResponse { draft })).into_response()
                }
            }
        }
    }
}

pub async fn draft_ws_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession<AuthBackend>,
    Path(draft_id): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;

    let draft_collaboration_manager = state.draft_collaboration_manager.clone();
    let draft_repo = state.draft_repo.clone();

    // TODO Authorize aginst draft
    ws.on_upgrade(move |socket| {
        handle_socket(socket, draft_id, draft_collaboration_manager, draft_repo)
    })
}

async fn handle_socket(
    socket: WebSocket,
    draft_id: String,
    draft_collaboration_manager: Arc<Mutex<DraftCollaborationManager>>,
    draft_repo: DraftRepository,
) {
    let (sender, receiver) = socket.split();
    let sender = Arc::new(Mutex::new(AxumSink::from(sender)));
    let receiver = AxumStream::from(receiver);

    let bcast: Arc<BroadcastGroup> = draft_collaboration_manager
        .lock()
        .await
        .get_room(draft_id.clone(), &draft_repo)
        .await
        .unwrap();

    let sub = bcast.subscribe(sender, receiver);

    // TODO; fix future is not send error
    match sub.completed().await {
        Ok(_) => {
            let old_draft = draft_repo.query_draft_by_id(&draft_id).await;
            match old_draft {
                Err(_) => (),
                Ok(old_draft) => {
                    let awareness = bcast.awareness().read().await;
                    let doc = awareness.doc();
                    let txn = doc.transact();
                    let rev = txn.snapshot();
                    let mut encoder = EncoderV1::new();
                    txn.encode_state_from_snapshot(&rev, &mut encoder).unwrap();
                    let update = encoder.to_vec();
                    let new_draft = DraftPayload {
                        content: update,
                        ..old_draft
                    };

                    tokio::spawn(async move {
                        draft_repo.update_draft(new_draft).await;
                    });
                }
            }
        }
        Err(_) => (),
    }
}
