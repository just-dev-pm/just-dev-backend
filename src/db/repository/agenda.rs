use std::io;

use surrealdb::sql::{Id, Thing};

use crate::db::{
    db_context::DbContext,
    model::agenda::{Agenda, Event},
};

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

    pub async fn query_events_by_agenda_id(
        &self,
        agenda_id: &str,
    ) -> Result<Vec<Event>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "(SELECT ->plan->event.* as events FROM agenda WHERE id == agenda:{}).events",
                agenda_id
            ),
        )
        .await?;
        let events = response
            .take::<Option<Vec<Event>>>(0)
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(events)
    }

    pub async fn query_event_id_by_agenda_id(
        &self,
        agenda_id: &str,
    ) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "(SELECT ->plan->event as events FROM agenda WHERE id == agenda:{}).events",
                agenda_id
            ),
        )
        .await?;
        let events = response
            .take::<Option<Vec<Thing>>>(0)
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(events))
    }

    pub async fn query_agenda_by_id(&self, id: &str) -> Result<Agenda, io::Error> {
        select_resourse(&self.context, id, "agenda").await
    }

    pub async fn delete_agenda(&self, agenda_id: &str) -> Result<Agenda, io::Error> {
        delete_resource(&self.context, agenda_id, "agenda").await
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<Event, io::Error> {
        let _ = exec_query(
            &self.context,
            format!(
                "for $follow_event in (select <-event_follow<-event as agenda from event where id == event:{}).agenda {{delete $follow_event;}}",
                event_id
            ),
        ).await?;
        delete_resource(&self.context, event_id, "event").await
    }

    pub async fn insert_agenda_for_user(
        &self,
        user_id: &str,
        name: &str,
    ) -> Result<Agenda, io::Error> {
        let agenda = Agenda::new(name.to_owned());

        let agenda = create_resource(&self.context, &agenda, "agenda").await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate user:{user_id} -> own -> agenda:{}",
                unwrap_thing(agenda.id.clone().unwrap())
            ),
        )
        .await?;
        Ok(agenda)
    }

    pub async fn insert_exagenda_for_user(
        &self,
        name: &str,
        user_id: &str,
    ) -> Result<Agenda, io::Error> {
        let mut agenda = Agenda::new(name.to_owned());
        agenda.id = Some(Thing {
            tb: "agenda".to_owned(),
            id: Id::String(user_id.to_owned()),
        });
        let agenda = create_resource(&self.context, &agenda, "agenda").await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate user:{user_id} -> own -> agenda:{}",
                agenda.id.as_ref().unwrap()
            ),
        )
        .await?;
        Ok(agenda)
    }

    pub async fn insert_agenda_for_project(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<Agenda, io::Error> {
        let agenda = Agenda::new(name.to_owned());
        let agenda = create_resource(&self.context, &agenda, "agenda").await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate project:{project_id} -> own -> agenda:{}",
                unwrap_thing(agenda.id.clone().unwrap())
            ),
        )
        .await?;
        Ok(agenda)
    }

    pub async fn insert_event_for_agenda(
        &self,
        event: &Event,
        agenda_id: &str,
    ) -> Result<Event, io::Error> {
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

    pub async fn query_event_by_id(&self, event_id: &str) -> Result<Event, io::Error> {
        select_resourse(&self.context, event_id, "event").await
    }

    pub async fn query_assignees_of_event(
        &self,
        event_id: &str,
    ) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "(SELECT <-event_follow<-event<-plan<-agenda<-own<-user as assignees FROM event WHERE id == event:{}).assignees",
                event_id
            ),
        )
        .await?;
        let assignees = response
            .take::<Option<Vec<Thing>>>(0)
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(assignees))
    }

    pub async fn deassign_event_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Event, io::Error> {
        let mut response = exec_double_query(
            &self.context, 
            format!("(select <-event_follow<-event as events from event where id == event:{event_id}).events"), 
            format!("(select ->plan->event as assigned from agenda where id == agenda:{user_id}).assigned")).await?;
        let events = unwrap_things(response
            .take::<Option<Vec<Thing>>>(0)
            .map_err(get_io_error)?
            .unwrap_or_default());
        let user_assigned = unwrap_things(response.take::<Option<Vec<Thing>>>(1).map_err(get_io_error)?.unwrap_or_default());
        for event in events {
            if user_assigned.contains(&event) {
                return Ok(delete_resource::<Event>(&self.context, &event, "event").await?);
            }
        }
        Err(custom_io_error("Assigning relation not found"))

    }

    pub async fn assign_event_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Event, io::Error> {
        let mut event = self.query_event_by_id(event_id).await?;
        event.id = None;
        let event = self.insert_event_for_agenda(&event, user_id).await?; // insert task into user's special tasklist

        let _ = self
            .context
            .db
            .query(format!(
                "relate event:{} -> event_follow -> event:{}",
                unwrap_thing(event.id.clone().unwrap()),
                event_id
            ))
            .await
            .map_err(|e| get_io_error(e))?;
        Ok(event)
    }
}
