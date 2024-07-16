use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::db::repository::utils::DbModelId;


pub struct AssetPath(pub String, pub(String, String));

pub enum NotificationSource {
    Task(AssetPath),
    Event(AssetPath),
    Draft(AssetPath),
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