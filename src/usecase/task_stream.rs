use futures::future::try_join_all;

use crate::db::repository::{task::TaskRepository, utils::unwrap_thing};
use std::io::{self, ErrorKind};

async fn refresh_task_status(
    task_id: &String,
    task_repo: &TaskRepository,
) -> Result<(), std::io::Error> {
    let db_task = task_repo
        .query_task_by_id(&task_id)
        .await
        .map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

    // get all pre tasks
    // strategy:
    // 1. status between incomplete status can be changed freely
    // 2. if any pre task is incomplete, task should be incomplete
    // 3. if all pre tasks are auto and complete, task should be complete
    // 4, task is update
    // if task status is changed between incomplete and complete then tasks dependent should be refreshed
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
            let mut task_clone = db_task.clone();
            task_clone.complete = false;
            task_repo.update_task_by_id(&task_id, &task_clone).await?;
            return Ok(());
        }
    }

    if dep_tasks.len() == 0 && pre_tasks_links.len() > 0 {
        let result_futures: Vec<_> = pre_tasks_links
            .into_iter()
            .map(|link| async move {
                let pre_task = task_repo
                    .query_task_by_id(&unwrap_thing(link.to_owned().incoming.unwrap()))
                    .await?;
                Ok::<_, io::Error>(pre_task)
            })
            .collect();
        if !try_join_all(result_futures)
            .await?
            .into_iter()
            .all(|task| task.complete == true)
        {
            return Ok(());
        }

        let mut task_clone = db_task.clone();
        task_clone.complete = true;
        task_repo.update_task_by_id(&task_id, &task_clone).await?;
    }

    Ok(())
}
