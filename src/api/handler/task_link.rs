use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
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
    db::{model::task::TaskLink, repository::utils::unwrap_thing},
    usecase::{task_stream::refresh_task_status, util::auth_backend::AuthBackend},
};

use super::util::{
    authorize_against_project_id, authorize_against_task_id, authorize_against_task_link,
    authorize_against_task_link_id, authorize_against_user_id, task_link_db_to_api,
    task_relation_category_to_kind,
};

pub fn project_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new().route(
        "/links",
        post(create_task_link_for_project).get(get_task_links_for_project),
    )
}

pub fn user_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new().route(
        "/links",
        post(create_task_link_for_user).get(get_task_links_for_user),
    )
}

pub fn router() -> Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route("/:link_id", delete(delete_task_link).patch(patch_task_link))
        .route("/tasks/:task_id", get(get_links_for_task))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetLinksForTaskResponse {
    task_links: Vec<TaskRelation>,
}

pub async fn get_links_for_task(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) = authorize_against_task_id(
        &auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_id,
    )
    .await
    {
        return value;
    }

    let task_links = match state.task_repo.query_task_links_by_task_id(&task_id).await {
        Ok(_links) => _links,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    (
        StatusCode::OK,
        Json(GetLinksForTaskResponse {
            task_links: task_links
                .into_iter()
                .filter_map(|link| match task_link_db_to_api(link) {
                    Ok(link) => Some(link),
                    Err(_) => None,
                })
                .collect(),
        }),
    )
        .into_response()
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
    let ref state = state.lock().await;
    if let Some(value) = authorize_against_user_id(auth_session.to_owned(), &user_id) {
        return value;
    }
    if let Some(value) = authorize_against_task_link(
        &auth_session,
        &state.project_repo,
        &state.task_repo,
        &req.from.id,
        &req.to.id,
    )
    .await
    {
        return value;
    }

    let task_link_result = state
        .task_repo
        .insert_task_link(
            &req.from.id,
            &req.to.id,
            task_relation_category_to_kind(&req.category),
        )
        .await;
    let task_link = match task_link_result {
        Ok(_link) => _link,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    if let Err(err) = refresh_task_status(&req.to.id, &state.task_repo).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
    }

    (
        StatusCode::OK,
        Json(CreateTaskLinkForUserResponse {
            relation: task_link_db_to_api(task_link).unwrap(),
        }),
    )
        .into_response()
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

//TODO: check if the tasks in project
pub async fn create_task_link_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateTaskLinkForProjectRequest>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    if let Some(value) =
        authorize_against_project_id(auth_session.to_owned(), &state.project_repo, &project_id)
            .await
    {
        return value;
    }
    if let Some(value) = authorize_against_task_link(
        &auth_session,
        &state.project_repo,
        &state.task_repo,
        &req.from.id,
        &req.to.id,
    )
    .await
    {
        return value;
    }

    let task_link_result = state
        .task_repo
        .insert_task_link(
            &req.from.id,
            &req.to.id,
            task_relation_category_to_kind(&req.category),
        )
        .await;
    let task_link = match task_link_result {
        Ok(_link) => _link,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    if let Err(err) = refresh_task_status(&req.to.id, &state.task_repo).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
    }

    (
        StatusCode::OK,
        Json(CreateTaskLinkForProjectResponse {
            relation: task_link_db_to_api(task_link).unwrap(),
        }),
    )
        .into_response()
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
    let state = state.lock().await;
    if let Some(value) = authorize_against_task_link_id(
        &auth_session,
        &state.project_repo,
        &state.task_repo,
        &link_id,
    )
    .await
    {
        return value;
    }

    match state.task_repo.delete_task_link_by_id(&link_id).await {
        Ok(_link) => {
            if let Err(err) =
                refresh_task_status(&unwrap_thing(_link.outgoing.unwrap()), &state.task_repo).await
            {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
            }
            StatusCode::OK.into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskLinkRequest {
    #[serde(flatten)]
    pub category: TaskRelationType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskLinkResponse {
    #[serde(flatten)]
    pub task_relation: TaskRelation,
}

pub async fn patch_task_link(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(link_id): Path<String>,
    Json(req): Json<PatchTaskLinkRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) = authorize_against_task_link_id(
        &auth_session,
        &state.project_repo,
        &state.task_repo,
        &link_id,
    )
    .await
    {
        return value;
    }

    let link = match state.task_repo.query_task_link_by_id(&link_id).await {
        Ok(_link) => _link,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    let response = match state
        .task_repo
        .update_task_link(
            &link_id,
            &TaskLink {
                id: None,
                kind: task_relation_category_to_kind(&req.category).to_owned(),
                ..link.to_owned()
            },
        )
        .await
    {
        Ok(_link) => (
            StatusCode::OK,
            Json(PatchTaskLinkResponse {
                task_relation: task_link_db_to_api(_link).unwrap(),
            }),
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    };

    if let Err(err) =
        refresh_task_status(&unwrap_thing(link.outgoing.unwrap()), &state.task_repo).await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
    }

    response
}
