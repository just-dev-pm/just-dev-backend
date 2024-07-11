use std::io;

use surrealdb::sql::Thing;

use crate::db::{
    db_context::DbContext,
    model::{
        agenda::Agenda,
        draft::{Draft, DraftPayload},
        project::Project,
        task::TaskList,
        user::User,
    },
};

use super::utils::{
    exec_double_query, exec_query, get_io_error, unwrap_thing, unwrap_things, DbModelId,
};

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
        let mut response = exec_double_query(
            &self.context,
            format!("select ->own->draft as drafts from user where id == user:{user_id}"),
            format!("select ->join->project->own->draft as drafts from user where id == user:{user_id}")
        )
        .await?;
        let mut drafts = response
            .take::<Option<Vec<Thing>>>((0, "drafts"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        drafts.extend(
            response
                .take::<Option<Vec<Thing>>>((1, "drafts"))
                .map_err(get_io_error)?
                .unwrap_or_default(),
        );

        Ok(unwrap_things(drafts))
    }

    pub async fn query_agenda_by_id(&self, user_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_double_query(
            &self.context,
            format!("select ->own->agenda as agendas from user where id == user:{user_id}"),
            format!("select ->join->project->own->agenda as agendas from user where id == user:{user_id}")
        )
        .await?;
        let mut agendas = response
            .take::<Option<Vec<Thing>>>((0, "agendas"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        agendas.extend(
            response
                .take::<Option<Vec<Thing>>>((1, "agendas"))
                .map_err(get_io_error)?
                .unwrap_or_default(),
        );

        Ok(unwrap_things(agendas))
    }

    pub async fn query_task_list_by_id(&self, user_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_double_query(
            &self.context,
            format!("select ->own->task_list as task_lists from user where id == user:{user_id}"),
            format!("select ->join->project->own->task_list as task_lists from user where id == user:{user_id}")
        )
        .await?;
        let mut task_lists =
            response.take::<Option<Vec<Thing>>>((0, "task_lists")).map_err(get_io_error)?.unwrap_or_default();
        task_lists.extend(response.take::<Option<Vec<Thing>>>((1, "task_lists")).map_err(get_io_error)?.unwrap_or_default());

        Ok(unwrap_things(task_lists))
    }

    pub async fn query_notif_by_user_id(&self, user_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT ->notified_by->notification as notifs FROM user where id == user:{}",
                user_id
            ),
        )
        .await?;
        let notifs = response
            .take::<Option<Vec<Thing>>>((0, "notifs"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(notifs))
    }
}
