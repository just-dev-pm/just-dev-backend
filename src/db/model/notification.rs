use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;



pub enum NotificationSource {
    Task(String),
    Event(String),
    Draft(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<Thing>,
    pub title: String,
    pub content: String,
    pub handled: bool,
}

impl Notification {
    pub fn new(title: String, content: String) -> Self {
        Notification {
            id: None,
            title,
            content,
            handled: false,
        }
    }
}