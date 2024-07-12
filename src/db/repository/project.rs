use surrealdb::sql::Thing;

use crate::db::db_context::DbContext;
use crate::db::model::project::Project;
use crate::db::model::user::User;
use std::io;

use crate::db::repository::utils::*;
#[derive(Clone)]
pub struct ProjectRepository {
    context: DbContext,
}

impl ProjectRepository {
    pub async fn new() -> ProjectRepository {
        ProjectRepository {
            context: DbContext::new().await,
        }
    }

    pub async fn query_project_by_id(&self, id: &str) -> Result<Project, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!("SELECT * FROM project WHERE id == project:{}", id)) // TODO: avoid risk of sql injection
            .await
            .unwrap();

        let project: Option<Project> = response.take(0).unwrap();

        project.ok_or(io::Error::new(io::ErrorKind::NotFound, "Project not found"))
    }

    pub async fn insert_project(&self, project: &Project) -> Result<Project, io::Error> {
        create_resource(&self.context, project, "project").await
    }

    pub async fn update_project(
        &self,
        project: &Project,
        project_id: &str,
    ) -> Result<Project, io::Error> {
        let result: Option<Project> = self
            .context
            .db
            .update(("project", project_id))
            .content(project)
            .await
            .unwrap();
        result.ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Project update fail",
        ))
    }

    pub async fn set_user_for_project(
        &self,
        user_id: &str,
        project_id: &str,
        admin: bool,
    ) -> Result<(), surrealdb::Error> {
        let _ = self
            .context
            .db
            .query(format!(
                "relate user:{user_id} -> join -> project:{project_id} set admin = {admin}"
            ))
            .await?;
        Ok(())
    }

    // admin can't be deleted
    pub async fn delete_user_from_project(
        &self,
        user_id: &str,
        project_id: &str,
    ) -> Result<(), surrealdb::Error> {
        let _ = self.context
            .db
            .query(format!("delete join where in == user:{user_id} and out == project:{project_id} and admin == false"))
            .await?;
        Ok(())
    }

    pub async fn query_admin_by_id(&self, id: &str) -> Result<User, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT in.* FROM join WHERE out.id == project:{} AND admin == true",
                id
            ))
            .await
            .unwrap();
        let admin: Option<User> = response.take((0, "in")).unwrap();
        admin.ok_or(io::Error::new(io::ErrorKind::NotFound, "Admin not found"))
    }

    pub async fn query_members_by_id(&self, id: &str) -> Result<Vec<User>, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT in.* FROM join WHERE out.id == project:{} AND admin == false",
                id
            ))
            .await
            .unwrap();
        let members: Vec<User> = response.take((0, "in")).unwrap(); //TODO: add error handling
        Ok(members)
    }

    pub async fn query_agenda_by_id(&self, project_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->own->agenda as agendas from user where id == project:{project_id}"),
        )
        .await?;
        let agendas: Option<Vec<Thing>> = response.take((0, "agendas")).map_err(get_io_error)?;

        Ok(unwrap_things(agendas.unwrap_or_default()))
    }

    pub async fn query_task_list_by_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "select ->own->task_list as task_lists from user where id == project:{project_id}"
            ),
        )
        .await?;
        let task_lists: Option<Vec<Thing>> =
            response.take((0, "task_lists")).map_err(get_io_error)?;

        Ok(unwrap_things(task_lists.unwrap_or_default()))
    }

    pub async fn query_draft_by_id(&self, project_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->own->draft as drafts from project where id == project:{project_id}"),
        )
        .await?;
        let agendas: Option<Vec<Thing>> = response.take((0, "drafts")).map_err(get_io_error)?;

        Ok(unwrap_things(agendas.unwrap_or_default()))
    }

    pub async fn query_requ_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("select ->require->requirement as requs from project where id == project:{project_id}"),
        )
        .await?;
        let notifs = response
            .take::<Option<Vec<Thing>>>((0, "requs"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(notifs))
    }
}
