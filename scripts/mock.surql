RELATE notification:1 -> about:1 -> event:1 CONTENT { id: about:1, in: notification:1, out: event:1 };
RELATE notification:2 -> about:2 -> task:1 CONTENT { id: about:2, in: notification:2, out: task:1 };
RELATE notification:3 -> about:3 -> draft:1 CONTENT { id: about:3, in: notification:3, out: draft:1 };
RELATE notification:4 -> about:4 -> event:2 CONTENT { id: about:4, in: notification:4, out: event:2 };
RELATE notification:5 -> about:5 -> task:2 CONTENT { id: about:5, in: notification:5, out: task:2 };

UPDATE agenda:1 CONTENT { id: agenda:1, name: 'Team Meeting' };
UPDATE agenda:2 CONTENT { id: agenda:2, name: 'Project Planning' };
UPDATE agenda:3 CONTENT { id: agenda:3, name: 'Daily Stand-up' };
UPDATE agenda:4 CONTENT { id: agenda:4, name: 'Sprint Review' };
UPDATE agenda:5 CONTENT { id: agenda:5, name: 'Retrospective' };


UPDATE draft:1 CONTENT { id: draft:1, content: '0x12345678', name: 'Draft 1' };
UPDATE draft:2 CONTENT { id: draft:2, content: '0x87654321', name: 'Draft 2' };
UPDATE draft:3 CONTENT { id: draft:3, content: '0xabcdef12', name: 'Draft 3' };
UPDATE draft:4 CONTENT { id: draft:4, content: '0x1234abcd', name: 'Draft 4' };
UPDATE draft:5 CONTENT { id: draft:5, content: '0xabcd1234', name: 'Draft 5' };


UPDATE event:1 CONTENT { id: event:1, datetime: '2024-07-08T10:00:00Z', description: 'Kick-off Meeting', duration: '1h', name: 'Kick-off' };
UPDATE event:2 CONTENT { id: event:2, datetime: '2024-07-09T14:00:00Z', description: 'Review Session', duration: '2h', name: 'Review' };
UPDATE event:3 CONTENT { id: event:3, datetime: '2024-07-10T09:00:00Z', description: 'Client Presentation', duration: '3h', name: 'Presentation' };
UPDATE event:4 CONTENT { id: event:4, datetime: '2024-07-11T13:00:00Z', description: 'Team Building', duration: '4h', name: 'Team Building' };
UPDATE event:5 CONTENT { id: event:5, datetime: '2024-07-12T15:00:00Z', description: 'Project Retrospective', duration: '2h', name: 'Retrospective' };


RELATE task_list:1 -> have:1 -> task:1 CONTENT { id: have:1, in: task_list:1, out: task:1 };
RELATE task_list:2 -> have:2 -> task:2 CONTENT { id: have:2, in: task_list:2, out: task:2 };
RELATE task_list:3 -> have:3 -> task:3 CONTENT { id: have:3, in: task_list:3, out: task:3 };
RELATE task_list:4 -> have:4 -> task:4 CONTENT { id: have:4, in: task_list:4, out: task:4 };
RELATE task_list:5 -> have:5 -> task:5 CONTENT { id: have:5, in: task_list:5, out: task:5 };


RELATE user:1 -> join:1 -> agenda:1 CONTENT { id: join:1, in: user:1, out: agenda:1 };
RELATE user:2 -> join:2 -> agenda:2 CONTENT { id: join:2, in: user:2, out: agenda:2 };
RELATE user:3 -> join:3 -> agenda:3 CONTENT { id: join:3, in: user:3, out: agenda:3 };
RELATE user:4 -> join:4 -> agenda:4 CONTENT { id: join:4, in: user:4, out: agenda:4 };
RELATE user:5 -> join:5 -> agenda:5 CONTENT { id: join:5, in: user:5, out: agenda:5 };


RELATE user:1 -> own:1 -> task_list:1 CONTENT { id: own:1, in: user:1, out: task_list:1 };
RELATE user:2 -> own:2 -> task_list:2 CONTENT { id: own:2, in: user:2, out: task_list:2 };
RELATE user:3 -> own:3 -> task_list:3 CONTENT { id: own:3, in: user:3, out: task_list:3 };
RELATE user:4 -> own:4 -> task_list:4 CONTENT { id: own:4, in: user:4, out: task_list:4 };
RELATE user:5 -> own:5 -> task_list:5 CONTENT { id: own:5, in: user:5, out: task_list:5 };


UPDATE task:1 CONTENT { id: task:1, complete: false, description: 'Complete the report', name: 'Report', status: 1, ddl: "2024-07-08T10:00:00Z"};
UPDATE task:2 CONTENT { id: task:2, complete: true, description: 'Send the email', name: 'Email', status: 2, ddl: "2024-07-08T10:00:00Z"};
UPDATE task:3 CONTENT { id: task:3, complete: false, description: 'Prepare presentation slides', name: 'Slides', status: 1 , ddl: "2024-07-08T10:00:00Z"};
UPDATE task:4 CONTENT { id: task:4, complete: true, description: 'Update project plan', name: 'Plan', status: 2 , ddl: "2024-07-08T10:00:00Z"};
UPDATE task:5 CONTENT { id: task:5, complete: false, description: 'Organize team event', name: 'Event', status: 1 , ddl: "2024-07-08T10:00:00Z"};


UPDATE task_list:1 CONTENT { id: task_list:1, name: 'Work' };
UPDATE task_list:2 CONTENT { id: task_list:2, name: 'Personal' };
UPDATE task_list:3 CONTENT { id: task_list:3, name: 'School' };
UPDATE task_list:4 CONTENT { id: task_list:4, name: 'Family' };
UPDATE task_list:5 CONTENT { id: task_list:5, name: 'Friends' };

UPDATE user:1 CONTENT { id: user:1, avatar: 'https://example.com/avatar1.png', email: 'user1@example.com', name: 'User One', password: 'password1' };
UPDATE user:2 CONTENT { id: user:2, avatar: 'https://example.com/avatar2.png', email: 'user2@example.com', name: 'User Two', password: 'password2' };
UPDATE user:3 CONTENT { id: user:3, avatar: 'https://example.com/avatar3.png', email: 'user3@example.com', name: 'User Three', password: 'password3' };
UPDATE user:4 CONTENT { id: user:4, avatar: 'https://example.com/avatar4.png', email: 'user4@example.com', name: 'User Four', password: 'password4' };
UPDATE user:5 CONTENT { id: user:5, avatar: 'https://example.com/avatar5.png', email: 'user5@example.com', name: 'User Five', password: 'password5' };

