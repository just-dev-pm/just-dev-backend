use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use axum_login::AuthSession;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{app::AppState, model::task::TaskList},
    usecase::util::auth_backend::AuthBackend,
};

use super::{
    task::{
        create_task_for_list, delete_task_from_list, get_all_tasks_for_project, get_all_tasks_for_user, get_assigned_tasks_for_user, get_tasks_for_list, patch_task
    },
    util::{
        authorize_against_project_id, authorize_against_task_list_id, authorize_against_user_id,
        task_list_db_to_api,
    },
};

pub fn user_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route(
            "/task_lists",
            get(get_task_lists_for_user).post(create_task_list_for_user),
        )
        .route("/tasks", get(get_assigned_tasks_for_user))
        .route("/tasks/personal", get(get_all_tasks_for_user))
}

pub fn project_router() -> Router<Arc<Mutex<AppState>>> {
    Router::new().route(
        "/task_lists",
        get(get_task_lists_for_project).post(create_task_list_for_project),
    )
    .route("/tasks", get(get_all_tasks_for_project))
}

pub fn router() -> Router<Arc<Mutex<AppState>>> {
    let router = Router::new()
        .route("/tasks", get(get_tasks_for_list).post(create_task_for_list))
        .route(
            "/tasks/:task_id",
            delete(delete_task_from_list).patch(patch_task),
        )
        .route("/", get(get_task_list_info).delete(delete_task_list));

    Router::new().nest("/:task_list_id", router)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListInfoResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn get_task_list_info(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(task_list_id): Path<String>,
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
    }

    let db_task_list = match state.task_repo.query_task_list_by_id(&task_list_id).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_list) => task_list,
    };

    let api_task_list = task_list_db_to_api(db_task_list);

    match api_task_list {
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(api_task_list) => (
            StatusCode::OK,
            Json(GetTaskListInfoResponse {
                task_list: api_task_list,
            }),
        )
            .into_response(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListsForProjectResponse {
    task_lists: Vec<TaskList>,
}

pub async fn get_task_lists_for_project(
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

    let db_task_lists = state.project_repo.query_task_list_by_id(&project_id).await;

    let db_task_lists = match db_task_lists {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_lists) => task_lists,
    };

    let db_task_list_futures: Vec<_> = db_task_lists
        .into_iter()
        .map(|id| {
            let task_repo = &state.task_repo;
            async move { task_repo.query_task_list_by_id(&id).await }
        })
        .collect();

    let db_task_lists = try_join_all(db_task_list_futures).await;

    let db_task_lists = match db_task_lists {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_lists) => task_lists,
    };

    let api_task_lists: Option<Vec<_>> = db_task_lists
        .into_iter()
        .map(|task_list| task_list_db_to_api(task_list))
        .collect();

    match api_task_lists {
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(task_lists) => (
            StatusCode::OK,
            Json(GetTaskListsForProjectResponse { task_lists }),
        )
            .into_response(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTaskListsForUserResponse {
    task_lists: Vec<TaskList>,
}

pub async fn get_task_lists_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let db_task_lists = state.user_repo.query_task_list_by_id(&user_id).await;
    let db_task_lists = match db_task_lists {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_lists) => task_lists,
    };

    let db_task_list_futures: Vec<_> = db_task_lists
        .into_iter()
        .map(|id| {
            let task_repo = &state.task_repo;
            async move { task_repo.query_task_list_by_id(&id).await }
        })
        .collect();

    let db_task_lists = try_join_all(db_task_list_futures).await;

    let db_task_lists = match db_task_lists {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_lists) => task_lists,
    };

    let api_task_lists: Option<Vec<_>> = db_task_lists
        .into_iter()
        .map(|task_list| task_list_db_to_api(task_list))
        .collect();

    match api_task_lists {
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(task_lists) => (
            StatusCode::OK,
            Json(GetTaskListsForUserResponse { task_lists }),
        )
            .into_response(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateTaskListForProjectRequest {
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateTaskListForProjectResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn create_task_list_for_project(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateTaskListForProjectRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;

    if let Some(value) =
        authorize_against_project_id(auth_session, &state.project_repo, &project_id).await
    {
        return value;
    }

    let returned_db_task_list = state
        .task_repo
        .insert_task_list_for_project(&project_id, &req.name)
        .await;

    let returned_db_task_list = match returned_db_task_list {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_list) => task_list,
    };

    let api_task_list = match task_list_db_to_api(returned_db_task_list) {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(task_list) => task_list,
    };

    (
        StatusCode::OK,
        Json(CreateTaskListForProjectResponse {
            task_list: api_task_list,
        }),
    )
        .into_response()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskListForUserRequest {
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTaskListForUserResponse {
    #[serde(flatten)]
    pub task_list: TaskList,
}

pub async fn create_task_list_for_user(
    auth_session: AuthSession<AuthBackend>,
    State(state): State<Arc<Mutex<AppState>>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateTaskListForUserRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;

    if let Some(value) = authorize_against_user_id(auth_session, &user_id) {
        return value;
    }

    let returned_db_task_list = state
        .task_repo
        .insert_task_list_for_user(&req.name, &user_id)
        .await;

    let returned_db_task_list = match returned_db_task_list {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(task_list) => task_list,
    };

    let api_task_list = match task_list_db_to_api(returned_db_task_list) {
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Some(task_list) => task_list,
    };

    (
        StatusCode::OK,
        Json(CreateTaskListForUserResponse {
            task_list: api_task_list,
        }),
    )
        .into_response()
}

pub async fn delete_task_list(
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

    let tasks = match state
        .task_repo
        .query_all_tasks_of_task_list(&task_list_id)
        .await
    {
        Ok(tasks) => tasks,
        Err(_) => return StatusCode::OK.into_response(),
    };

    let task_futures: Vec<_> = tasks
        .into_iter()
        .map(|task_id| async move { state.task_repo.delete_task(&task_id).await })
        .collect();

    let _ = try_join_all(task_futures).await;

    match state.task_repo.delete_task_list(&task_list_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
