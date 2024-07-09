use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api::app::AppState, usecase::util::auth_backend::AuthBackend};

use super::util::authorize_against_user_id;

#[derive(Serialize, Deserialize)]
pub struct GetProjectsForUserResponse {
    pub projects: Vec<ProjectUser>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectUser {
    pub id: String,
    #[serde(flatten)]
    pub position: UserPermissionInProject,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "position")]
#[serde(rename_all = "snake_case")]
pub enum UserPermissionInProject {
    Admin,
    Member,
}

pub async fn get_projects_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let state = state.lock().await;

    let projects = state
        .user_repo
        .query_project_join_by_id(&user_id)
        .await
        .unwrap();

    let admin_projects: Option<Vec<_>> = projects
        .0
        .iter()
        .map(|project| {
            project.id.clone().map(|id| ProjectUser {
                id: id.id.to_string(),
                position: UserPermissionInProject::Admin,
            })
        })
        .collect();

    let member_projects: Option<Vec<_>> = projects
        .1
        .iter()
        .map(|project| {
            project.id.clone().map(|id| ProjectUser {
                id: id.id.to_string(),
                position: UserPermissionInProject::Member,
            })
        })
        .collect();

    let admin_projects = match admin_projects {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(admin_projects) => admin_projects,
    };

    let member_projects = match member_projects {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(member_projects) => member_projects,
    };

    (
        StatusCode::OK,
        Json(GetProjectsForUserResponse {
            projects: admin_projects
                .into_iter()
                .chain(member_projects.into_iter())
                .collect(),
        }),
    )
        .into_response()
}
