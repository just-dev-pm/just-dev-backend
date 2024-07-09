use axum::{extract::ws::close_code::PROTOCOL, http::StatusCode, response::IntoResponse};
use axum_login::{AuthSession, AuthUser};

use crate::{
    api::model::status::{IndexedStatusItem, StatusItem},
    db::{
        model::{
            project::Project,
            status::{Status, StatusPool},
        },
        repository::project::ProjectRepository,
    },
    usecase::util::auth_backend::AuthBackend,
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
            .map(|status| IndexedStatusItem {
                id: format!("{}", status.number.unwrap()),
                status: StatusItem {
                    name: status.name.clone(),
                    description: status.description.clone(),
                },
            })
            .collect(),
        complete: crate::api::model::status::StatusItem {
            name: status_pool.complete.name,
            description: status_pool.complete.description,
        },
    })
}

pub fn status_pool_api_to_db(status_pool: crate::api::model::status::StatusPool) -> StatusPool {
    StatusPool {
        id: None,
        incomplete: status_pool
            .incomplete
            .iter()
            .map(|indexed| Status {
                name: indexed.clone().status.name,
                description: indexed.clone().status.description,
                id: None,
                number: indexed.id.parse().ok(),
            })
            .collect(),
        complete: Status {
            name: status_pool.complete.name,
            number: None,
            description: status_pool.complete.description,
            id: None,
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
    })
}
