use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use crate::db::repository::utils::DbModelId;

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
    name: String,
    description: String,
    start_time: Option<Datetime>,
    end_time: Option<Datetime>,
}