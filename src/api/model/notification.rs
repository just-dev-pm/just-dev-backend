use serde::{Deserialize, Serialize};

use super::asset::Asset;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub content: String,
    pub asset: Asset,
    pub handled: bool,
}
