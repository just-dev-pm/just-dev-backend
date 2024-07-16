use std::io;

use surrealdb::sql::Thing;

use crate::db::{
    model::notification::{AssetPath, Notification, NotificationSource},
    repository::{
        agenda::AgendaRepository,
        draft::DraftRepository,
        notification::NotificationRepository,
        task::TaskRepository,
        utils::{custom_io_error, exec_query, get_io_error},
    },
};

use super::util::notification::{
    assigned_event_to_notif, assigned_task_to_notif, deassign_event_to_notif,
    deassign_task_to_notif,
};

pub async fn assign_task_to_user(
    task_repo: &TaskRepository,
    notif_repo: &NotificationRepository,
    task_id: &str,
    user_id: &str,
) -> Result<(), std::io::Error> {
    task_repo._assign_task_to_user(task_id, user_id).await?;
    let task = task_repo.query_task_by_id(task_id).await?;
    let _ = notif_repo
        .insert_notif(user_id, task_id, "task", assigned_task_to_notif(task))
        .await?;
    Ok(())
}

pub async fn deassign_task_for_user(
    task_repo: &TaskRepository,
    notif_repo: &NotificationRepository,
    task_id: &str,
    user_id: &str,
) -> Result<(), std::io::Error> {
    task_repo._deassign_task_for_user(task_id, user_id).await?;
    let task = task_repo.query_task_by_id(task_id).await?;
    let _ = notif_repo
        .insert_notif(user_id, task_id, "task", deassign_task_to_notif(task))
        .await?;
    Ok(())
}

pub async fn assign_event_for_user(
    agenda_repo: &AgendaRepository,
    notif_repo: &NotificationRepository,
    event_id: &str,
    user_id: &str,
) -> Result<(), std::io::Error> {
    agenda_repo
        ._assign_event_for_user(event_id, user_id)
        .await?;
    let event = agenda_repo.query_event_by_id(event_id).await?;
    let _ = notif_repo
        .insert_notif(user_id, event_id, "event", assigned_event_to_notif(event))
        .await?;
    Ok(())
}

pub async fn deassign_event_for_user(
    agenda_repo: &AgendaRepository,
    notif_repo: &NotificationRepository,
    event_id: &str,
    user_id: &str,
) -> Result<(), std::io::Error> {
    agenda_repo
        ._deassign_event_for_user(event_id, user_id)
        .await?;
    let event = agenda_repo.query_event_by_id(event_id).await?;
    let _ = notif_repo
        .insert_notif(user_id, event_id, "event", deassign_event_to_notif(event))
        .await?;
    Ok(())
}

pub async fn query_notif_by_id(
    notif_repo: &NotificationRepository,
    task_repo: &TaskRepository,
    agenda_repo: &AgendaRepository,
    // draft_repo: &DraftRepository,
    id: &str,
) -> Result<(Notification, NotificationSource), io::Error> {
    let notif: Option<Notification> = notif_repo
        .context
        .db
        .select(("notification", id))
        .await
        .map_err(get_io_error)?;
    let mut response = exec_query(
        &notif_repo.context,
        format!(
            "select ->about.out as source from notification where id == notification:{}",
            id
        ),
    )
    .await?;
    let source = response
        .take::<Option<Vec<Thing>>>((0, "source"))
        .map_err(get_io_error)?
        .unwrap_or_default()
        .pop()
        .ok_or(custom_io_error("Notification source find failed"))?;
    let source_id = source.id.clone().to_string();
    let source = match source.tb.as_str() {
        "task" => NotificationSource::Task(AssetPath(
            source_id.to_owned(),
            task_repo.query_task_path_by_id(&source_id).await?,
        )),
        "event" => NotificationSource::Event(AssetPath(
            source_id.to_owned(),
            agenda_repo.query_event_path_by_id(&source_id).await?,
        )),
        // "draft" => NotificationSource::Draft(source.id.to_string()),
        _ => NotificationSource::Task(AssetPath(
            source_id.to_owned(),
            task_repo.query_task_path_by_id(&source_id).await?,
        )),
    };
    Ok((
        notif.ok_or(custom_io_error("Notification find failed"))?,
        source,
    ))
}
