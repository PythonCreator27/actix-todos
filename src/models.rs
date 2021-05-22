use super::schema::{todos, users};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Identifiable)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub done: bool,
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTodoReq {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[table_name = "todos"]
pub struct NewTodo {
    pub text: String,
    pub user_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum UpdateTodo {
    TextOnly { text: String },
    DoneOnly { done: bool },
    Both { text: String, done: bool },
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUser {
    pub token: String,
    pub username: String,
    pub id: i32,
}
