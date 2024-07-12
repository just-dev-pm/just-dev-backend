use crate::db::repository::{agenda::AgendaRepository, notification::NotificationRepository, task::TaskRepository};

use super::util::notification::{assigned_event_to_notif, assigned_task_to_notif, deassign_event_to_notif, deassign_task_to_notif};

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
    agenda_repo._assign_event_for_user(event_id, user_id).await?;
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
    agenda_repo._deassign_event_for_user(event_id, user_id).await?;
    let event = agenda_repo.query_event_by_id(event_id).await?;
    let _ = notif_repo
        .insert_notif(user_id, event_id, "event", deassign_event_to_notif(event))
        .await?;
    Ok(())
}
