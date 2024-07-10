use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct InvitationTokenRepository {
    pub tokens: HashMap<String, InvitationInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InvitationInfo {
    pub inviter: String,
    pub invitee: String,
    pub project: String,
}
