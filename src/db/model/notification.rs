use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<Thing>,
    pub title: String,
    pub content: String,
    pub handled: bool,
}