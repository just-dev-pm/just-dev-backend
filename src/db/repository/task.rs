use std::io;

use surrealdb::sql::Thing;

use crate::db::{
    db_context::DbContext,
    model::{
        status::StatusPool,
        task::{Task, TaskList},
    },
};

use super::utils::*;

#[derive(Clone)]
pub struct TaskRepository {
    pub context: DbContext,
}

impl TaskRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_task_by_id(&self, id: &str) -> Result<Task, io::Error> {
        let mut task: Task = select_resourse(&self.context, id, "task").await?;

        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT <-follow<-have<-task_list<-own<-user as user as assignees FROM task where id == task:{}",
                id
            ))
            .query(format!("select <-have<-task_list<-own<-user.status_pool.* as status_pool from task where id == task:{}", id))
            .query(format!("select <-have<-task_list<-own<-project.status_pool.* as status_pool from task where id == task:{}", id))
            .await
            .unwrap();
        let assignees: Vec<Thing> = response.take((0, "user")).unwrap();
        let status_pool_user: Option<StatusPool> = response.take((1, "status_pool")).unwrap();
        let status_pool_project: Option<StatusPool> = response.take((2, "status_pool")).unwrap();

        task.assignees = Some(unwrap_things(assignees));
        task.status_pool = status_pool_project.or(status_pool_user);
        Ok(task)
    }

    pub async fn insert_task_for_task_list(
        &self,
        task: &Task,
        task_list_id: &str,
    ) -> Result<Task, io::Error> {
        let task = create_resource(&self.context, task, "task").await?;
        let _ = exec_query(
            &self.context,
            format!(
                "relate task_list:{task_list_id} -> have -> task:{}",
                get_str_id(&task.id)
            ),
        )
        .await?;
        Ok(task)
    }

    pub async fn insert_extask_list_for_user(
        &self,
        name: &str,
        user_id: &str,
    ) -> Result<TaskList, io::Error> {
        let task_list = create_resource(
            &self.context,
            &TaskList::new_with_id(name.to_string(), user_id, "task_list"),
            "task_list",
        )
        .await?;

        let _ = self
            .context
            .db
            .query(format!(
                "relate user:{} -> own -> task_list:{}",
                user_id,
                get_str_id(&task_list.id)
            ))
            .await
            .map_err(|e| get_io_error(e))?;
        Ok(task_list)
    }

    pub async fn insert_task_list_for_user(
        &self,
        name: &str,
        user_id: &str,
    ) -> Result<TaskList, io::Error> {
        let task_list =
            create_resource(&self.context, &TaskList::new(name.to_string()), "task_list").await?;
        let _ = self
            .context
            .db
            .query(format!(
                "relate user:{} -> own -> task_list:{}",
                user_id,
                get_str_id(&task_list.id)
            ))
            .await
            .map_err(|e| get_io_error(e))?;
        Ok(task_list)
    }

    pub async fn assign_task_to_user(
        &self,
        task_id: &str,
        user_id: &str,
    ) -> Result<Task, io::Error> {
        let mut task = self.query_task_by_id(task_id).await?;
        task.id = None;
        let task = self.insert_task_for_task_list(&task, user_id).await?; // insert task into user's special tasklist

        let _ = self
            .context
            .db
            .query(format!(
                "relate task:{} -> follow -> task:{}",
                task_id, user_id
            ))
            .await
            .map_err(|e| get_io_error(e))?;
        Ok(task)
    }

    pub async fn query_task_list_by_id(&self, id: &str) -> Result<TaskList, io::Error> {
        let mut task_list: TaskList = select_resourse(&self.context, id, "task_list").await?;

        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT ->have->task as tasks FROM task_list where id == task_list:{}",
                id
            ),
        )
        .await?;
        let tasks: Vec<Thing> = response.take((0, "tasks")).unwrap();

        task_list.tasks = Some(unwrap_things(tasks));
        Ok(task_list)
    }

    pub async fn query_task_list_by_user_id(&self, id: &str) -> Result<Vec<TaskList>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("SELECT ->own->task_list FROM user where id == user:{}", id),
        )
        .await?;
        let task_lists: Vec<TaskList> = response.take(0).unwrap();
        Ok(task_lists)
    }
}
