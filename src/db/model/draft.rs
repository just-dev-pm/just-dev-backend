use serde::{Deserialize, Serialize};
use surrealdb::sql::{Bytes, Thing};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Draft {
    pub id: Option<Thing>,
    pub name: String,
    pub content: Bytes,
}