use api::app::App;
use tracing::Level;
mod api;
mod db;
mod usecase;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let app = App::new().await;

    app.serve().await;
}
