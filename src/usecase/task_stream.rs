use crate::db::repository::task::TaskRepository;
use std::io::{self, ErrorKind};

async fn refresh_task_status(
    task_id: &String,
    task_repo: &TaskRepository,
) -> Result<(), std::io::Error> {
    let db_task = task_repo.query_task_by_id(&task_id).await;

    let db_task = db_task.map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

    // get all pre tasks
    // strategy:
    // 1. status between incomplete status can be changed freely
    // 2. if any pre task is incomplete, task should be incomplete
    // 3. if all pre tasks are auto and complete, task should be complete
    // 4, task is update
    // if task status is changed between incomplete and complete then tasks dependent should be refreshed
    

    Ok(())
}
