use std::io;

use base64_lib::{decode, encode};
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Draft {
    pub id: Option<Thing>,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPayload {
    pub id: Option<String>,
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftWithoutContent {
    pub id: Option<Thing>,
    pub name: String,
}


impl Draft {
    pub fn new(name: String, content: &Vec<u8>) -> Self {
        Draft {
            id: None,
            name,
            content: encode(content),
        }
    }

    pub fn new_with_id(id: &str, name: String, content: &Vec<u8>) -> Self {
        Draft {
            id: Some(Thing {
                tb: "draft".to_string(),
                id: Id::String(id.to_string()),
            }),
            name,
            content: encode(content),
        }
    }

    pub fn get_content(&self) -> Vec<u8> {
        decode(&self.content)
    }

    pub fn from(draft: DraftPayload) -> Draft {
        match draft.id {
            Some(id) => Self::new_with_id(&id, draft.name, &draft.content),
            None => Self::new(draft.name, &draft.content),
        }
    }
}

impl DraftPayload {
    pub fn from(draft: Draft) -> Result<DraftPayload, io::Error> {
        Ok(DraftPayload {
            id: match draft.id {
                Some(id) => Some(id.id.to_string()),
                None => None,
            },
            name: draft.name,
            content: decode(&draft.content),
        })
    }

    pub fn new(name: String, content: Vec<u8>) -> Self {
        DraftPayload {
            id: None,
            name,
            content,
        }
    }
}
