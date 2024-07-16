use std::{env, sync::Arc};

use axum::{
    http::{header, HeaderValue, Method}, middleware, routing::{get, post}, Router
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

use super::handler::*;

use super::handler::{
    draft::draft_ws_handler, webhook::{filter_github_webhook_requests, handle_pull_request_event},
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
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::OPTIONS,
                Method::PATCH,
                Method::DELETE,
            ])
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
                .nest("/api/projects", project::router())
                .nest("/api/users", user::router())
                .nest("/api/task_lists", task_list::router())
                .nest("/api/links", task_link::router())
                .nest("/api/agendas", agenda::router())
                .nest("/api/drafts", draft::router())
                .nest("/api/invitation", project::invitation_router())
                .route_layer(login_required!(AuthBackend, login_url = "/login"))
                .nest("/api/auth", auth::router())
                .route("/api/webhooks/github", post(handle_pull_request_event).layer(middleware::from_fn(filter_github_webhook_requests))) // TODO: add auth to webhook
                .layer(auth_layer)
                .layer(cors_layer)
                .with_state(state.clone()),
            config: AppConfig {
                url: env::var("JUST_DEV_SERVER_URL").expect("JUST_DEV_SERVER_URL must be set"),
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
