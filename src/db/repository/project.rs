use crate::db::db_context::DbContext;
use crate::db::model::project::Project;
use crate::db::model::user::User;
use std::io;

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
        let result: Option<Project> = self
            .context
            .db
            .create("project")
            .content(project)
            .await
            .unwrap()
            .pop();
        result.ok_or(io::Error::new(io::ErrorKind::NotFound, "Project insert fail"))
    }


    pub async fn set_user_for_project() {

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
}
