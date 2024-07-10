use std::{error::Error, io};
use std::task::Context;
use surrealdb::sql::Thing;

pub type DbModelId = String;


pub fn get_str_id(id: &Option<Thing>) -> String {
    id.as_ref().unwrap().id.to_string()
}

pub fn get_io_error(err: impl Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

pub fn custom_io_error(err: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub fn unwrap_thing(thing: Thing) -> DbModelId {
    thing.id.to_string()
}

pub fn unwrap_things(things: Vec<Thing>) -> Vec<DbModelId> {
    things.into_iter().map(|thing| thing.id.to_string()).collect()
}


use surrealdb::{opt, Response};

use crate::db::db_context::DbContext;


pub async fn create_resource<T>(
    context: &DbContext,
    content: &T,
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

pub async fn select_resourse<T>(context: &DbContext, id: &str, table: &str) -> Result<T, io::Error>
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

pub async fn exec_query(context: &DbContext, query: String) -> Result<Response, io::Error> {
    context.db.query(query).await.map_err(get_io_error)
}

pub async fn exec_double_query(context: &DbContext, query1: String, query2: String) -> Result<Response, io::Error> {
    context.db.query(query1).query(query2).await.map_err(get_io_error)
}

pub async fn extract_from_response<T>(response:&mut Response, index: impl opt::QueryResult<Option<T>>) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
{
    let result = response.take::<Option<T>>(index).map_err(get_io_error)?;
    result.ok_or(io::Error::new(io::ErrorKind::NotFound, "Resource not found"))
} 

