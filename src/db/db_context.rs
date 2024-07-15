use std::env;

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
        db.connect::<Ws>(
            env::var("JUST_DEV_DATABASE_URL").expect("JUST_DEV_DATABASE_URL must be set"),
        )
        .await
        .unwrap();
        db.use_ns("justdev").use_db("public").await.unwrap();

        DbContext { db }
    }
}
