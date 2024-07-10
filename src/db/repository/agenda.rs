use std::io;

use crate::db::{db_context::DbContext, model::agenda::{Agenda, Event}};

use crate::db::repository::utils::*;

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
        select_resourse(&self.context, id, "agenda").await
    }

    pub async fn insert_agenda_for_user(
        &self,
        user_id: &str,
        name: &str,
    ) -> Result<Agenda, io::Error> {
        let agenda = Agenda::new(name.to_owned());

        let agenda = create_resource(&self.context, &agenda, "agenda")
            .await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate user:{user_id} -> own -> agenda:{}",
                agenda.id.as_ref().unwrap()
            ),
        );
        Ok(agenda)
    }

    pub async fn insert_agenda_for_project(
        &self,
        name: &str,
        project_id: &str,
    ) -> Result<Agenda, io::Error> {
        let agenda = Agenda::new(name.to_owned());
        let agenda = create_resource(&self.context, &agenda, "agenda")
            .await?;
        let _ = exec_query(
                &self.context,
                format!(
                    "relate project:{project_id} -> own -> agenda:{}",
                    agenda.id.as_ref().unwrap().id.to_string()
                ),
            )
            .await?;
        Ok(agenda)
    }

    pub async fn insert_event_for_agenda(&self, event: &Event, agenda_id: &str) -> Result<Event, io::Error> {
        let event = create_resource(&self.context, event, "event").await?;
        let _ = exec_query(
                &self.context,
                format!(
                    "relate agenda:{agenda_id} -> plan -> event:{}",
                    event.id.as_ref().unwrap().id.to_string()
                ),
            )
            .await?;
        Ok(event)
    }
}
