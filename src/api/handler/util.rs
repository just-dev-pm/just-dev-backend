use crate::{
    api::model::status::{IndexedStatusItem, StatusItem},
    db::model::status::StatusPool,
};

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
