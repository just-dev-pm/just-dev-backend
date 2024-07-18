use octocrate::PullRequestSimple;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct PullRequest {
    pub owner: String,
    pub repo: String, 
    pub pull_number: i64,
    pub title: String,
}


impl From<PullRequestSimple> for PullRequest {
    fn from(pr: PullRequestSimple) -> Self {
        Self {
            owner: match pr.user {
                None => "null".to_owned(),
                Some(user) => user.login,
            },
            repo: pr.base.repo.name,
            pull_number: pr.number,
            title: pr.title,
        }
    }
}   

