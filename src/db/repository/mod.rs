pub mod user;
pub mod project;
pub mod task;
pub mod utils;

#[cfg(test)]
mod test_user {

    use axum_login::AuthUser;
    

    use crate::db::{model::{project::Project, status::StatusPool, task::Task, user::User}, repository::{project::ProjectRepository, task::TaskRepository, user::UserRepository}};

    fn create_user() -> User {
        User {
            id: None,
            username: "test".to_string(),
            avatar: "test".to_string(),
            email: "test".to_string(),
            password: "".to_string(),
            status_pool: StatusPool::default(),
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
        assert_eq!(user.id(), "dc");
    }

    #[tokio::test]
    async fn test_insert_user() {
        let user_repository = UserRepository::new().await;
        let user = user_repository.insert_user(&create_user()).await.unwrap();
        assert_eq!(user.username, "test");
    }

    #[tokio::test]
    async fn test_update_user() {
        let repository = UserRepository::new().await;
        let user = repository.update_user("dc", &create_user()).await.unwrap();
        assert_eq!(user.username, "test");
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
        let project = repository.insert_project(&Project {id: None, name: "test".to_string(), avatar: None, status_pool: StatusPool::default()}).await.unwrap();
        assert_eq!(project.name, "test");
    }

    #[tokio::test] 
    async fn test_set_user_for_project() {
        let repository = ProjectRepository::new().await;

        let result = repository.set_user_for_project("dc", "xiwen", false).await.unwrap();
        assert_eq!(result, ());
    }

    #[tokio::test]
    async fn test_delete_user_from_project() {
        let repository = ProjectRepository::new().await;
        let result = repository.delete_user_from_project("dc", "xiwen").await.unwrap();
        assert_eq!(result, ());
    }

    #[tokio::test]
    async fn test_insert_task_for_task_list() {
        let repository = TaskRepository::new().await;
        let result = repository.insert_task_for_task_list(&Task::new("succceed".to_string()), "xiwen").await.unwrap();
        assert_eq!(result.name, "succceed");
    }
}
