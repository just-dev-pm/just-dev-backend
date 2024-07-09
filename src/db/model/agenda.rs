use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Agenda {
    id: Option<Thing>,
    name: String,
    events: Option<Vec<Event>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    id: Option<Thing>,
    name: String,
    description: String,
    start_time: Option<Datetime>,
    end_time: Option<Datetime>,
}