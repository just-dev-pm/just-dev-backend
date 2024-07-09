use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Duration, Thing};

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
    datetime: Option<Datetime>,
    duration: Option<Duration>,
}