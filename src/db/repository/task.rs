use std::io;

use futures::future::try_join_all;
use surrealdb::sql::Thing;

use crate::db::{
    db_context::DbContext,
    model::{
        status::StatusPool,
        task::{Task, TaskLink, TaskList},
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

    #[deprecated]
    pub async fn query_task_is_following(&self, task_id: &str) -> Result<Option<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT ->follow->task as tasks FROM task WHERE id == task:{}",
                task_id
            ),
        ).await?;
        let tasks = response
            .take::<Option<Vec<Thing>>>("tasks")
            .map_err(get_io_error)?
            .unwrap_or_default();
        if tasks.len() == 0 {
            return Ok(None);
        } else {
            return Ok(Some(unwrap_thing(tasks[0].clone())));
        }
    }

    pub async fn query_task_list_source(&self, task_list_id: &str) -> Result<Thing, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT <-own.in as source FROM task_list WHERE id == task_list:{}",
                task_list_id
            ),
        ).await?;
        let source = response
            .take::<Option<Vec<Thing>>>("source")
            .map_err(get_io_error)?
            .map(|mut v| v.pop())
            .ok_or(custom_io_error("Task list source not found"))?
            .ok_or(custom_io_error("Task list source not found"))?;
        Ok(source)
    }

    pub async fn query_task_by_id(&self, id: &str) -> Result<Task, io::Error> {
        let mut task: Task = select_resourse(&self.context, id, "task").await?;
        let mut response = self
            .context
            .db
            .query(format!(
                "SELECT ->assign->user as assignees FROM task where id == task:{}",
                id
            ))
            .await
            .unwrap();
        let assignees = response
            .take::<Option<Vec<Thing>>>((0, "assignees"))
            .map_err(get_io_error)?
            .unwrap_or_default();

        task.assignees = Some(unwrap_things(assignees));
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

    // #[deprecated]
    // pub async fn assign_task_to_user(
    //     &self,
    //     task_id: &str,
    //     user_id: &str,
    // ) -> Result<Task, io::Error> {
    //     let mut task = self.query_task_by_id(task_id, Entity::Project).await?;
    //     task.id = None;
    //     let task = self.insert_task_for_task_list(&task, user_id).await?; // insert task into user's special tasklist

    //     let _ = self
    //         .context
    //         .db
    //         .query(format!(
    //             "relate task:{} -> follow -> task:{}",
    //             unwrap_thing(task.id.clone().unwrap()),
    //             task_id
    //         ))
    //         .await
    //         .map_err(|e| get_io_error(e))?;
    //     Ok(task)
    // }")]
    /// Don't use this func directly because of no notification
    #[doc(hidden)]
    pub async fn _assign_task_to_user(&self, task_id: &str, user_id: &str) -> Result<(), io::Error> {

        let _ = exec_query(&self.context, format!("relate task:{task_id} -> assign -> user:{user_id}")).await?;
        Ok(())
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
            format!(
                "SELECT ->own->task_list as task_lists FROM user where id == user:{}",
                id
            ),
        )
        .await?;
        let task_lists = response
            .take::<Option<Vec<Thing>>>((0, "task_lists"))
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(task_lists))
    }

    pub async fn query_task_links_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<Vec<TaskLink>, io::Error> {

        let mut response = exec_double_query(
            &self.context,
            format!("select * from link where out.id == task:{task_id}"),
            format!("select * from link where in.id == task:{task_id}"),
        )
        .await?;
        let mut tasks: Vec<_> = response
            .take::<Vec<TaskLink>>(0)
            .map_err(get_io_error)?;
            
        tasks.extend(
            response
                .take::<Vec<TaskLink>>(1)
                .map_err(get_io_error)?
        );

        Ok(tasks)
    }

    pub async fn query_task_outgoing_links_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<Vec<TaskLink>, io::Error> {

        let mut response = exec_query(
            &self.context,
            format!("select * from link where in.id == task:{task_id}"),
        )
        .await?;
        let tasks: Vec<_> = response
            .take::<Vec<TaskLink>>(0)
            .map_err(get_io_error)?;

        Ok(tasks)
    }

    pub async fn query_task_incoming_links_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<Vec<TaskLink>, io::Error> {

        let mut response = exec_query(
            &self.context,
            format!("select * from link where out.id == task:{task_id}"),
        )
        .await?;
        let tasks: Vec<_> = response
            .take::<Vec<TaskLink>>(0)
            .map_err(get_io_error)?;

        Ok(tasks)
    }

    pub async fn insert_task_link(&self, former: &str, latter: &str, kind: &str) -> Result<TaskLink, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "relate task:{former} -> link -> task:{latter} set type = '{kind}'",
            )
        ).await?;
        let link = response.take::<Option<TaskLink>>(0).map_err(get_io_error)?;
        link.ok_or(custom_io_error("Create link fail"))
    }

    pub async fn delete_task_link_by_id(&self, task_link_id: &str) -> Result<TaskLink, io::Error> {
        let task_link: Option<TaskLink> = self.context.db.delete(("link", task_link_id)).await.map_err(get_io_error)?;
        task_link.ok_or(custom_io_error("Delete link fail"))
    }

    pub async fn update_task_by_id(&self, task_id: &str, task: &Task) -> Result<Task, io::Error> {
        let task = update_resource(&self.context, task_id, task, "task").await?;
        Ok(task)
    }

    pub async fn delete_task_list(&self, task_list_id: &str) -> Result<TaskList, io::Error> {
        let task_list: TaskList = delete_resource(&self.context, task_list_id, "task_list").await?;
        Ok(task_list)
    }

    // #[deprecated]
    // pub async fn query_assignees_of_task(
    //     &self,
    //     event_id: &str,
    // ) -> Result<Vec<DbModelId>, io::Error> {
    //     let mut response = exec_query(
    //         &self.context,
    //         format!(
    //             "(SELECT <-follow<-task<-have<-task_list<-own<-user as assignees FROM event WHERE id == task:{}).assignees",
    //             event_id
    //         ),
    //     )
    //     .await?;
    //     let assignees = response
    //         .take::<Option<Vec<Thing>>>(0)
    //         .map_err(get_io_error)?
    //         .unwrap_or_default();
    //     Ok(unwrap_things(assignees))
    // }

    pub async fn query_assignees_of_task(&self, task_id: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(&self.context, format!("SELECT ->assign->user as assignees FROM task where id == task:{}", task_id)).await?;
        let assignees = response.take::<Option<Vec<Thing>>>("assignees").map_err(get_io_error)?.unwrap_or_default();
        Ok(unwrap_things(assignees))
    } 

    // #[deprecated]
    // pub async fn deassign_task_for_user(
    //     &self,
    //     event_id: &str,
    //     user_id: &str,
    // ) -> Result<Task, io::Error> {
    //     let mut response = exec_double_query(
    //         &self.context, 
    //         format!("(select <-follow<-task as events from event where id == event:{event_id}).events"), 
    //         format!("(select ->have->task_list as assigned from agenda where id == agenda:{user_id}).assigned")).await?;
    //     let events = unwrap_things(response
    //         .take::<Option<Vec<Thing>>>(0)
    //         .map_err(get_io_error)?
    //         .unwrap_or_default());
    //     let user_assigned = unwrap_things(response.take::<Option<Vec<Thing>>>(1).map_err(get_io_error)?.unwrap_or_default());
    //     for event in events {
    //         if user_assigned.contains(&event) {
    //             return Ok(delete_resource::<Task>(&self.context, &event, "task").await?);
    //         }
    //     }
    //     Err(custom_io_error("Assigning relation not found"))
    // }

    /// Don't use this func directly because of no notification
    pub async fn _deassign_task_for_user(&self, task_id: &str, user_id: &str) -> Result<(), io::Error> {
        let _ = exec_query(&self.context, format!("DELETE task:{task_id}->assign WHERE out==user:{user_id}")).await?;
        Ok(())
    }

    pub async fn insert_task_list_for_project(&self, project_id: &str, name: &str) -> Result<TaskList, io::Error> {
        let task_list = create_resource(
            &self.context,
            &TaskList::new(name.to_string()),
            "task_list",
        )
        .await?;

        let _ = self
            .context
            .db
            .query(format!(
                "relate project:{} -> own -> task_list:{}",
                project_id,
                get_str_id(&task_list.id)
            ))
            .await
            .map_err(|e| get_io_error(e))?;
        Ok(task_list)
    }

    pub async fn query_all_tasks_of_task_list(&self, task_list: &str) -> Result<Vec<DbModelId>, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT ->have->task as tasks FROM task_list where id == task_list:{}",
                task_list
            ),
        )
        .await?;
        let tasks = response
            .take::<Option<Vec<Thing>>>("tasks")
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_things(tasks))
    }

    pub async fn query_task_list_id_by_task(&self, task_id: &str) -> Result<DbModelId, io::Error> {
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT <-have<-task_list as task_lists FROM task where id == task:{task_id}"
            )
        ).await?;
        let mut task_lists = response
            .take::<Option<Vec<Thing>>>("task_lists")
            .map_err(get_io_error)?
            .unwrap_or_default();
        Ok(unwrap_thing(task_lists.pop().ok_or(custom_io_error("Task has no parent task list!"))?))
    }

    pub async fn query_assigned_tasks_by_user(&self, user_id: &str) -> Result<Vec<(Task, DbModelId, DbModelId)>, io::Error> {
        // task_id  task_list_id  source_id
        let mut response = exec_query(
            &self.context,
            format!(
                "SELECT <-assign<-task as tasks FROM user where id == user:{}",
                user_id
            )
        ).await?;
        let tasks = unwrap_things(response
            .take::<Option<Vec<Thing>>>("tasks")
            .map_err(get_io_error)?
            .unwrap_or_default());
        let futures = tasks.into_iter().map(|task_id| async move {
            let task = self.query_task_by_id(&task_id).await?;
            let task_list = self.query_task_list_id_by_task(&task_id).await?;
            let source = self.query_task_list_source(&task_list).await?;

            Ok::<_, io::Error>((task, task_list, source.id.to_string()))
        }).collect::<Vec<_>>();
        Ok(try_join_all(futures).await?) 
    
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<Task, io::Error> {
        let task: Task = delete_resource(&self.context, task_id, "task").await?;
        Ok(task)
    }
}
