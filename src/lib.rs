use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub mod error;

#[macro_use]
extern crate diesel;

pub mod actions;
pub mod auth;
pub mod models;
mod schema;
