pub mod agenda;
pub mod draft;
pub mod notification;
pub mod project;
pub mod requirement;
pub mod task;
pub mod user;
pub mod utils;

#[cfg(test)]
mod test_user {

    use axum_login::AuthUser;

    use crate::db::model::notification::Notification;
    use crate::usecase::user::insert_user;
    use crate::db::{
            model::{
                agenda::Event, draft::DraftPayload, project::Project, status::StatusPool,
                task::Task, user::User,
            },
            repository::{
                agenda::AgendaRepository,
                draft::DraftRepository,
                notification::NotificationRepository,
                project::ProjectRepository,
                requirement::RequirementRepository,
                task::TaskRepository,
                user::UserRepository,
                utils::unwrap_thing,
            },
        };

    fn create_user() -> User {
        User {
            id: None,
            username: "test".to_string(),
            avatar: "test".to_string(),
            email: "test".to_string(),
            password: "".to_string(),
            status_pool: StatusPool::new(),
        }
    }

    #[tokio::test]
    async fn test_query_user_by_name() {
        let user_repository = UserRepository::new().await;
        let user = user_repository.query_user_by_name("xiwen").await.unwrap();
        assert_eq!(user.username, "xiwen");
    }

    #[tokio::test]
    async fn test_query_user_by_id() {
        let user_repository = UserRepository::new().await;
        let user = user_repository.query_user_by_id("xiwen").await.unwrap();
        assert_eq!(user.id(), "xiwen");
    }

    #[tokio::test]
    async fn test_insert_user() {
        let user_repo = UserRepository::new().await;
        let task_repo = TaskRepository::new().await;
        let agenda_repo = AgendaRepository::new().await;
        let user = insert_user(&user_repo, &task_repo, &agenda_repo, &create_user())
            .await
            .unwrap();
        assert_eq!(user.username, "test");
    }

    #[tokio::test]
    async fn test_update_user() {
        let repository = UserRepository::new().await;
        let mut user = create_user();
        user.username = "dc".to_owned();
        let mut user = repository.update_user("dc", &create_user()).await.unwrap();
        assert_eq!(user.username, "test");
        user.username = "dc".to_owned();
        repository.update_user("dc", &user).await.unwrap();
    }
    #[tokio::test]
    async fn test_query_project_by_id() {
        let repository = ProjectRepository::new().await;
        let project = repository.query_project_by_id("xiwen").await.unwrap();
        assert_eq!(project.name, "xiwen");
    }

    #[tokio::test]
    async fn test_query_admin_by_id() {
        let repository = ProjectRepository::new().await;
        let admin = repository.query_admin_by_id("xiwen").await.unwrap();
        assert_eq!(admin.id(), "xiwen");
    }

    #[tokio::test]
    async fn test_query_members_by_id() {
        let repository = ProjectRepository::new().await;
        let members = repository.query_members_by_id("xiwen").await.unwrap();
        assert_eq!(members[0].id(), "dc");
    }

    #[tokio::test]
    async fn test_insert_project() {
        let repository = ProjectRepository::new().await;
        let project = repository
            .insert_project(&Project {
                id: None,
                name: "test".to_string(),
                avatar: None,
                status_pool: StatusPool::new(),
            })
            .await
            .unwrap();
        assert_eq!(project.name, "test");
    }

    #[tokio::test]
    async fn test_set_user_for_project() {
        let repository = ProjectRepository::new().await;

        let result = repository
            .set_user_for_project("dc", "xiwen", false)
            .await
            .unwrap();
        assert_eq!(result, ());
    }

    #[tokio::test]
    async fn test_delete_user_from_project() {
        let repository = ProjectRepository::new().await;
        let _ = repository
            .set_user_for_project("dc", "xiwen", false)
            .await
            .unwrap();
        let result = repository
            .delete_user_from_project("dc", "xiwen")
            .await
            .unwrap();
        assert_eq!(result, ());
        let _ = repository
            .set_user_for_project("dc", "xiwen", false)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_insert_task_for_task_list() {
        let repository = TaskRepository::new().await;
        let result = repository
            .insert_task_for_task_list(&Task::new("succceed".to_string()), "xiwen")
            .await
            .unwrap();
        assert_eq!(result.name, "succceed");
    }

    #[tokio::test]
    async fn test_query_draft_by_id() {
        let repository = DraftRepository::new().await;
        let result = repository.query_draft_by_id("v7n0ezprm76lvh9mjjpj").await.unwrap();
        assert!(result.content.len() > 0);
    }

    #[tokio::test]
    async fn test_query_draft_by_id_project() {
        let repository = ProjectRepository::new().await;
        let result = repository.query_draft_by_id("v7n0ezprm76lvh9mjjpj").await.unwrap();
        assert!(result.len() > 0);
    }

    #[tokio::test]
    async fn test_query_project_join_by_id() {
        let repository = UserRepository::new().await;
        let result = repository.query_project_join_by_id("xiwen").await.unwrap();
        assert_eq!(result.0[0].name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_draft_for_user() {
        let repository = DraftRepository::new().await;
        let result = repository
            .insert_draft_for_user("xiwen", "xiwen")
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_draft_for_project() {
        let repository = DraftRepository::new().await;
        let result = repository
            .insert_draft_for_project("xiwen", "xiwen")
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_update_project() {
        let repo = ProjectRepository::new().await;
        let project = Project {
            id: None,
            name: "xiwen".into(),
            avatar: None,
            status_pool: StatusPool::default(),
        };
        let result = repo.update_project(&project, "test").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_update_draft() {
        let repo = DraftRepository::new().await;
        let mut dp = DraftPayload::new("xiwen".into(), "content".into());
        dp.id = Some("xiwen".to_owned());
        let result = repo.update_draft(dp).await.unwrap();
        assert_eq!(result.content, "content".to_owned().into_bytes());
    }

    #[tokio::test]
    async fn test_query_agenda_by_id() {
        let repo = AgendaRepository::new().await;
        let result = repo.query_agenda_by_id("xiwen").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_agenda_for_user() {
        let repo = AgendaRepository::new().await;
        let result = repo.insert_agenda_for_user("xiwen", "xiwen").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_agenda_for_project() {
        let repo = AgendaRepository::new().await;
        let result = repo
            .insert_agenda_for_project("test", "xiwen")
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_event_for_agenda() {
        let repo = AgendaRepository::new().await;
        let event = Event::new("xiwen".into(), "test".into());
        let result = repo.insert_event_for_agenda(&event, "xiwen").await.unwrap();
        assert_eq!(result.description, "test")
    }

    #[tokio::test]
    async fn test_query_task_by_id() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_by_id("xiwen").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_user_query_agenda_by_id() {
        let repo = UserRepository::new().await;
        let result = repo.query_agenda_by_id("xiwen").await.unwrap();
        assert!(result.contains(&"xiwen".to_string()));
        assert!(result.contains(&"odm38nhhrt6qacstkrc0".to_string()));
    }

    #[tokio::test]
    async fn test_user_query_draft_by_id() {
        let repo = UserRepository::new().await;
        let result = repo.query_draft_by_id("xiwen").await.unwrap();
        assert!(result.contains(&"xiwen".to_string()));
    }

    #[tokio::test]
    async fn test_assign_task_to_user() {
        let task_repo = TaskRepository::new().await;
        let user_repo = UserRepository::new().await;
        let agenda_repo = AgendaRepository::new().await;
        let user = insert_user(&user_repo, &task_repo, &agenda_repo, &create_user())
            .await
            .unwrap();
        let user_id = unwrap_thing(user.id.clone().unwrap());
        let _ = task_repo
            ._assign_task_to_user("xiwen", &user_id)
            .await
            .unwrap();
        let result = task_repo.query_assigned_tasks_by_user(&user_id).await.unwrap();
        assert!(result.len() > 0);
        let _ = task_repo._deassign_task_for_user("xiwen", &user_id).await.unwrap();
        let result = task_repo.query_assigned_tasks_by_user(&user_id).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_query_task_list_by_user_id() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_list_by_user_id("xiwen").await.unwrap();
        assert!(result.contains(&"xiwen".to_owned()))
    }

    #[tokio::test]
    async fn test_query_task_links_by_task_id() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_links_by_task_id("xiwen").await.unwrap();
        assert!(result.len() == 2)
    }

    #[tokio::test]
    async fn test_insert_delete_links() {
        let repo = TaskRepository::new().await;
        let task_link = repo
            .insert_task_link("xiwen", "orig", "auto")
            .await
            .unwrap();
        assert_eq!(task_link.kind, "auto");
        let link_id = unwrap_thing(task_link.id.clone().unwrap());
        let task_link = repo.delete_task_link_by_id(&link_id).await.unwrap();
        assert_eq!(task_link.kind, "auto");
    }

    #[tokio::test]
    async fn test_update_task_by_id() {
        let repo = TaskRepository::new().await;
        let task = repo
            .update_task_by_id("xiwen", &Task::new("xiwen".to_string()))
            .await
            .unwrap();
        assert_eq!(task.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_notif() {
        let repo = NotificationRepository::new().await;
        let result = repo
            .insert_notif(
                "xiwen",
                "xiwen",
                "task",
                Notification::new("xiwen".to_owned(), "xiwen".to_owned())
            )
            .await
            .unwrap();
        assert_eq!(result.title, "xiwen");
    }

    #[tokio::test]
    async fn test_query_notif_by_id() {
        let repo = NotificationRepository::new().await;
        let (notif, _) = repo.query_notif_by_id("xiwen").await.unwrap();
        assert_eq!(notif.title, "xiwen");
    }

    #[tokio::test]
    async fn test_handle_notif() {
        let repo = NotificationRepository::new().await;
        let result = repo.handle_notif_by_id("xiwen").await.unwrap();
        assert_eq!(result.handled, true);
    }

    #[tokio::test]
    async fn test_query_notif_by_user_id() {
        let repo = UserRepository::new().await;
        let result = repo.query_notif_by_user_id("xiwen").await.unwrap();
        assert!(result.contains(&"qqvxafo5l6vkp7etig3b".to_owned()));
    }

    #[tokio::test]
    async fn test_assign_event_for_user() {
        let repo = AgendaRepository::new().await;
        let result = repo._assign_event_for_user("xiwen", "xiwen").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }



    #[tokio::test]
    async fn test_delete_event() {
        let repo = AgendaRepository::new().await;
        let event = Event::new("xiwen".into(), "test".into());
        let result = repo.insert_event_for_agenda(&event, "xiwen").await.unwrap();
        assert_eq!(result.description, "test");
        let repo = AgendaRepository::new().await;
        let result = repo
            .delete_event(&unwrap_thing(result.id.clone().unwrap()))
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_insert_requ_for_project() {
        let repo = RequirementRepository::new().await;
        let result = repo
            .insert_requ_for_project("xiwen", "xiwen".to_owned(), "xiwen".to_owned())
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
        let repo = RequirementRepository::new().await;
        let result = repo
            .delete_requ_from_project(&unwrap_thing(result.id.clone().unwrap()))
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_query_events_by_agenda_id() {
        let repo = AgendaRepository::new().await;
        let result = repo.query_events_by_agenda_id("xiwen").await.unwrap();
        assert!(result.len() != 0)
    }

    #[tokio::test]
    async fn test_query_assignees_of_event() {
        let repo = AgendaRepository::new().await;
        let result = repo.query_assignees_of_event("xiwen").await.unwrap();
        assert!(result.contains(&"xiwen".to_owned()));
    }

    #[tokio::test]
    async fn test_deassign_user_of_event() {
        let repo = AgendaRepository::new().await;
        let result = repo
            ._deassign_event_for_user("xiwen", "xiwen")
            .await
            .unwrap();
        assert_eq!(result.name, "xiwen");
        let repo = AgendaRepository::new().await;
        let result = repo._assign_event_for_user("xiwen", "xiwen").await.unwrap();
        assert_eq!(result.name, "xiwen");
    }


    #[tokio::test]
    async fn test_query_all_tasks_of_list() {
        let repo = TaskRepository::new().await;
        let result = repo.query_all_tasks_of_task_list("xiwen").await.unwrap();
        assert!(result.contains(&"su2ys11l163wj1vf9s73".to_owned()));
    }

    #[tokio::test]
    async fn test_insert_task_list_for_project() {
        let repo = TaskRepository::new().await;
        let result = repo
            .insert_task_list_for_project("xiwen", "insert task list test")
            .await
            .unwrap();
        assert_eq!(result.name, "insert task list test")
    }

    #[tokio::test]
    async fn test_query_task_is_following() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_is_following("orig").await.unwrap();
        assert_eq!(result.unwrap(), "xiwen");
        let result = repo.query_task_is_following("xiwen").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_query_task_list_source() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_list_source("xiwen").await.unwrap();
        assert_eq!(result.id.to_string(), "xiwen");
    }

    #[tokio::test]
    async fn test_query_assigned_tasks_by_user() {
        let repo = TaskRepository::new().await;
        let result = repo.query_assigned_tasks_by_user("xiwen").await.unwrap();
        assert!(result.len() > 0);
    }

}
