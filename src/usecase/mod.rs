pub mod draft_collaboration;
pub mod invitation_token;
pub mod task_stream;
pub mod user;
pub mod notification;
pub mod util;

#[cfg(test)]
mod test {
    use crate::{
        db::{
            model::{status::StatusPool, user::User},
            repository::{agenda::AgendaRepository, task::TaskRepository, user::UserRepository},
        },
        usecase::user::insert_user,
    };

    fn create_user() -> User {
        User {
            id: None,
            username: "test_insert_user".to_string(),
            avatar: "test".to_string(),
            email: "test".to_string(),
            password: "".to_string(),
            status_pool: StatusPool::default(),
        }
    }

    #[tokio::test]
    async fn test_insert_user() {
        let user_repo = UserRepository::new().await;
        let task_repo = TaskRepository::new().await;
        let agenda_repo = AgendaRepository::new().await;

        let user = insert_user(&user_repo, &task_repo, &agenda_repo, &create_user())
            .await
            .unwrap();
        assert_eq!(user.username, "test_insert_user");
    }
}
