use std::io::Error;

use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_login::{AuthSession, AuthUser};
use surrealdb::sql::Thing;

use crate::{api::model::asset::{DraftPath, EventPath, TaskPath}, db::{
    model::{
        notification::AssetPath,
        status::{Status, StatusPool},
        task::Task,
    },
    repository::{
        agenda::AgendaRepository, project::ProjectRepository, task::TaskRepository,
        user::UserRepository, utils::unwrap_thing,
    },
}};
use crate::usecase::util::auth_backend::AuthBackend;
use crate::{
    api::model::{
        agenda::Event,
        asset::Asset,
        status::{IndexedStatusContent, StatusContent},
        task::{TaskRelation, TaskRelationType},
        util::Id,
    },
    db::{model::task::TaskLink, repository::utils::custom_io_error},
};

pub fn authorize_against_user_id(
    auth_session: AuthSession<AuthBackend>,
    user_id: &String,
) -> Option<axum::http::Response<axum::body::Body>> {
    match auth_session.user {
        None => return Some(StatusCode::UNAUTHORIZED.into_response()),
        Some(user) => match user.id().eq(user_id) {
            true => (),
            false => return Some(StatusCode::UNAUTHORIZED.into_response()),
        },
    };
    None
}

pub async fn authorize_against_project_id(
    auth_session: AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    project_id: &String,
) -> Option<axum::http::Response<axum::body::Body>> {
    let user_id = match auth_session.user {
        None => return Some(StatusCode::UNAUTHORIZED.into_response()),
        Some(user) => user.id(),
    };

    let admin = project_repo.query_admin_by_id(project_id).await;
    let members = project_repo.query_members_by_id(project_id).await;

    match (admin, members) {
        (Ok(admin), Ok(members)) => {
            let member_id_eqs: Vec<_> = members
                .iter()
                .map(|user| {
                    user.id
                        .clone()
                        .is_some_and(|thing| thing.id.to_string().eq(&user_id))
                })
                .collect();
            if admin.id().eq(&user_id) || member_id_eqs.contains(&true) {
                None
            } else {
                Some(StatusCode::UNAUTHORIZED.into_response())
            }
        }
        _ => Some(StatusCode::UNAUTHORIZED.into_response()),
    }
}

pub async fn authorize_admin_against_project_id(
    auth_session: &AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    project_id: &String,
) -> Option<axum::http::Response<axum::body::Body>> {
    let user_id = match auth_session.user.clone() {
        None => return Some(StatusCode::UNAUTHORIZED.into_response()),
        Some(user) => user.id(),
    };
    let admin = project_repo.query_admin_by_id(project_id).await;

    match admin {
        Err(_) => Some(StatusCode::UNAUTHORIZED.into_response()),
        Ok(admin) => match admin.id().eq(&user_id) {
            false => Some(StatusCode::UNAUTHORIZED.into_response()),
            true => None,
        },
    }
}

pub async fn authorize_against_agenda_id(
    auth_session: &AuthSession<AuthBackend>,
    user_repo: &UserRepository,
    agenda_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    let user_id = match auth_session.user.clone() {
        None => return Some(StatusCode::UNAUTHORIZED.into_response()),
        Some(user) => user.id(),
    };
    let agendas = match user_repo.query_agenda_by_id(&user_id).await {
        Ok(agendas) => agendas,
        Err(_) => return Some(StatusCode::UNAUTHORIZED.into_response()),
    };
    if agendas.contains(&agenda_id.to_string()) {
        None
    } else {
        Some(StatusCode::UNAUTHORIZED.into_response())
    }
}

pub async fn authorize_against_event_id(
    auth_session: &AuthSession<AuthBackend>,
    agenda_repo: &AgendaRepository,
    user_repo: &UserRepository,
    agenda_id: &str,
    event_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    if let Some(value) = authorize_against_agenda_id(&auth_session, user_repo, &agenda_id).await {
        return Some(value);
    }
    let event_ids = agenda_repo.query_event_id_by_agenda_id(&agenda_id).await;
    if let Err(msg) = event_ids {
        Some((StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response())
    } else {
        if !event_ids.unwrap().contains(&event_id.to_owned()) {
            Some(StatusCode::UNAUTHORIZED.into_response())
        } else {
            None
        }
    }
}

pub async fn authorize_against_task_id(
    auth_session: &AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    task_repo: &TaskRepository,
    task_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    let task_list_id = match task_repo.query_task_list_id_by_task(task_id).await {
        Ok(_id) => _id,
        Err(err) => {
            return Some((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        }
    };

    if let Some(value) = authorize_against_task_list_id(
        auth_session.to_owned(),
        project_repo,
        task_repo,
        &task_list_id,
    )
    .await
    {
        return Some(value);
    };
    None
}

pub async fn authorize_against_task_link_id(
    auth_session: &AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    task_repo: &TaskRepository,
    link_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    let link = match task_repo.query_task_link_by_id(&link_id).await {
        Ok(_link) => _link,
        Err(err) => {
            return Some(
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
            );
        }
    };
    let link = match task_link_db_to_api(link) {
        Ok(link) => link,
        Err(_) => {
            return Some(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed to convert link".to_string()),
                )
                    .into_response(),
            );
        }
    };
    if let Some(value) =
        authorize_against_task_id(auth_session, project_repo, task_repo, &link.to.id).await
    {
        return Some(value);
    }

    if let Some(value) =
        authorize_against_task_id(auth_session, project_repo, task_repo, &link.from.id).await
    {
        return Some(value);
    }
    None
}

pub async fn authorize_against_task_link(
    auth_session: &AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    task_repo: &TaskRepository,
    from_id: &str,
    to_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    if let Some(value) =
        authorize_against_task_id(auth_session, project_repo, task_repo, from_id).await
    {
        return Some(value);
    }

    if let Some(value) =
        authorize_against_task_id(auth_session, project_repo, task_repo, to_id).await
    {
        return Some(value);
    }
    None
}

pub async fn authorize_against_task_list_id(
    auth_session: AuthSession<AuthBackend>,
    project_repo: &ProjectRepository,
    task_repo: &TaskRepository,
    task_list_id: &str,
) -> Option<axum::http::Response<axum::body::Body>> {
    let user_id = match auth_session.user.clone() {
        None => return Some(StatusCode::UNAUTHORIZED.into_response()),
        Some(user) => user.id(),
    };

    let source = task_repo.query_task_list_source(&task_list_id).await;
    if let Err(err) = source {
        return Some((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response());
    }
    let source = source.unwrap();
    match source.tb.as_str() {
        "project" => {
            let project_id = source.id.to_string();
            if let Some(value) =
                authorize_against_project_id(auth_session, project_repo, &project_id).await
            {
                return Some(value);
            }
        }
        "user" => {
            if !source.id.to_string().eq(&user_id) {
                return Some(
                    (StatusCode::FORBIDDEN, "The task list is not belong to you").into_response(),
                );
            }
        }
        _ => {
            return Some(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Exception in target list",
                )
                    .into_response(),
            );
        }
    };
    None
}

// pub async fn authorize_against_draft_id(
//     auth_session: &AuthSession<AuthBackend>,
//     user_repo: &UserRepository,
//     project_repo: &ProjectRepository,
//     draft_id: &String,
// ) -> Option<axum::http::Response<axum::body::Body>> {
//     let user_id = match auth_session.user.clone() {
//         None => return Some(StatusCode::UNAUTHORIZED.into_response()),
//         Some(user) => user.id(),
//     };

//     let user_drafts = user_repo
//         .query_draft_by_id(&user_id)
//         .await
//         .unwrap_or(Some(StatusCode::UNAUTHORIZED.into_response()));

//     let (admin_projects, member_projects) = user_repo
//         .query_project_join_by_id(&user_id)
//         .await
//         .unwrap_or(Some(StatusCode::UNAUTHORIZED.into_response()));

//     let admin_projects: Option<Vec<_>> = admin_projects
//         .iter()
//         .map(|proj| proj.id.clone().map(|id| id.id.to_string()))
//         .collect();

//     let admin_projects = admin_projects.unwrap_or(Some(StatusCode::UNAUTHORIZED.into_response()));

//     let member_projects: Option<Vec<_>> = member_projects
//         .iter()
//         .map(|proj| proj.id.clone().map(|id| id.id.to_string()))
//         .collect();

//     let member_projects = member_projects.unwrap_or(Some(StatusCode::UNAUTHORIZED.into_response()));

//     let admin_project_drafts: Result<Vec<_>, _> = admin_projects.iter().map(|id| project_repo.query_project_by_id())
// }

pub fn user_db_to_api(user: crate::db::model::user::User) -> Option<crate::api::model::user::User> {
    if let Some(id) = user.id {
        let email = if user.email.is_empty() {
            None
        } else {
            Some(user.email)
        };

        let avatar = if user.avatar.is_empty() {
            None
        } else {
            Some(user.avatar)
        };

        Some(crate::api::model::user::User {
            id: id.id.to_string(),
            email,
            username: user.username,
            avatar,
            status_pool: status_pool_db_to_api(user.status_pool),
        })
    } else {
        None
    }
}

pub fn user_api_to_db(
    user: crate::api::model::user::User,
    password: &str,
) -> crate::db::model::user::User {
    crate::db::model::user::User {
        id: Some(surrealdb::sql::Thing::from((
            String::from("user"),
            surrealdb::sql::Id::String(user.id),
        ))),
        username: user.username,
        avatar: user.avatar.unwrap_or(String::new()),
        password: String::from(password),
        email: user.email.unwrap_or(String::new()),
        status_pool: match user.status_pool {
            None => StatusPool::default(),
            Some(status_pool) => status_pool_api_to_db(status_pool),
        },
    }
}

pub fn status_pool_db_to_api(
    status_pool: crate::db::model::status::StatusPool,
) -> Option<crate::api::model::status::StatusPool> {
    Some(crate::api::model::status::StatusPool {
        incomplete: status_pool
            .incomplete
            .iter()
            .map(|status| IndexedStatusContent {
                id: format!("{}", status.number),
                status: StatusContent {
                    name: status.name.clone(),
                    description: status.description.clone(),
                },
            })
            .collect(),
        complete: crate::api::model::status::StatusContent {
            name: status_pool.complete.name,
            description: status_pool.complete.description,
        },
    })
}

pub fn status_pool_api_to_db(status_pool: crate::api::model::status::StatusPool) -> StatusPool {
    StatusPool {
        incomplete: status_pool
            .incomplete
            .iter()
            .map(|indexed| Status {
                name: indexed.clone().status.name,
                description: indexed.clone().status.description,
                number: indexed.id.to_owned(),
            })
            .collect(),
        complete: Status {
            name: status_pool.complete.name,
            number: "0".to_owned(),
            description: status_pool.complete.description,
        },
    }
}

pub fn credential_api_to_user_db(
    cred: crate::api::model::user::Credential,
) -> Option<crate::db::model::user::User> {
    Some(crate::db::model::user::User {
        id: None,
        username: cred.username,
        avatar: String::new(),
        email: String::new(),
        password: cred.password,
        status_pool: StatusPool::default(),
    })
}

pub fn project_db_to_api(
    project: crate::db::model::project::Project,
) -> Option<crate::api::model::project::Project> {
    let id = match project.id {
        None => return None,
        Some(id) => id.id.to_string(),
    };

    Some(crate::api::model::project::Project {
        id,
        name: project.name,
        description: String::new(),
        avatar: project.avatar,
        status_pool: status_pool_db_to_api(project.status_pool),
        github: Some(project.github),
    })
}

pub fn project_api_to_db(
    project: crate::api::model::project::Project,
) -> crate::db::model::project::Project {
    crate::db::model::project::Project {
        id: if project.id.is_empty() {
            None
        } else {
            Some(Thing::from((
                "project",
                surrealdb::sql::Id::String(project.id),
            )))
        },
        name: project.name,
        avatar: project.avatar,
        status_pool: match project.status_pool {
            None => StatusPool::default(),
            Some(status_pool) => status_pool_api_to_db(status_pool),
        },
        github: 0,
    }
}

use crate::db::model::notification::NotificationSource;


pub fn notif_db_to_api(
    notif: crate::db::model::notification::Notification,
    source: NotificationSource,
) -> crate::api::model::notification::Notification {
    crate::api::model::notification::Notification {
        id: unwrap_thing(notif.id.unwrap()),
        title: notif.title,
        content: notif.content,
        handled: notif.handled,
        asset: match source {
            NotificationSource::Task(AssetPath(task_id, (task_list_id, project_id))) => {
                Asset::Task {
                    path: TaskPath {
                        task_id,
                        task_list_id,
                        project_id,
                    }
                }
            }
            NotificationSource::Event(AssetPath(event_id, (agenda_id, project_id))) => {
                Asset::Event {
                    path: EventPath {
                        event_id,
                        agenda_id,
                        project_id,
                    }
                }
            }
            NotificationSource::Draft(AssetPath(id, _)) => Asset::Draft { path: DraftPath { id } },
        },
    }
}

pub fn requ_db_to_api(
    requ: crate::db::model::requirement::Requirement,
) -> crate::api::model::requirement::Requirement {
    crate::api::model::requirement::Requirement {
        id: unwrap_thing(requ.id.unwrap()),
        name: requ.name,
        content: requ.description,
    }
}

pub fn draft_db_to_api(
    draft: crate::db::model::draft::DraftPayload,
) -> Option<crate::api::model::draft::Draft> {
    let id = match draft.id {
        None => return None,
        Some(id) => id,
    };

    Some(crate::api::model::draft::Draft {
        id,
        name: draft.name,
    })
}

pub fn event_db_to_api(
    event: crate::db::model::agenda::Event,
    participants: Vec<Id>,
) -> crate::api::model::agenda::Event {
    Event {
        id: unwrap_thing(event.id.unwrap()),
        name: event.name,
        description: event.description,
        start_time: event.start_time.0,
        end_time: event.end_time.0,
        participants,
    }
}

pub fn agenda_db_to_api(
    agenda: crate::db::model::agenda::Agenda,
    events: Option<Vec<String>>,
) -> crate::api::model::agenda::Agenda {
    crate::api::model::agenda::Agenda {
        id: unwrap_thing(agenda.id.unwrap()),
        name: agenda.name,
        events: events
            .unwrap_or_default()
            .into_iter()
            .map(|event| Id { id: event })
            .collect(),
    }
}

pub fn task_list_db_to_api(
    task_list: crate::db::model::task::TaskList,
) -> Option<crate::api::model::task::TaskList> {
    let id = match task_list.id {
        None => return None,
        Some(id) => id.id.to_string(),
    };

    Some(crate::api::model::task::TaskList {
        id,
        name: task_list.name,
        tasks: match task_list.tasks {
            None => vec![],
            Some(tasks) => tasks.into_iter().map(|id| Id { id }).collect(),
        },
    })
}

pub fn task_db_to_api_assigned(
    (task, task_list, project): (Task, String, String),
) -> crate::api::handler::task::AssignedTask {
    crate::api::handler::task::AssignedTask {
        id: unwrap_thing(task.id.unwrap()),
        name: task.name,
        description: task.description,
        assignees: task
            .assignees
            .unwrap_or_default()
            .into_iter()
            .map(|a| Id { id: a })
            .collect(),
        status: match task.complete {
            true => crate::api::model::status::Status::Complete,
            false => crate::api::model::status::Status::Incomplete { id: task.status },
        },
        deadline: task.ddl.unwrap_or_default().0,
        project,
        task_list,
    }
}

pub fn task_db_to_api(task: Task) -> crate::api::model::task::Task {
    crate::api::model::task::Task {
        id: unwrap_thing(task.id.unwrap()),
        name: task.name,
        description: task.description,
        assignees: task
            .assignees
            .unwrap_or_default()
            .into_iter()
            .map(|a| Id { id: a })
            .collect(),
        status: match task.complete {
            true => crate::api::model::status::Status::Complete,
            false => crate::api::model::status::Status::Incomplete { id: task.status },
        },
        deadline: task.ddl.unwrap_or_default().0,
        pr: match task.pr_assigned {
            true => Some(task.pr),
            false => None
        },
    }
}

pub fn task_link_db_to_api(link: TaskLink) -> Result<TaskRelation, Error> {
    Ok(TaskRelation {
        id: unwrap_thing(
            link.id
                .ok_or(custom_io_error("Thing unwrap failed in task relation!"))?,
        ),
        from: Id {
            id: unwrap_thing(
                link.incoming
                    .ok_or(custom_io_error("Thing unwrap failed in task relation!"))?,
            ),
        },
        to: Id {
            id: unwrap_thing(
                link.outgoing
                    .ok_or(custom_io_error("Thing unwrap failed in task relation!"))?,
            ),
        },
        category: match link.kind.as_str() {
            "auto" => TaskRelationType::Auto,
            "dep" => TaskRelationType::Dep,
            _ => return Err(custom_io_error("Exception in task relation kind!")),
        },
    })
}

pub fn task_relation_category_to_kind<'a>(category: &'a TaskRelationType) -> &'a str {
    match category {
        TaskRelationType::Auto => "auto",
        TaskRelationType::Dep => "dep",
    }
}
