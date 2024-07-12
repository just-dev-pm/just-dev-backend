use axum_ycrdt_websocket::{broadcast::BroadcastGroup, AwarenessRef};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use yrs::{sync::Awareness, updates::decoder::Decode, Doc, Options, Transact, Update};

use crate::db::{model::draft::DraftPayload, repository::draft::DraftRepository};

#[derive(Clone)]
pub struct DraftCollaborationManager {
    pub rooms: HashMap<String, Arc<BroadcastGroup>>,
}

impl DraftCollaborationManager {
    pub async fn get_room(
        &mut self,
        room_id: String,
        draft_repo: &DraftRepository,
    ) -> Option<Arc<BroadcastGroup>> {
        match self.rooms.get(&room_id) {
            Some(bcast) => Some(bcast.clone()),
            None => match draft_repo.query_draft_by_id(&room_id).await {
                Ok(draft_payload) => {
                    let bcast = new_room_for_draft_payload(&draft_payload).await;
                    self.rooms.insert(room_id, bcast.clone());

                    Some(bcast)
                }
                Err(_) => return None,
            },
        }
    }

    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

async fn new_room_for_draft_payload(draft_payload: &DraftPayload) -> Arc<BroadcastGroup> {
    let awareness: AwarenessRef = {
        let doc = Doc::with_options(Options {
            skip_gc: true,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            txn.apply_update(Update::decode_v1(&draft_payload.content).unwrap());
        }
        Arc::new(RwLock::new(Awareness::new(doc)))
    };
    Arc::new(BroadcastGroup::new(awareness.clone(), 32).await)
}
