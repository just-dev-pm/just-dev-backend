use std::io;

use axum_login::AuthUser;

use crate::db::{model::user::User, repository::{agenda::AgendaRepository, task::TaskRepository, user::UserRepository}};
use crate::db::repository::utils::get_io_error;



pub async fn insert_user(user_repo:&UserRepository, task_repo:&TaskRepository, agenda_repo: &AgendaRepository, user:&User) -> Result<User, io::Error> {
    let result: Option<User> = user_repo
        .context
        .db
        .create("user")
        .content(user)
        .await
        .map_err(|e| get_io_error(e))?.pop();
    let user = result.ok_or(io::Error::new(io::ErrorKind::NotFound, "User insert fail"))?;
    let _ = agenda_repo.insert_exagenda_for_user("Excluded agenda for you", &user.id());
    
    Ok(user)
}