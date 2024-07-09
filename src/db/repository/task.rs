use std::io;

use crate::db::{
    db_context::DbContext,
    model::{status::StatusPool, task::{Task, TaskList}, user::User},
};

pub struct TaskRepository {
    context: DbContext,
}

impl TaskRepository {
    pub async fn new() -> Self {
        Self {
            context: DbContext::new().await,
        }
    }

    pub async fn query_task_by_id(&self, id: &str) -> Result<Task, io::Error> {
        let mut task: Option<Task> = self
            .context
            .db
            .select(("task", id))
            .await
            .unwrap_or_else(|_| None);

        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT <-follow<-have<-task_list<-own<-user as assignees FROM task where id == task:{}",
                id
            ))
            .query(format!("select <-have<-task_list<-own<-user.status_pool from task where id == task:{}", id))
            .query(format!("select <-have<-task_list<-own<-project.status_pool from task where id == task:{}", id))
            .await
            .unwrap();
        let assignees: Vec<User> = response.take(0).unwrap();
        let status_pool_user: Option<StatusPool> = response.take(1).unwrap();
        let status_pool_project: Option<StatusPool> = response.take(2).unwrap(); 


        if let Some(task) = task.as_mut() {
            task.assignees = Some(assignees);
            task.status_pool = status_pool_project.or(status_pool_user);
        }
        task.ok_or(io::Error::new(io::ErrorKind::NotFound, "Task not found"))
    }

    pub async fn insert_task(&self, task: &Task) -> Result<Task, io::Error> {
        let result: Option<Task> = self
            .context
            .db
            .create("task")
            .content(task)
            .await
            .unwrap()
            .pop();
        result.ok_or(io::Error::new(io::ErrorKind::NotFound, "Task insert fail"))
    }

    pub async fn query_task_list_by_id(&self, id: &str) -> Result<TaskList, io::Error> {
        let mut task_list: Option<TaskList> = self
            .context
            .db
            .select(("task_list", id))
            .await
            .unwrap_or_else(|_| None);

        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT ->have->task FROM task_list where id == task_list:{}",
                id
            ))
            .await
            .unwrap();
        let tasks: Vec<Task> = response.take(0).unwrap();

        if let Some(task) = task_list.as_mut() {
            task.tasks = Some(tasks);
        }
        task_list.ok_or(io::Error::new(io::ErrorKind::NotFound, "TaskList not found"))
    }

    pub async fn query_task_list_by_user_id(&self, id: &str) -> Result<Vec<TaskList>, io::Error> {
        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT ->own->task_list FROM user where id == user:{}",
                id
            ))
            .await
            .unwrap();
        let task_lists: Vec<TaskList> = response.take(0).unwrap();
        Ok(task_lists)
    }
    
}
