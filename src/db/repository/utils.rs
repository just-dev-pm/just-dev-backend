use std::{error::Error, io};

use surrealdb::sql::Thing;




pub fn get_str_id(id: &Option<Thing>) -> String {
    id.as_ref().unwrap().id.to_string()
}

pub fn get_io_error(err: impl Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

pub fn custom_io_error(err: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}