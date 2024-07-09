use std::io;

use axum_login::AuthUser;

use crate::db::{model::user::User, repository::{task::TaskRepository, user::UserRepository}};
use crate::db::repository::utils::get_io_error;



pub async fn insert_user(user_repo:&UserRepository, task_repo:&TaskRepository, user:&User) -> Result<User, io::Error> {
    let result: Option<User> = user_repo
        .context
        .db
        .create("user")
        .content(user)
        .await
        .map_err(|e| get_io_error(e))?.pop();

    task_repo.insert_extask_list_for_user("Tasks assigned to you", &user.id());
    
    result.ok_or(io::Error::new(io::ErrorKind::NotFound, "User insert fail"))
}