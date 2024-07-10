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

pub enum Entity {
    User,
    Project,
}

impl TaskRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_task_by_id(&self, id: &str, entity: Entity) -> Result<Task, io::Error> {
        let mut task: Task = select_resourse(&self.context, id, "task").await?;

        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT <-follow<-task<-have<-task_list<-own<-user as assignees FROM task where id == task:{}",
                id
            ))
            .query(format!("select <-have<-task_list<-own<-user.status_pool.* as status_pool from task where id == task:{}", id))
            .query(format!("select <-have<-task_list<-own<-project.status_pool.* as status_pool from task where id == task:{}", id))
            .await
            .unwrap();
        let assignees = response
            .take::<Option<Vec<Thing>>>((0, "assignees"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        let status_pool = match entity {
            Entity::User => response
                .take::<Option<Vec<StatusPool>>>((1, "status_pool"))
                .map_err(get_io_error)?,
            Entity::Project => response
                .take::<Option<Vec<StatusPool>>>((2, "status_pool"))
                .map_err(get_io_error)?,
        };

        task.assignees = Some(unwrap_things(assignees));
        task.status_pool = status_pool.and_then(|mut s| s.pop());
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
        let mut task = self.query_task_by_id(task_id, Entity::Project).await?;
        task.id = None;
        let task = self.insert_task_for_task_list(&task, user_id).await?; // insert task into user's special tasklist

        let _ = self
            .context
            .db
            .query(format!(
                "relate task:{} -> follow -> task:{}",
                unwrap_thing(task.id.clone().unwrap()),
                task_id
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
        let tasks = response
            .take::<Option<Vec<Thing>>>((0, "tasks"))
            .unwrap()
            .unwrap_or_default();

        task_list.tasks = Some(unwrap_things(tasks));
        Ok(task_list)
    }

    pub async fn query_task_list_by_user_id(&self, id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!("SELECT ->own->task_list as task_lists FROM user where id == user:{}", id),
        )
        .await?;
        let task_lists = response.take::<Option<Vec<Thing>>>((0, "task_lists")).map_err(get_io_error)?.unwrap_or_default();
        Ok(unwrap_things(task_lists))
    }
}
