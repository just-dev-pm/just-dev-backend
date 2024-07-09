use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use super::status::StatusPool;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<Thing>,
    pub name: String,
    pub avatar: Option<String>,
    pub status_pool: StatusPool
}


