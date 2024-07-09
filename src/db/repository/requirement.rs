use std::io;

use crate::db::{db_context::DbContext, model::requirement::Requirement};

use super::utils::get_io_error;



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

    pub async fn query_requ_by_id(&self, requ_id:&str) -> Result<Requirement, io::Error> {
        let requ: Option<Requirement> = self
            .context
            .db
            .select(("requirement", requ_id))
            .await
            .map_err(get_io_error)?;
        requ.ok_or(io::Error::new(io::ErrorKind::NotFound, "Requirement not found"))
    }
}

