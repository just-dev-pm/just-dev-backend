use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_login::{AuthSession, AuthUser};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::api::handler::{agenda, requirement};
use crate::{
    api::{
        app::AppState,
        model::{pr::PullRequest, project::Project, status::StatusPool, user::User},
    },
    usecase::{invitation_token::InvitationInfo, util::auth_backend::AuthBackend},
};

use super::{
    draft, task::IoErrorWrapper, task_link, task_list, util::{
        authorize_admin_against_project_id, authorize_against_project_id,
        authorize_against_user_id, project_api_to_db, project_db_to_api, user_db_to_api,
    }
};

pub fn router() -> axum::Router<Arc<Mutex<AppState>>> {
    let router = Router::new()
        .merge(requirement::router())
        .merge(agenda::project_router())
        .merge(task_link::project_router())
        .merge(task_list::project_router())
        .merge(draft::project_router())
        .route("/", get(get_project_info).patch(patch_project))
        .route("/prs", get(get_all_prs))
        .route("/users", get(get_users_for_project));

    Router::new()
        .route("/", post(create_project))
        .nest("/:project_id", router)
}

pub fn invitation_router() -> axum::Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route("/:token_id", get(get_token_info))
        .route("/accept", post(accept_invitation))
        .route("/generate", post(gen_invitation_token))
}

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
        github: None,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_pool: Option<StatusPool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchProjectResponse {
    #[serde(flatten)]
    pub project: Project,
}

pub async fn patch_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<PatchProjectRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return value;
    }

    let original_db_prject = state.project_repo.query_project_by_id(&project_id).await;

    let original_db_project = match original_db_prject {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(p) => p,
    };

    let original_api_project = project_db_to_api(original_db_project);

    let original_api_project = match original_api_project {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(p) => p,
    };

    let new_api_project = Project {
        id: original_api_project.id,
        name: req.name.unwrap_or(original_api_project.name),
        description: req.description.unwrap_or(original_api_project.description),
        avatar: req.avatar.or(original_api_project.avatar),
        status_pool: req.status_pool.or(original_api_project.status_pool),
        github: None,
    };

    let new_db_project = project_api_to_db(new_api_project);

    let updated_db_project = state
        .project_repo
        .update_project(&new_db_project, &project_id)
        .await;

    let updated_db_project = match updated_db_project {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(p) => p,
    };

    let updated_api_project = project_db_to_api(updated_db_project);

    match updated_api_project {
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(p) => (StatusCode::OK, Json(PatchProjectResponse { project: p })).into_response(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenInvitationTokenRequest {
    pub invitor_id: String,
    pub invitee_id: String,
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenInvitationTokenResponse {
    pub invitation_token: String,
}

pub async fn gen_invitation_token(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<GenInvitationTokenRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let mut invitation_token_repo = state.invitation_token_repo.lock().await;
    let project_repo = &state.project_repo;

    if let Some(value) =
        authorize_admin_against_project_id(&auth_session, project_repo, &req.project_id).await
    {
        return value;
    }
    if let Some(value) = authorize_against_user_id(auth_session, &req.invitor_id) {
        return value;
    }

    let invitation_token = nanoid!();

    invitation_token_repo.tokens.insert(
        invitation_token.clone(),
        InvitationInfo {
            inviter: req.invitor_id,
            invitee: req.invitee_id,
            project: req.project_id,
        },
    );

    (
        StatusCode::OK,
        Json(GenInvitationTokenResponse { invitation_token }),
    )
        .into_response()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AcceptInvitationRequest {
    invitation_token: String,
}

pub async fn accept_invitation(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<AcceptInvitationRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let mut invitation_token_repo = state.invitation_token_repo.lock().await;

    let kv = invitation_token_repo
        .tokens
        .get_key_value(&req.invitation_token);

    let invitation_info = match kv {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(kv) => kv.1.clone(),
    };

    if let Some(value) = authorize_against_user_id(auth_session, &invitation_info.invitee) {
        return value;
    }

    invitation_token_repo
        .tokens
        .retain(|_, v| v.clone() != invitation_info);

    let result = state
        .project_repo
        .set_user_for_project(&invitation_info.invitee, &invitation_info.project, false)
        .await;

    match result {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetTokenInfoResponse {
    pub invitor_id: String,
    pub project_name: String,
}

pub async fn get_token_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(token_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let invitation_token_repo = &state.invitation_token_repo.lock().await;

    let kv = invitation_token_repo.tokens.get_key_value(&token_id);

    let invitation_info = match kv {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(kv) => kv.1.clone(),
    };

    if let Some(value) = authorize_against_user_id(auth_session, &invitation_info.invitee) {
        return value;
    }

    let db_invitor = &state
        .user_repo
        .query_user_by_id(&invitation_info.inviter)
        .await;

    let invitor_id = match db_invitor {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(invitor) => invitor.clone().id(),
    };

    let db_project = &state
        .project_repo
        .query_project_by_id(&invitation_info.project)
        .await;

    let project_name = match db_project {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(project) => project.clone().name,
    };

    (
        StatusCode::OK,
        Json(GetTokenInfoResponse {
            invitor_id,
            project_name,
        }),
    )
        .into_response()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAllPullRequestsResponse {
    pub prs: Vec<PullRequest>,
}

pub async fn get_all_prs(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, IoErrorWrapper> {
    let state = state.lock().await;
    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return Ok(value);
    }

    let prs = state
        .project_repo
        .query_prs_by_project_id(&project_id)
        .await?;
    let prs :Vec<_> =  prs
        .into_iter()
        .map(|pr| PullRequest {
            owner: match pr.user {
                None => "null".to_owned(),
                Some(user) => user.login,
            },
            repo: pr.base.repo.name,
            pull_number: pr.number,
            title: pr.title,
        })
        .collect();

        
        Ok((StatusCode::OK, Json(GetAllPullRequestsResponse { prs })).into_response())
}
