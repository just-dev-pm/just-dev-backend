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

use crate::{
    api::{
        app::AppState,
        model::{project::Project, status::StatusPool, user::User},
    },
    usecase::util::auth_backend::AuthBackend,
};

use super::util::{
    authorize_against_project_id, authorize_against_user_id, project_api_to_db, project_db_to_api,
    user_db_to_api,
};

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

#[derive(Serialize)]
pub struct GetProjectInfoResponse {
    #[serde(flatten)]
    pub project: Project,
}

pub async fn get_project_info(
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

    let project = state.project_repo.query_project_by_id(&project_id).await;

    match project {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(project) => {
            let project = project_db_to_api(project);

            match project {
                None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                Some(project) => {
                    (StatusCode::OK, Json(GetProjectInfoResponse { project })).into_response()
                }
            }
        }
    }
}

#[derive(Serialize)]
pub struct GetUsersForProjectResponse {
    pub users: Vec<User>,
}

pub async fn get_users_for_project(
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

    let members = state.project_repo.query_members_by_id(&project_id).await;
    let admin = state.project_repo.query_admin_by_id(&project_id).await;

    match (admin, members) {
        (Ok(admin), Ok(members)) => {
            let admin = user_db_to_api(admin);
            let members: Option<Vec<_>> = members
                .iter()
                .map(|user| user_db_to_api(user.clone()))
                .collect();
            match (admin, members) {
                (Some(admin), Some(ref mut members)) => {
                    members.push(admin);

                    (
                        StatusCode::OK,
                        Json(GetUsersForProjectResponse {
                            users: members.clone(),
                        }),
                    )
                        .into_response()
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateProjectRequest {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_pool: Option<StatusPool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateProjectResponse {
    #[serde(flatten)]
    pub project: Project,
}

pub async fn create_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let user_id = auth_session.user;

    let user_id = match user_id {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(id) => id,
    };

    let user_id = user_id.id;

    let user_id = match user_id {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(id) => id.id.to_string(),
    };

    let db_project = project_api_to_db(Project {
        id: String::new(),
        name: req.name,
        description: req.description,
        avatar: req.avatar,
        status_pool: req.status_pool,
    });
    let state = state.lock().await;

    let returned_db_project = state.project_repo.insert_project(&db_project).await;

    let db_project = match returned_db_project {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(project) => project,
    };

    let api_project = project_db_to_api(db_project);

    let project_id = match api_project {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(ref project) => project.id.clone(),
    };

    let result = state
        .project_repo
        .set_user_for_project(&user_id, &project_id, true)
        .await;

    if let Err(_) = result {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match api_project {
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(project) => (StatusCode::OK, Json(CreateProjectResponse { project })).into_response(),
    }
}
