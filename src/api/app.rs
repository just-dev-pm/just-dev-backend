use axum::Router;

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
        App {
            router: Router::new(),
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
