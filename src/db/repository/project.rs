use futures::io::BufReader;
use futures::TryFutureExt;
use octocrate::{APIConfig, AppAuthorization, GitHubAPI, PersonalAccessToken, PullRequestSimple};
use surrealdb::sql::Thing;

use crate::api::model::pr::PullRequest;
use crate::db::db_context::DbContext;
use crate::db::model::draft::DraftWithoutContent;
use crate::db::model::project::Project;
use crate::db::model::user::User;
use std::fs::{self, File};
use std::io;
use std::sync::Arc;

use crate::db::repository::utils::*;
#[derive(Clone)]
pub struct ProjectRepository {
    github_api: Arc<GitHubAPI>,
    context: DbContext,
}

impl ProjectRepository {
    pub async fn new() -> ProjectRepository {
        ProjectRepository {
            github_api: {
                let app_id = std::env::var("JUST_DEV_GITHUB_APP_ID")
                    .expect("JUST_DEV_GITHUB_APP_ID must be set");
                let app_private_key = std::env::var("JUST_DEV_GITHUB_APP_PRIVATE_KEY")
                    .expect("JUST_DEV_GITHUB_APP_PRIVATE_KEY must be set");
                let app_private_key =
                    fs::read_to_string(app_private_key).expect("Read private key file failed");
                let config =
                    APIConfig::with_token(AppAuthorization::new(app_id, app_private_key)).shared();
                Arc::new(GitHubAPI::new(&config))
            },
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
                "(SELECT <-user.* as users from join where out.id == project:{id} AND admin == false).users"
            ))
            .await
            .unwrap();
        let members = response
            .take::<Vec<Vec<User>>>(0)
            .map_err(get_io_error)?
            .into_iter()
            .filter_map(|mut user| user.pop())
            .collect::<Vec<_>>(); //TODO: add error handling
        Ok(members)
    }

    pub async fn query_agenda_by_id(&self, project_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "select ->own->agenda as agendas from project where id == project:{project_id}"
            ),
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
                "select ->own->task_list as task_lists from project where id == project:{project_id}"
            ),
        )
        .await?;
        let task_lists: Option<Vec<Thing>> =
            response.take((0, "task_lists")).map_err(get_io_error)?;

        Ok(unwrap_things(task_lists.unwrap_or_default()))
    }

    pub async fn query_draft_by_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<DraftWithoutContent>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("for $draft in (select ->own->draft as drafts from project where id == project:{project_id}).drafts {{return select id, name from $draft}}"),
        )
        .await?;
        let agendas = response
            .take::<Vec<DraftWithoutContent>>(0)
            .map_err(get_io_error)?;

        Ok(agendas)
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

    pub async fn query_prs_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<PullRequestSimple>, io::Error> {
        let project = self.query_project_by_id(project_id).await?;
        if project.github == 0 {
            return Ok(vec![])
        }
        let installation_token: octocrate::InstallationToken = self
            .github_api
            .apps
            .create_installation_access_token(project.github)
            .send()
            .map_err(get_io_error)
            .await?;

        let api = GitHubAPI::new(
            &APIConfig::with_token(PersonalAccessToken::new(installation_token.token)).shared(),
        );
        let repos = api
            .apps
            .list_repos_accessible_to_installation()
            .send()
            .await
            .map_err(get_io_error)?
            .repositories;

        let mut prs = vec![];
        for repo in repos {
            let prs_in_repo = api
                .pulls
                .list(repo.owner.login, repo.name)
                .send()
                .await
                .map_err(get_io_error)?;
            prs.extend(prs_in_repo);
        }

        Ok(prs)
    }
}
