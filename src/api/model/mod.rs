pub mod agenda;
pub mod asset;
pub mod draft;
pub mod notification;
pub mod project;
pub mod requirement;
pub mod status;
pub mod task;
pub mod user;
pub mod util;

#[cfg(test)]
mod tests {

    use self::{
        task::TaskRelation,
        user::User,
        util::Id,
    };

    use super::*;
    
    use serde_json;

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "user1".to_owned(),
            username: "dc392".to_owned(),
            email: Some("dc392@email.com".to_owned()),
            avatar: None,
            status_pool: None,
        };

        let json = serde_json::to_string(&user).unwrap();

        assert_eq!(
            json,
            r#"{"id":"user1","username":"dc392","email":"dc392@email.com"}"#
        );
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{"id":"user1","username":"dc392","email":"dc392@email.com"}"#;
        let deserialized: User = serde_json::from_str(json).unwrap();

        assert_eq!(
            deserialized,
            User {
                id: "user1".to_owned(),
                username: "dc392".to_owned(),
                email: Some("dc392@email.com".to_owned()),
                avatar: None,
                status_pool: None,
            }
        );
    }

    // #[test]
    // fn test_status_serialization() {
    //     let status = Status {
    //         pool: StatusPool {
    //             incomplete: vec![],
    //             complete: StatusItem {
    //                 name: "complete".to_owned(),
    //                 description: "finally!".to_owned(),
    //             },
    //         },
    //         status_item: ActualStatusItem::Complete,
    //     };
    //     let json = serde_json::to_string(&status).unwrap();

    //     assert_eq!(
    //         json,
    //         r#"{"pool":{"incomplete":[],"complete":{"name":"complete","description":"finally!"}},"status_item":{"category":"complete"}}"#
    //     );
    // }

    // #[test]
    // fn test_status_deserialization() {
    //     let json = r#"{"pool":{"incomplete":[{"id":"1","status":{"name":"plan","description":"planned"}}],"complete":{"name":"com","description":"alr"}},"status_item":{"id":"1","category":"incomplete"}}"#;

    // let status = Status {
    //     pool: StatusPool {
    //         incomplete: vec![IndexedStatusItem {
    //             id: "1".to_owned(),
    //             status: StatusItem {
    //                 name: "plan".to_owned(),
    //                 description: "planned".to_owned(),
    //             },
    //         }],
    //         complete: StatusItem {
    //             name: "com".to_owned(),
    //             description: "alr".to_owned(),
    //         },
    //     },
    //     status_item: ActualStatusItem::Incomplete { id: "1".to_owned() },
    // };

    // let deserialized: Status = serde_json::from_str(json).unwrap();

    //     assert_eq!(status, deserialized);
    // }

    // #[test]
    // fn test_task_serialization() {
    //     let task = Task {
    //         id: "1".to_owned(),
    //         name: "do something".to_owned(),
    //         description: "I must do something".to_owned(),
    //         assignees: vec![],
    //         status: None,
    //         deadline: DateTime::default(),
    //     };

    // let json = serde_json::to_string(&task).unwrap();

    // let expected = r#"{"id":"1","name":"do something","description":"I must do something","assignees":[]}"#;

    //     assert_eq!(json, expected)
    // }

    #[test]
    fn test_task_relation_serialization() {
        let task_relation = TaskRelation {
            id: "1".to_owned(),
            from: Id { id: "2".to_owned() },
            to: Id { id: "3".to_owned() },
            category: task::TaskRelationType::Auto,
        };

        let json = serde_json::to_string(&task_relation).unwrap();

        let expected = r#"{"id":"1","from":{"id":"2"},"to":{"id":"3"},"category":"auto"}"#;

        assert_eq!(json, expected);
    }
}
