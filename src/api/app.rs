use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, patch, post},
    Router,
};
use axum_login::{
    login_required,
    tower_sessions::{
        cookie::{time::Duration, SameSite},
        Expiry, MemoryStore, SessionManagerLayer,
    },
    AuthManagerLayerBuilder,
};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::{
    db::repository::{
            agenda::AgendaRepository, draft::DraftRepository, notification::NotificationRepository,
            project::ProjectRepository, requirement::RequirementRepository, task::TaskRepository,
            user::UserRepository,
        },
    usecase::{
        draft_collaboration::DraftCollaborationManager,
        invitation_token::InvitationTokenRepository, util::auth_backend::AuthBackend,
    },
};

use super::handler::{
    agenda::{
        create_agenda_for_project, create_agenda_for_user, delete_agenda, get_agenda_info,
        get_agendas_for_project, get_agendas_for_user,
    },
    auth::{login, logout, signup},
    draft::{
        create_draft_for_project, create_draft_for_user, draft_ws_handler, get_draft_info,
        get_drafts_for_project, get_drafts_for_user, patch_draft_info,
    },
    event::{create_event_for_agenda, delete_event, get_events_for_agenda, patch_event},
    notification::{get_notifications, handle_notification},
    project::{
        accept_invitation, create_project, gen_invitation_token, get_project_info,
        get_projects_for_user, get_token_info, get_users_for_project, patch_project,
    },
    requirement::{
        create_requirement_for_project, delete_requirement, get_requirement_info,
        get_requirements_for_project, patch_requirement,
    },
    task::{create_task_for_list, delete_task_from_list, get_assigned_tasks_for_user, get_tasks_for_list, patch_task},
    task_link::{
        create_task_link_for_project, create_task_link_for_user, delete_task_link,
        get_links_for_task, get_task_links_for_project, get_task_links_for_user, patch_task_link,
    },
    task_list::{
        create_task_list_for_project, create_task_list_for_user, delete_task_list,
        get_task_list_info, get_task_lists_for_project, get_task_lists_for_user,
    },
    user::{get_user_info, patch_user_info},
};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: UserRepository,
    pub task_repo: TaskRepository,
    pub project_repo: ProjectRepository,
    pub agenda_repo: AgendaRepository,
    pub draft_repo: DraftRepository,
    pub notif_repo: NotificationRepository,
    pub requ_repo: RequirementRepository,
    pub invitation_token_repo: Arc<Mutex<InvitationTokenRepository>>,
    pub draft_collaboration_manager: Arc<Mutex<DraftCollaborationManager>>,
}

pub struct App {
    router: Router,
    config: AppConfig,
}

pub struct AppConfig {
    url: String,
}

impl App {
    pub async fn new() -> Self {
        let state = Arc::new(Mutex::new(AppState {
            user_repo: UserRepository::new().await,
            task_repo: TaskRepository::new().await,
            project_repo: ProjectRepository::new().await,
            agenda_repo: AgendaRepository::new().await,
            draft_repo: DraftRepository::new().await,
            notif_repo: NotificationRepository::new().await,
            requ_repo: RequirementRepository::new().await,
            invitation_token_repo: Arc::new(Mutex::new(InvitationTokenRepository::default())),
            draft_collaboration_manager: Arc::new(Mutex::new(DraftCollaborationManager::new())),
        }));

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax)
            .with_http_only(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        let backend = AuthBackend::new(Arc::new(UserRepository::new().await));

        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
        let cors_layer = CorsLayer::new()
            .allow_origin(["http://localhost:4000".parse::<HeaderValue>().unwrap()])
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_private_network(true)
            .allow_credentials(true)
            .allow_headers([
                header::ACCEPT,
                header::AUTHORIZATION,
                header::COOKIE,
                header::CONTENT_TYPE,
            ]);

        App {
            router: Router::new()
                .route("/ws/drafts/:draft_id", get(draft_ws_handler))
                .route(
                    "/api/projects/:project_id/requirements/:requirement_id",
                    get(get_requirement_info)
                        .patch(patch_requirement)
                        .delete(delete_requirement),
                )
                .route(
                    "/api/projects/:project_id/requirements",
                    get(get_requirements_for_project).post(create_requirement_for_project),
                )
                .route(
                    "/api/users/:user_id/notifications/:notification_id",
                    patch(handle_notification),
                )
                .route("/api/users/:user_id/notifications", get(get_notifications))
                .route(
                    "/api/agendas/:agenda_id/events/:event_id",
                    patch(patch_event).delete(delete_event),
                )
                .route(
                    "/api/agendas/:agenda_id/events",
                    post(create_event_for_agenda).get(get_events_for_agenda),
                )
                .route(
                    "/api/agendas/:agenda_id",
                    get(get_agenda_info).delete(delete_agenda),
                )
                .route(
                    "/api/projects/:project_id/agendas",
                    get(get_agendas_for_project).post(create_agenda_for_project),
                )
                .route("/api/users/:user_id/tasks", get(get_assigned_tasks_for_user))
                .route(
                    "/api/users/:user_id/agendas",
                    get(get_agendas_for_user).post(create_agenda_for_user),
                )
                .route(
                    "/api/links/:link_id",
                    delete(delete_task_link).patch(patch_task_link),
                )
                .route(
                    "/api/projects/:project_id/links",
                    post(create_task_link_for_project).get(get_task_links_for_project),
                )
                .route(
                    "/api/users/:user_id/links",
                    post(create_task_link_for_user).get(get_task_links_for_user),
                )
                .route("/api/links/tasks/:task_id", get(get_links_for_task))
                .route(
                    "/api/task_lists/:task_list_id/tasks/:task_id",
                    delete(delete_task_from_list).patch(patch_task),
                )
                .route(
                    "/api/task_lists/:task_list_id/tasks",
                    get(get_tasks_for_list).post(create_task_for_list),
                )
                .route(
                    "/api/users/:user_id/task_lists",
                    get(get_task_lists_for_user).post(create_task_list_for_user),
                )
                .route(
                    "/api/projects/:project_id/task_lists",
                    get(get_task_lists_for_project).post(create_task_list_for_project),
                )
                .route(
                    "/api/task_lists/:task_list_id",
                    get(get_task_list_info).delete(delete_task_list),
                )
                .route(
                    "/api/projects/:project_id/drafts",
                    get(get_drafts_for_project).post(create_draft_for_project),
                )
                .route(
                    "/api/users/:user_id/drafts",
                    get(get_drafts_for_user).post(create_draft_for_user),
                )
                .route(
                    "/api/drafts/:draft_id",
                    get(get_draft_info).patch(patch_draft_info),
                )
                .route("/api/invitation/:token_id", get(get_token_info))
                .route("/api/invitation/accept", post(accept_invitation))
                .route("/api/invitation/generate", post(gen_invitation_token))
                .route("/api/projects", post(create_project))
                .route(
                    "/api/projects/:project_id/users",
                    get(get_users_for_project),
                )
                .route(
                    "/api/projects/:project_id",
                    get(get_project_info).patch(patch_project),
                )
                .route("/api/users/:user_id/projects", get(get_projects_for_user))
                .route(
                    "/api/users/:user_id",
                    get(get_user_info).patch(patch_user_info),
                )
                .route_layer(login_required!(AuthBackend, login_url = "/login"))
                .route("/api/auth/login", post(login))
                .route("/api/auth/signup", post(signup))
                .route("/api/auth/logout", post(logout))
                .layer(auth_layer)
                .layer(cors_layer)
                .with_state(state.clone()),
            config: AppConfig {
                url: String::from("127.0.0.1:3000"),
            },
        }
    }

    pub async fn serve(self) {
        let listener = tokio::net::TcpListener::bind(&self.config.url)
            .await
            .unwrap();

        axum::serve(listener, self.router).await.unwrap()
    }
}
