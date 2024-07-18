use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use tokio::sync::Mutex;
// use axum_core::Response;

use crate::{
    api::{
        app::AppState,
        model::{pr::PullRequest, status::Status, task::Task, util::Id},
    },
    db::repository::utils::unwrap_thing,
    usecase::{
        notification::{assign_task_to_user, deassign_task_for_user},
        task_stream::{check_task_switch_complete, refresh_task_status_entry, TaskSwitchable},
        util::auth_backend::AuthBackend,
    },
};

use super::util::{
    authorize_against_project_id, authorize_against_task_list_id, authorize_against_user_id,
    task_db_to_api, task_db_to_api_assigned,
};

pub struct IoErrorWrapper(io::Error);

impl IntoResponse for IoErrorWrapper {
    fn into_response(self) -> Response<Body> {
        let status_code = match self.0.kind() {
            io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.0.to_string()).into_response()
    }
}

impl From<io::Error> for IoErrorWrapper {
    fn from(err: io::Error) -> Self {
        IoErrorWrapper(err)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskWithListId {
    #[serde(flatten)]
    pub task: Task,
    pub task_list_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTasksForUser {
    pub tasks: Vec<TaskWithListId>,
}

pub async fn get_all_tasks_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, IoErrorWrapper> {
    let ref state = state.lock().await;
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return Ok(value);
    }

    let task_lists = state.user_repo.query_task_list_by_id(&user_id).await?;
    let mut tasks = vec![];
    for list in task_lists {
        let list_tasks = state.task_repo.query_all_tasks_of_task_list(&list).await?;
        for task in list_tasks {
            tasks.push(TaskWithListId {
                task: task_db_to_api(state.task_repo.query_task_by_id(&task).await?),
                task_list_id: list.clone(),
            });
        }
    }

    Ok((StatusCode::OK, Json(GetTasksForUser { tasks })).into_response())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTasksForProject {
    pub tasks: Vec<TaskWithListId>,
}

pub async fn get_all_tasks_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, IoErrorWrapper> {
    let ref state = state.lock().await;
    let ref task_repo = state.task_repo;
    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return Ok(value);
    }

    let task_lists = state
        .project_repo
        .query_task_list_by_id(&project_id)
        .await?;
    let mut tasks = vec![];
    for list in task_lists {
        let list_tasks = task_repo.query_all_tasks_of_task_list(&list).await?;
        for task in list_tasks {
            tasks.push(TaskWithListId {
                task: task_db_to_api(state.task_repo.query_task_by_id(&task).await?),
                task_list_id: list.clone(),
            });
        }
    }

    Ok((StatusCode::OK, Json(GetTasksForProject { tasks })).into_response())
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<PullRequest>,
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
) -> Result<impl IntoResponse, IoErrorWrapper> {
    let ref state = state.lock().await;
    if let Some(value) = authorize_against_task_list_id(
        auth_session,
        &state.project_repo,
        &state.task_repo,
        &task_list_id,
    )
    .await
    {
        return Ok(value);
    }

    let assignees: Vec<_> = req.assignees.clone().into_iter().map(|id| id.id).collect();

    let mut task = crate::db::model::task::Task {
        id: None,
        name: req.name.clone(),
        description: req.description.clone(),
        assignees: Some(assignees.clone()),
        status: match &req.status {
            Status::Complete => "complete".to_owned(),
            Status::Incomplete { id } => id.to_owned(),
        },
        ddl: Some(Datetime {
            0: req.deadline.clone(),
        }),
        complete: match req.status {
            Status::Complete => true,
            Status::Incomplete { .. } => false,
        },
        pr_assigned: false,
        pr_number: 0,
        pr: crate::api::model::pr::PullRequest::default(),
    };

    match req.pr {
        Some(_pr) => {
            task.pr_number = _pr.pull_number.clone();
            task.pr = _pr;
            task.pr_assigned = true;
        }
        None => {
            task.pr_assigned = false;
            task.pr = PullRequest::default();
        }
    }

    let mut task = state
        .task_repo
        .insert_task_for_task_list(&task, &task_list_id)
        .await?;

    let task_id = unwrap_thing(task.id.clone().unwrap());
    for id in &assignees {
        let _ = assign_task_to_user(&state.task_repo, &state.notif_repo, &task_id, &id).await?;
    }

    task.assignees = Some(assignees);

    Ok((
        StatusCode::OK,
        Json(CreateTaskForListResponse {
            task: task_db_to_api(task),
        }),
    )
        .into_response())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<PullRequest>,
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
        id: None,
        ..task.clone()
    };

    let mut switchable = TaskSwitchable::TrueAndFalse;

    if let Some(_) = req.status {
        switchable = match check_task_switch_complete(&task_id, &state.task_repo).await {
            Ok(_result) => _result,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response()
            }
        }
    }

    (new_task.complete, new_task.status) = match req.status {
        Some(Status::Complete) => match switchable {
            TaskSwitchable::False => (false, "incomplete".to_owned()),
            _ => (true, "complete".to_owned()),
        },
        Some(Status::Incomplete { id }) => match switchable {
            TaskSwitchable::True => (true, id),
            _ => (false, id),
        },
        None => (task.complete, task.status),
    };

    if let Some(_pr) = req.pr {
        new_task.pr_assigned = true;
        new_task.pr = _pr;
    }

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

    if task.complete != new_task.complete {
        if let Err(err) = refresh_task_status_entry(&task_id, &state.task_repo).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
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
