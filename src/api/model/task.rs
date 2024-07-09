use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{status::Status, util::Id};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assignees: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskRelation {
    pub id: String,
    pub from: Id,
    pub to: Id,
    #[serde(flatten)]
    pub category: TaskRelationType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "category")]
#[serde(rename_all = "snake_case")]
pub enum TaskRelationType {
    Auto,
    Dep,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskList {
    pub id: String,
    pub name: String,
    pub tasks: Vec<Id>,
}
