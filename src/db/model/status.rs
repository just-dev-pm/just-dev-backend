use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;


#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Status {
    pub id: Option<Thing>,
    pub name: String,
    pub description: String,
    pub number: Option<i32>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct StatusPool {
    pub id: Option<Thing>,
    pub incomplete: Vec<Status>,
    pub complete: Status,
}