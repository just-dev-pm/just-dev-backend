use crate::db::model::{agenda::Event, notification::Notification, task::Task};


pub fn assigned_task_to_notif(task: Task) -> Notification {
    Notification {
        id: None,
        title: format!("Task: {} has been assigned to you", task.name),
        content: format!("Task description: {}", task.description),
        handled: false,
    }
}


pub fn deassign_task_to_notif(task: Task) -> Notification {
    Notification {
        id: None,
        title: format!("Task: {} has been deassigned from you", task.name),
        content: format!("Task description: {}", task.description),
        handled: false,
    }
}

pub fn assigned_event_to_notif(event: Event) -> Notification {
    Notification {
        id: None,
        title: format!("Event: {} has been assigned to you", event.name),
        content: format!("Event description: {}", event.description),
        handled: false,
    }
}

pub fn deassign_event_to_notif(event: Event) -> Notification {
    Notification {
        id: None,
        title: format!("Event: {} has been deassigned from you", event.name),
        content: format!("Event description: {}", event.description),
        handled: false,
    }
}