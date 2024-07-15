use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_login::AuthSession;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::requirement::Requirement},
    usecase::util::auth_backend::AuthBackend,
};

use super::util::{authorize_against_project_id, requ_db_to_api};

pub fn router() -> axum::Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route(
            "/requirements/:requirement_id",
            get(get_requirement_info)
                .patch(patch_requirement)
                .delete(delete_requirement),
        )
        .route(
            "/requirements",
            get(get_requirements_for_project).post(create_requirement_for_project),
        )
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetRequirementsForProjectResponse {
    pub requirements: Vec<Requirement>,
}

pub async fn get_requirements_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref project_repo = state.project_repo;
    let ref requ_repo = state.requ_repo;
    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let requ_ids = match project_repo.query_requ_by_project_id(&project_id).await {
        Ok(requs) => requs,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let requs: Vec<_> = requ_ids
        .into_iter()
        .map(|requ_id| async move { requ_repo.query_requ_by_id(&requ_id).await })
        .collect();
    let requs = match try_join_all(requs).await {
        Ok(requs) => requs,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    // let requs:<Vec<Requirement>> = requs.into_iter().map(requ_db_to_api).collect();
    Json(GetRequirementsForProjectResponse {
        requirements: requs.into_iter().map(|r| requ_db_to_api(r)).collect(),
    })
    .into_response()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetRequirementInfoResponse {
    #[serde(flatten)]
    pub requirement: Requirement,
}

pub async fn get_requirement_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((project_id, requirement_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref project_repo = state.project_repo;
    let ref requ_repo = state.requ_repo;
    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let requ = match requ_repo.query_requ_by_id(&requirement_id).await {
        Ok(requ) => requ,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(GetRequirementInfoResponse {
        requirement: requ_db_to_api(requ),
    })
    .into_response()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateRequirementForProjectRequest {
    pub name: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateRequirementForProjectResponse {
    #[serde(flatten)]
    pub requirement: Requirement,
}

pub async fn create_requirement_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateRequirementForProjectRequest>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref project_repo = state.project_repo;
    let ref requ_repo = state.requ_repo;
    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let returned_requ = requ_repo
        .insert_requ_for_project(&project_id, req.name, req.content)
        .await;
    let requ = match returned_requ {
        Ok(requ) => requ,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(requ_db_to_api(requ))).into_response()
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PatchRequirementRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PatchRequirementResponse {
    #[serde(flatten)]
    pub requirement: Requirement,
}

pub async fn patch_requirement(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((project_id, requirement_id)): Path<(String, String)>,
    Json(req): Json<PatchRequirementRequest>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref project_repo = state.project_repo;
    let ref requ_repo = state.requ_repo;
    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let mut requ = match requ_repo.query_requ_by_id(&requirement_id).await {
        Ok(requ) => requ,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let (Some(name), Some(content)) = (req.name, req.content) {
        requ.name = name;
        requ.description = content;
    }

    let returned_requ = requ_repo.update_requ(&requirement_id, &requ).await;
    let requ = match returned_requ {
        Ok(requ) => requ,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(requ_db_to_api(requ))).into_response()
}

pub async fn delete_requirement(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((project_id, requirement_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    let ref project_repo = state.project_repo;
    let ref requ_repo = state.requ_repo;
    if let Some(value) = authorize_against_project_id(auth_session, project_repo, &project_id).await
    {
        return value;
    }

    let requ = match requ_repo.delete_requ_from_project(&requirement_id).await {
        Ok(requ) => requ,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    (StatusCode::OK, Json(requ_db_to_api(requ))).into_response()
}
