use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Requirement {
    id: Option<Thing>,
    name: String,
    description: String,
}