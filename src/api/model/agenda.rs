use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::util::Id;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub participants: Vec<Id>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Agenda {
    pub id: String,
    pub name: String,
    pub events: Vec<Id>,
}
