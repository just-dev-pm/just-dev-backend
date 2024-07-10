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

    use crate::{
        db::{
            model::{
                agenda::Event, draft::DraftPayload, project::Project, status::StatusPool,
                task::Task, user::User,
            },
            repository::{
                agenda::AgendaRepository,
                draft::DraftRepository,
                project::ProjectRepository,
                task::{Entity, TaskRepository},
                user::UserRepository,
                utils::unwrap_thing,
            },
        },
        usecase::user::insert_user,
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
        assert_eq!(user.id(), "xiwen");
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
        let user = insert_user(&user_repo, &task_repo, &create_user())
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
        let result = repository.query_draft_by_id("xiwen").await.unwrap();
        assert_eq!(result.content, "content".to_owned().into_bytes());
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
        assert_eq!(result.name, "test");
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
        let result = repo.query_task_by_id("xiwen", Entity::User).await.unwrap();
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
        let user = insert_user(&user_repo, &task_repo, &create_user())
            .await
            .unwrap();
        let user_id = unwrap_thing(user.id.clone().unwrap());
        let result = task_repo
            .assign_task_to_user("xiwen", &user_id)
            .await
            .unwrap();
        let user_task_list = task_repo.query_task_list_by_id(&user_id).await.unwrap();
        let task_id = user_task_list.tasks.unwrap().pop().unwrap();
        let task = task_repo
            .query_task_by_id(&task_id, Entity::User)
            .await
            .unwrap();
        assert_eq!(task.name, "xiwen");
        assert_eq!(result.name, "xiwen");
    }

    #[tokio::test]
    async fn test_query_task_list_by_user_id() {
        let repo = TaskRepository::new().await;
        let result = repo.query_task_list_by_user_id("xiwen").await.unwrap();
        assert!(result.contains(&"xiwen".to_owned()))
    }
}
