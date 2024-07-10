use std::io;

use surrealdb::Response;

use crate::db::db_context::DbContext;

use super::utils::get_io_error;

pub trait Repository {
    async fn create_resource<T>(
        &self,
        context: &DbContext,
        content: T,
        table: &str,
    ) -> Result<T, io::Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let result: Option<T> = context
            .db
            .create(table)
            .content(content)
            .await
            .map_err(|e| get_io_error(e))?
            .pop();
        result.ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Resource insert fail",
        ))
    }

    async fn select_resourse<T>(&self, context: &DbContext, id: &str, table: &str) -> Result<T, io::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let result: Option<T> = context
            .db
            .select((table, id))
            .await
            .map_err(|e| get_io_error(e))?;
        result.ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Resource not found",
        ))
    }

    async fn exec_query(&self, context: &DbContext, query: String) -> Result<Response, io::Error> {
        context.db.query(query).await.map_err(get_io_error)
    }
}
