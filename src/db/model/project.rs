use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    id: Option<Thing>,
    pub name: String,
}

