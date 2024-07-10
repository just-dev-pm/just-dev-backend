use std::io;

use surrealdb::sql::Thing;

use crate::db::{
    db_context::DbContext,
    model::{
        agenda::Agenda, draft::{Draft, DraftPayload}, project::Project, task::TaskList, user::User
    },
};

use super::utils::{exec_query, get_io_error, unwrap_thing, unwrap_things, DbModelId};

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

    pub async fn query_project_join_by_id(
        &self,
        user_id: &str,
    ) -> Result<(Vec<Project>, Vec<Project>), io::Error> {
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

    pub async fn query_draft_by_id(&self, user_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->own->draft as drafts from user where id == user:{user_id}"),
        )
        .await?;
        let drafts: Option<Vec<Thing>> = response.take((0, "drafts")).map_err(get_io_error)?;

        Ok(unwrap_things(drafts.unwrap_or_default()))
    }

    pub async fn query_agenda_by_id(&self, user_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->own->agenda as agendas from user where id == user:{user_id}"),
        )
        .await?;
        let agendas: Option<Vec<Thing>> = response.take((0, "agendas")).map_err(get_io_error)?;

        Ok(unwrap_things(agendas.unwrap_or_default()))
    }

    pub async fn query_task_list_by_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->own->task_list as task_lists from user where id == user:{user_id}"),
        )
        .await?;
        let task_lists: Option<Vec<Thing>> = response.take((0, "task_lists")).map_err(get_io_error)?;

        Ok(unwrap_things(task_lists.unwrap_or_default()))
    }
}
