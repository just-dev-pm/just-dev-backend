use std::sync::Arc;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, patch, post},
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
    db::repository::{project::ProjectRepository, task::TaskRepository, user::UserRepository},
    usecase::util::auth_backend::AuthBackend,
};

use super::handler::{
    auth::{login, logout, signup},
    project::get_projects_for_user,
    user::{get_project_info, get_user_info, patch_user_info},
};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: UserRepository,
    pub task_repo: TaskRepository,
    pub project_repo: ProjectRepository,
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
            .allow_origin(["http://localhost:3000".parse::<HeaderValue>().unwrap()])
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
                .route("/api/projects/:project_id", get(get_project_info))
                .route("/api/users/:user_id/projects", get(get_projects_for_user))
                .route("/api/users/:user_id", patch(patch_user_info))
                .route("/api/users/:user_id", get(get_user_info))
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
