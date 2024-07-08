use api::app::App;
mod api;
mod db;
mod usecase;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let app = App::new().await;

    app.serve().await;
}
