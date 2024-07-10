use std::io;

use serde_json::json;

use crate::db::{
    db_context::DbContext,
    model::draft::{Draft, DraftPayload},
};

use super::utils::{custom_io_error, get_io_error};

#[derive(Clone)]
pub struct DraftRepository {
    pub context: DbContext,
}

impl DraftRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_draft_by_id(&self, draft_id: &str) -> Result<DraftPayload, io::Error> {
        let draft: Option<Draft> = self
            .context
            .db
            .select(("draft", draft_id))
            .await
            .map_err(get_io_error)?;
        DraftPayload::from(draft.ok_or(io::Error::new(io::ErrorKind::NotFound, "Draft not found"))?)
    }

    pub async fn insert_draft_for_user(&self, name: &str, user_id: &str) -> Result<DraftPayload, io::Error> {
        let draft = Draft::new(name.to_string(), &vec![]);
        let result: Option<Draft> = self
            .context
            .db
            .create("draft")
            .content(&draft)
            .await
            .map_err(get_io_error)?
            .pop();
        let draft = DraftPayload::from(result.ok_or(custom_io_error("draft query failed"))?)?;
        let _ = self.context.db.query(format!("relate user:{user_id} -> own -> draft:{}", draft.id.as_ref().unwrap())).await.map_err(get_io_error)?;   

        Ok(draft)
    }

    pub async fn insert_draft_for_project(&self, name: &str, project_id: &str) -> Result<DraftPayload, io::Error> {
        let draft = Draft::new(name.to_string(), &vec![]);
        let result: Option<Draft> = self
            .context
            .db
            .create("draft")
            .content(&draft)
            .await
            .map_err(get_io_error)?
            .pop();
        let draft = DraftPayload::from(result.ok_or(custom_io_error("draft query failed"))?)?;
        let _ = self.context.db.query(format!("relate project:{project_id} -> own -> draft:{}", draft.id.as_ref().unwrap())).await.map_err(get_io_error)?;   

        Ok(draft)
    }



    pub async fn update_draft(&self, draft: DraftPayload) -> Result<DraftPayload, io::Error> {
        let result: Option<Draft> = self
            .context
            .db
            .update((
                "draft",
                draft
                    .id
                    .as_ref()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Draft id not found"))?,
            ))
            .content(Draft::from(draft))
            .await
            .map_err(get_io_error)?;
        DraftPayload::from(result.ok_or(io::Error::new(io::ErrorKind::Other, "Draft update fail"))?)
    }

}
