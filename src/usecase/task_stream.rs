
use crate::db::repository::{task::TaskRepository, utils::unwrap_thing};
use std::io::{self};

pub async fn refresh_task_status(
    task_id: &String,
    task_repo: &TaskRepository,
) -> Result<(), std::io::Error> {
    // get all pre tasks
    // strategy:
    // 1. status between incomplete status can be changed freely
    // 2. if any pre task is incomplete, task should be incomplete
    // 3. if all pre tasks are auto and complete, task should be complete
    // 4, task is update
    // if task status is changed between incomplete and complete then tasks dependent should be refreshed

    let db_task = task_repo.query_task_by_id(&task_id).await?;
    let pre_tasks_links = task_repo
        .query_task_incoming_links_by_task_id(task_id)
        .await?;
    let dep_tasks: Vec<_> = pre_tasks_links
        .clone()
        .into_iter()
        .filter(|link| link.kind.eq("dep"))
        .collect();
    let mut new_task = db_task.clone();
    for dep_task in &dep_tasks {
        let pre_task = task_repo
            .query_task_by_id(&unwrap_thing(dep_task.to_owned().incoming.unwrap()))
            .await?;
        if pre_task.complete == false {
            new_task.complete = false;
            if new_task.complete != db_task.complete {
                task_repo.update_task_by_id(&task_id, &new_task).await?;
                Box::pin(async move {
                    refresh_task_status_entry(task_id, task_repo).await?;
                    Ok::<_, io::Error>(())
                })
                .await?;
            }
            return Ok(());
        }
    }

    if dep_tasks.len() == 0 && pre_tasks_links.len() > 0 {
        if !task_repo
            .task_links_to_tasks(pre_tasks_links)
            .await?
            .into_iter()
            .all(|task| task.complete == true)
        {
            new_task.complete = false;
            if db_task.complete != new_task.complete {
                task_repo.update_task_by_id(&task_id, &new_task).await?;
                Box::pin(async move {
                    refresh_task_status_entry(task_id, task_repo).await?;
                    Ok::<_, io::Error>(())
                })
                .await?;
            }
            return Ok(());
        }

        new_task.complete = true;
        if db_task.complete != new_task.complete {
            task_repo.update_task_by_id(&task_id, &new_task).await?;
            Box::pin(async move {
                refresh_task_status_entry(task_id, task_repo).await?;
                Ok::<_, io::Error>(())
            })
            .await?;
        }
    }

    Ok(())
}

pub async fn refresh_task_status_entry(
    task_id: &str,
    task_repo: &TaskRepository,
) -> Result<(), io::Error> {
    let outgoings = task_repo
        .query_task_outgoing_links_by_task_id(task_id)
        .await?;
    for link in outgoings {
        refresh_task_status(&unwrap_thing(link.outgoing.unwrap()), task_repo).await?;
    }
    Ok(())
}

pub enum TaskSwitchable {
    False,
    True,
    TrueAndFalse,
}

pub async fn check_task_switch_complete(
    task_id: &str,
    task_repo: &TaskRepository,
) -> Result<TaskSwitchable, io::Error> {
    let pre_tasks_links = task_repo
        .query_task_incoming_links_by_task_id(task_id)
        .await?;
    let dep_tasks: Vec<_> = pre_tasks_links
        .clone()
        .into_iter()
        .filter(|link| link.kind.eq("dep"))
        .collect();
    for dep_task in &dep_tasks {
        let pre_task = task_repo
            .query_task_by_id(&unwrap_thing(dep_task.to_owned().incoming.unwrap()))
            .await?;
        if pre_task.complete == false {
            return Ok(TaskSwitchable::False);
        }
    }

    if dep_tasks.len() == 0 && pre_tasks_links.len() > 0 {
        if task_repo
            .task_links_to_tasks(pre_tasks_links)
            .await?
            .into_iter()
            .all(|task| task.complete == true)
        {
            return Ok(TaskSwitchable::True);
        }
    }
    Ok(TaskSwitchable::TrueAndFalse)
}
