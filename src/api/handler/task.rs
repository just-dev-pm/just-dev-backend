use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use tokio::sync::Mutex;

use crate::{
    api::{
        app::AppState,
        model::{status::Status, task::Task, util::Id},
    },
    usecase::{
        notification::{assign_task_to_user, deassign_task_for_user},
        util::auth_backend::AuthBackend,
    },
};

use super::{
    user::PatchUserInfoRequest,
    util::{
        authorize_against_task_list_id, authorize_against_user_id, task_db_to_api,
        task_db_to_api_assigned,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTasksForList {
    pub tasks: Vec<Task>,
}

pub async fn get_tasks_for_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    if let Some(value) = authorize_against_task_list_id(
        auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_list_id,
    )
    .await
    {
        return value;
    }

    let tasks = state
        .task_repo
        .query_all_tasks_of_task_list(&task_list_id)
        .await;

    let tasks = match tasks {
        Ok(_tasks) => _tasks,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    let tasks: Vec<_> = tasks
        .into_iter()
        .map(|id| async move { state.task_repo.query_task_by_id(&id).await })
        .collect();

    let tasks = match try_join_all(tasks).await {
        Ok(_tasks) => _tasks,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    (
        StatusCode::OK,
        Json(GetTasksForList {
            tasks: tasks.into_iter().map(|task| task_db_to_api(task)).collect(),
        }),
    )
        .into_response()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskForListRequest {
    pub name: String,
    pub description: String,
    pub assignees: Vec<Id>,
    pub status: Status,
    pub deadline: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskForListResponse {
    #[serde(flatten)]
    pub task: Task,
}

pub async fn create_task_for_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
    Json(req): Json<CreateTaskForListRequest>,
) -> impl IntoResponse {
    let ref state = state.lock().await;
    if let Some(value) = authorize_against_task_list_id(
        auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_list_id,
    )
    .await
    {
        return value;
    }

    let task = crate::db::model::task::Task {
        id: None,
        name: req.name.clone(),
        description: req.description.clone(),
        assignees: Some(req.assignees.into_iter().map(|id| id.id).collect()),
        status: match req.status {
            Status::Complete => "complete".to_owned(),
            Status::Incomplete { id } => id,
        },
        ddl: Some(Datetime {
            0: req.deadline.clone(),
        }),
        complete: false,
    };

    let task = match state
        .task_repo
        .insert_task_for_task_list(&task, &task_list_id)
        .await
    {
        Ok(_task) => _task,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    (
        StatusCode::OK,
        Json(CreateTaskForListResponse {
            task: task_db_to_api(task),
        }),
    )
        .into_response()
}

pub async fn delete_task_from_list(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((task_list_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) = authorize_against_task_list_id(
        auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_list_id,
    )
    .await
    {
        return value;
    };

    match state.task_repo.delete_task(&task_id).await {
        Ok(_) => (StatusCode::OK, Json("")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<Id>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchTaskResponse {
    #[serde(flatten)]
    pub task: Task,
}

pub async fn patch_task(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path((task_list_id, task_id)): Path<(String, String)>,
    Json(req): Json<PatchTaskRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) = authorize_against_task_list_id(
        auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_list_id,
    )
    .await
    {
        return value;
    };

    let task = match state.task_repo.query_task_by_id(&task_id).await {
        Ok(_task) => _task,
        Err(err) => return (StatusCode::BAD_REQUEST, Json(err.to_string())).into_response(),
    };

    let mut new_task = crate::db::model::task::Task {
        name: req.name.unwrap_or(task.name.clone()),
        description: req.description.unwrap_or(task.description.clone()),
        ddl: req.deadline.map(|ddl| Datetime { 0: ddl }),
        complete: task.complete,
        ..task.clone()
    };

    (new_task.complete, new_task.status) = match req.status {
        Some(Status::Complete) => (true, "complete".to_owned()),
        Some(Status::Incomplete { id }) => (false, id),
        None => (task.complete, task.status),
    };

    let assignees = match state.task_repo.query_assignees_of_task(&task_id).await {
        Ok(_assignees) => _assignees,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
        }
    };

    if let Some(assignees_ref) = req.assignees {
        let assignees_ref: Vec<_> = assignees_ref.into_iter().map(|a| a.id).collect();
        for assignee in &assignees {
            if !assignees_ref.contains(assignee) {
                if let Err(err) =
                    deassign_task_for_user(&state.task_repo, &state.notif_repo, &task_id, &assignee)
                        .await
                {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                        .into_response();
                }
            }
        }
        for assignee in &assignees_ref {
            if !assignees.contains(&assignee) {
                if let Err(err) =
                    assign_task_to_user(&state.task_repo, &state.notif_repo, &task_id, &assignee)
                        .await
                {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))
                        .into_response();
                }
            }
        }
        new_task.assignees = Some(assignees_ref);
    }

    if let Err(err) = state.task_repo.update_task_by_id(&task_id, &new_task).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
    }

    (
        StatusCode::OK,
        Json(PatchTaskResponse {
            task: task_db_to_api(new_task),
        }),
    )
        .into_response()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssignedTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assignees: Vec<Id>,
    pub status: Status,
    pub deadline: DateTime<Utc>,
    pub project: String,
    pub task_list: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAssignedTasksReponse {
    pub tasks: Vec<AssignedTask>,
}

pub async fn get_assigned_tasks_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let ref state = state.lock().await;

    let tasks = state
        .task_repo
        .query_assigned_tasks_by_user(&user_id)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(GetAssignedTasksReponse {
            tasks: tasks
                .into_iter()
                .map(|task| task_db_to_api_assigned(task))
                .collect(),
        }),
    )
        .into_response()
}
