use std::io;

use crate::db::{db_context::DbContext, model::notification::Notification};

use super::utils::get_io_error;

#[derive(Clone)]
pub struct NotificationRepository {
    pub context: DbContext,
}

impl NotificationRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_notif_by_id(&self, id: &str) -> Result<Notification, io::Error> {
        let notif: Option<Notification> = self
            .context
            .db
            .select(("notification", id))
            .await
            .map_err(get_io_error)?;
        notif.ok_or(io::Error::new(io::ErrorKind::NotFound, "Notification not found"))
    }

    pub async fn handle_notif_by_id(&self, id: &str) -> Result<Notification, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!("update notification set handled = true where id == notification:{id}"))
            .await
            .map_err(get_io_error)?;
        let notif: Option<Notification> = response.take(0).map_err(get_io_error)?;
        notif.ok_or(io::Error::new(io::ErrorKind::NotFound, "Notification not found"))
    }
    
}

