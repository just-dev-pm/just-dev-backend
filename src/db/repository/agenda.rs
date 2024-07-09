use std::io;

use crate::db::{db_context::DbContext, model::agenda::Agenda};

use super::utils::get_io_error;



#[derive(Clone)]
pub struct AgendaRepository {
    pub context: DbContext,
}

impl AgendaRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_agenda_by_id(&self, id: &str) -> Result<Agenda, io::Error> {
        let agenda: Option<Agenda> = self
            .context
            .db
            .select(("agenda", id))
            .await
            .map_err(get_io_error)?;
        agenda.ok_or(io::Error::new(io::ErrorKind::NotFound, "Agenda not found"))
    }
}

