use std::io;

use surrealdb::sql::Thing;

use crate::db::{
    db_context::DbContext,
    model::notification::{Notification, NotificationSource},
};

use super::utils::{create_resource, custom_io_error, exec_query, get_io_error, unwrap_thing};

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

    pub async fn query_notif_by_id(
        &self,
        id: &str,
    ) -> Result<(Notification, NotificationSource), io::Error> {
        let notif: Option<Notification> = self
            .context
            .db
            .select(("notification", id))
            .await
            .map_err(get_io_error)?;
        let mut response = exec_query(
            &self.context,
            format!(
                "select <-about<-notification as source from notification where id == notification:{}",
                id
            ),
        )
        .await?;
        let source = response
            .take::<Option<Thing>>((0, "source"))
            .map_err(get_io_error)?
            // .unwrap_or_default();
            // .pop()
            .ok_or(custom_io_error("Notification source find failed"))?;
        let source = match source.tb.as_str() {
            "task" => NotificationSource::Task(source.id.to_string()),
            "event" => NotificationSource::Event(source.id.to_string()),
            "draft" => NotificationSource::Draft(source.id.to_string()),
            _ => NotificationSource::Task(source.id.to_string()),
        };
        Ok((notif.ok_or(custom_io_error("Notification find failed"))?, source))
    }

    pub async fn handle_notif_by_id(&self, id: &str) -> Result<Notification, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!(
                "update notification set handled = true where id == notification:{id}"
            ))
            .await
            .map_err(get_io_error)?;
        let notif: Option<Notification> = response.take(0).map_err(get_io_error)?;
        notif.ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Notification not found",
        ))
    }

    pub async fn insert_notif(
        &self,
        title: String,
        content: String,
        user_id: &str,
        about_table: &str,
        about_id: &str,
    ) -> Result<Notification, io::Error> {
        let notif = create_resource(
            &self.context,
            &Notification::new(title, content),
            "notification",
        )
        .await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate user:{user_id} -> notified_by -> notification:{}",
                unwrap_thing(notif.id.clone().unwrap())
            ),
        )
        .await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate notification:{notif_id} -> about -> {about_table}:{about_id}",
                notif_id = unwrap_thing(notif.id.clone().unwrap()),
                about_table = about_table,
                about_id = about_id,
            ),
        )
        .await?;
        Ok(notif)
    }
}
