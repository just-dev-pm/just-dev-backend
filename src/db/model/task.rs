use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use super::{status::StatusPool, user::User};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    pub status_number: i32,
    pub status_pool: Option<StatusPool>,
    pub complete: bool,
    pub ddl: Option<Datetime>,
    pub assignees: Option<Vec<User>>,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskList {
    pub id: Option<Thing>,
    pub name: String,
    pub tasks: Option<Vec<Task>>,
    pub owner: Option<User>,
}

