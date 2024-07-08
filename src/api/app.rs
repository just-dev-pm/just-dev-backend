use std::sync::Arc;

use axum::{extract::State, routing::get, Router};
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {}

pub struct App {
    router: Router,
    config: AppConfig,
}

pub struct AppConfig {
    url: String,
}

impl App {
    pub async fn new() -> Self {
        let state = Arc::new(Mutex::new(AppState {}));

        App {
            router: Router::new().with_state(state.clone()),
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
