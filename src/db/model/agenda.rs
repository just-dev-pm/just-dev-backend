use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use crate::{api::handler::event::CreateEventForAgendaRequest, db::repository::utils::DbModelId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Agenda {
    pub id: Option<Thing>,
    pub name: String,
    pub events: Option<Vec<DbModelId>>,
}

impl Agenda {
    pub fn new(name: String) -> Self {
        Agenda {
            id: None,
            name,
            events: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    pub start_time: Datetime,
    pub end_time: Datetime,
}

impl Event {
    pub fn new(name: String, description: String) -> Self {
        Event {
            id: None,
            name,
            description,
            start_time: Datetime::default(),
            end_time: Datetime::default(),
        }
    }

    pub fn from_create_request(req: CreateEventForAgendaRequest) -> Self {
        Event {
            id: None,
            name: req.name,
            description: req.description,
            start_time: Datetime(req.start_time),
            end_time: Datetime(req.end_time),
        }
    }
}