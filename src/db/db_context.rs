use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

#[derive(Clone, Debug)]
pub struct DbContext {
    pub db: Surreal<Client>,
}

impl DbContext {
    pub async fn new() -> Self {
        let db: Surreal<Client> = Surreal::init();
        db.connect::<Ws>("localhost:8000").await.unwrap();
        db.use_ns("justdev").use_db("public").await.unwrap();

        DbContext { db }
    }
}
