use crate::{auth, error::TodosError, models, schema};

use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn get_all_todos(uid: i32, conn: &PgConnection) -> Result<Vec<models::Todo>, TodosError> {
    use schema::todos::dsl::*;
    let todos_list = todos
        .filter(user_id.eq(uid))
        .load::<models::Todo>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => TodosError::TodoNotFoundError,
            _ => TodosError::DieselCrudError,
        })?;
    Ok(todos_list)
}

pub fn create_new_todo(
    uid: i32,
    data: models::NewTodoReq,
    conn: &PgConnection,
) -> Result<models::Todo, TodosError> {
    use schema::todos::dsl::*;
    let todo = diesel::insert_into(todos)
        .values(models::NewTodo {
            text: data.text,
            user_id: uid,
        })
        .get_result(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => TodosError::TodoNotFoundError,
            _ => TodosError::DieselCrudError,
        })?;
    Ok(todo)
}

pub fn update_existing_todo(
    exisiting_todo: models::Todo,
    data: models::UpdateTodo,
    conn: &PgConnection,
) -> Result<models::Todo, TodosError> {
    use schema::todos::dsl::*;
    let todo: models::Todo;
    use models::UpdateTodo::*;
    match data {
        Both {
            done: new_done,
            text: new_text,
        } => {
            todo = diesel::update(&exisiting_todo)
                .set((text.eq(new_text), done.eq(new_done)))
                .get_result(conn)
                .map_err(|_| TodosError::DieselCrudError)?;
        }
        DoneOnly { done: new_done } => {
            todo = diesel::update(&exisiting_todo)
                .set(done.eq(new_done))
                .get_result(conn)
                .map_err(|_| TodosError::DieselCrudError)?;
        }
        TextOnly { text: new_text } => {
            todo = diesel::update(&exisiting_todo)
                .set(text.eq(new_text))
                .get_result(conn)
                .map_err(|_| TodosError::DieselCrudError)?;
        }
    }
    Ok(todo)
}

pub fn delete_existing_todo(
    exisiting_todo: models::Todo,
    conn: &PgConnection,
) -> Result<models::Todo, TodosError> {
    let todo = diesel::delete(&exisiting_todo)
        .get_result(conn)
        .map_err(|_| TodosError::DieselCrudError)?;

    Ok(todo)
}

pub fn register_user(
    data: models::NewUser,
    conn: &PgConnection,
) -> Result<auth::RegisterResponse, TodosError> {
    use schema::users::dsl::*;

    let salt = SaltString::generate(&mut rand_core::OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password_simple(data.password.as_bytes(), salt.as_ref())
        .unwrap()
        .to_string();
    let user = diesel::insert_into(users)
        .values(models::NewUser {
            password: password_hash,
            username: data.username,
        })
        .get_result::<models::User>(conn)
        .map_err(|a| {
            eprintln!("{:?}", a);
            TodosError::DieselCrudError
        })?;
    let token = auth::create_jwt(user.id, user.username.clone())
        .map_err(|_| TodosError::JwtTokenCreationError)?;
    Ok(auth::RegisterResponse {
        id: user.id,
        token,
        username: user.username,
    })
}

pub fn login_user(
    data: auth::LoginBody,
    conn: &PgConnection,
) -> Result<models::JwtUser, TodosError> {
    use schema::users::dsl::*;

    let argon2 = Argon2::default();
    let user = users
        .filter(username.eq(data.username))
        .first::<models::User>(conn)
        .map_err(|_| TodosError::DieselCrudError)?;

    let parsed_hash = PasswordHash::new(user.password.as_str()).unwrap();
    argon2
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .map_err(|_| TodosError::BadCreds)?;

    let token = auth::create_jwt(user.id, user.username.clone())
        .map_err(|_| TodosError::JwtTokenCreationError)?;
    Ok(models::JwtUser {
        id: user.id,
        token,
        username: user.username,
    })
}
