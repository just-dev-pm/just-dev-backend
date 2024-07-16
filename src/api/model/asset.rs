use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "source")]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    Task { path: TaskPath },
    Draft { path: DraftPath },
    Event { path: EventPath },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskPath {
    pub task_id: String,
    pub task_list_id: String,
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DraftPath {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EventPath {
    pub event_id: String,
    pub agenda_id: String,
    pub project_id: String,
}
