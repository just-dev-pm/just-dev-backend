use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use crate::{api::model::pr::PullRequest, db::repository::utils::DbModelId};



#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Task {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    #[serde(rename = "status")]
    pub status: String,
    pub complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ddl: Option<Datetime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<DbModelId>>,
    pub pr: PullRequest, 
    pub pr_number: i64,
    pub pr_assigned: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TaskLink {
    pub id: Option<Thing>,
    #[serde(rename = "in")]
    pub incoming: Option<Thing>,
    #[serde(rename = "out")]
    pub outgoing: Option<Thing>,
    #[serde(rename = "type")]
    pub kind: String,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskList {
    pub id: Option<Thing>,
    pub name: String,
    pub tasks: Option<Vec<DbModelId>>,
    pub owner: Option<DbModelId>,
}

impl Task {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            description: "".to_string(),
            status: "0".to_owned(),
            complete: false,
            ddl: None,
            assignees: None,
            pr_number: 0,
            pr_assigned: false,
            pr: PullRequest::default(),
        }
    }
}

impl TaskList {
    pub fn new(name: String) -> Self  {
        Self {
            id: None,
            name,
            tasks: None,
            owner: None,
        }
    }

    pub fn new_with_id(name: String, id: &str, table: &str) -> Self  {
        Self {
            id: Some(Thing::from((table, id))),
            name,
            tasks: None,
            owner: None,
        }
    }
}

