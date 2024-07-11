use std::io;


use crate::db::{db_context::DbContext, model::requirement::Requirement};

use super::utils::{
    create_resource, delete_resource, exec_query, get_io_error,
    unwrap_thing, update_resource,
};

#[derive(Clone)]
pub struct RequirementRepository {
    pub context: DbContext,
}

impl RequirementRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_requ_by_id(&self, requ_id: &str) -> Result<Requirement, io::Error> {
        let requ: Option<Requirement> = self
            .context
            .db
            .select(("requirement", requ_id))
            .await
            .map_err(get_io_error)?;
        requ.ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Requirement not found",
        ))
    }

    pub async fn insert_requ_for_project(
        &self,
        project_id: &str,
        name: String,
        description: String,
    ) -> Result<Requirement, io::Error> {
        let requ = Requirement::new(name, description);
        let requ = create_resource(&self.context, &requ, "requirement").await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate project:{project_id} -> require -> requirement:{}",
                unwrap_thing(requ.id.clone().unwrap())
            ),
        )
        .await?;
        Ok(requ)
    }

    pub async fn delete_requ_from_project(&self, requ_id: &str) -> Result<Requirement, io::Error> {
        delete_resource::<Requirement>(&self.context, requ_id, "requirement").await
    }

    pub async fn update_requ(
        &self,
        requ_id: &str,
        requ: &Requirement,
    ) -> Result<Requirement, io::Error> {
        update_resource(&self.context, requ_id, requ, "requirement").await
    }
}
