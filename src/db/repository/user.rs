use std::io;

use crate::db::{
    db_context::DbContext,
    model::{project::Project, user::User},
};

use super::utils::get_io_error;

#[derive(Clone, Debug)]
pub struct UserRepository {
    pub context: DbContext,
}

impl UserRepository {
    pub async fn new() -> UserRepository {
        UserRepository {
            context: DbContext::new().await,
        }
    }

    pub async fn query_user_by_name(&self, name: &str) -> Result<User, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!("SELECT * FROM user WHERE username == '{}'", name))
            .await
            .unwrap();

        let user: Option<User> = response.take(0).unwrap();

        user.ok_or(io::Error::new(io::ErrorKind::NotFound, "User not found"))
    }

    pub async fn query_user_by_id(&self, id: &str) -> Result<User, io::Error> {
        let user: Option<User> = self
            .context
            .db
            .select(("user", id))
            .await
            .unwrap_or_else(|_| None);
        user.ok_or(io::Error::new(io::ErrorKind::NotFound, "User not found"))
    }

    pub async fn update_user(&self, user_id: &str, user: &User) -> Result<User, io::Error> {
        let result: Option<User> = self
            .context
            .db
            .update(("user", user_id))
            .content(user)
            .await
            .unwrap();
        result.ok_or(io::Error::new(io::ErrorKind::NotFound, "User update fail"))
    }

    pub async fn query_project_join_by_id(&self, user_id: &str) -> Result<(Vec<Project>, Vec<Project>), io::Error> {
        let mut response = self
            .context
            .db
            .query(format!(
                "select out.* as project from join where in.id == user:{user_id} and admin == true"
            ))
            .query(format!("select out.* as project from join where in.id == user:{user_id} and admin == false"))
            .await
            .map_err(get_io_error)?;
        let admin_projects: Vec<Project> = response.take((0, "project")).map_err(get_io_error)?;
        let member_projects: Vec<Project> = response.take((1, "project")).map_err(get_io_error)?;
        Ok((admin_projects, member_projects))
    }
}
