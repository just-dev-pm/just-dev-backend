pub mod user;
pub mod project;
pub mod task;


#[cfg(test)]
mod test_user {

    use axum_login::AuthUser;
    

    use crate::db::{model::{status::StatusPool, user::User}, repository::user::UserRepository};

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
        let user = user_repository.query_user_by_id("dc").await.unwrap();
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
    

    
}
