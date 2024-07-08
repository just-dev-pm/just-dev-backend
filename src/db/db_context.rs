use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

pub struct DbContext {
    db: Surreal<Client>,
}

impl DbContext {
    pub async fn new() -> Self {
        let db: Surreal<Client> = Surreal::init();
        let _ = db.connect::<Ws>("localhost:8000").await.unwrap();
        let _ = db.use_ns("just-dev").use_db("public").await.unwrap();

        DbContext { db }
    }
}
