use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PullRequest {
    pub owner: String,
    pub repo: String, 
    pub pull_number: i64,
    pub title: String,
}

