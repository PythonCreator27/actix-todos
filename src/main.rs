use actix_web::{delete, get, patch, post, web, App, Error, HttpResponse, HttpServer};
use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use todos::{
    actions::{
        create_new_todo, delete_existing_todo, get_all_todos, login_user, register_user,
        update_existing_todo,
    },
    auth::{AuthUser, LoginBody, TodoIsOfUser},
    error::TodosError,
    models::{self, UpdateTodo},
    DbPool,
};

#[get("/todos")]
async fn get_todos(pool: web::Data<DbPool>, user: AuthUser) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");
    let result = web::block(move || get_all_todos(user.id, &conn)).await;

    match result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into())
            }
            _ => unreachable!(),
        },
        Ok(todos) => Ok(HttpResponse::Ok().json(todos)),
    }
}

#[post("/todos")]
async fn add_todo(
    pool: web::Data<DbPool>,
    body: web::Json<models::NewTodoReq>,
    user: AuthUser,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");
    let result = web::block(move || create_new_todo(user.id, body.into_inner(), &conn)).await;

    match result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into())
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok(HttpResponse::Created().json(todo)),
    }
}

#[get("/todos/{todo_id}")]
async fn get_todo(todo_result: TodoIsOfUser) -> Result<HttpResponse, Error> {
    match todo_result.result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into());
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok(HttpResponse::Ok().json(todo)),
    }
}

#[patch("/todos/{todo_id}")]
async fn update_todo(
    pool: web::Data<DbPool>,
    body: web::Json<UpdateTodo>,
    todo_result: TodoIsOfUser,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");
    let todo = match todo_result.result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into());
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok::<models::Todo, Error>(todo),
    }?;
    let result = web::block(move || update_existing_todo(todo, body.into_inner(), &conn)).await;
    match result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into());
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok(HttpResponse::Ok().json(todo)),
    }
}

#[delete("/todos/{todo_id}")]
async fn delete_todo(
    pool: web::Data<DbPool>,
    todo_result: TodoIsOfUser,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");

    let todo = match todo_result.result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into());
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok::<models::Todo, Error>(todo),
    }?;

    let result = web::block(move || delete_existing_todo(todo, &conn)).await;

    match result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while fetching the todo."
                    }))
                    .into())
            }
            TodosError::TodoNotFoundError => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "The todo that you were trying to find does not exist."
                    }))
                    .into())
            }
            _ => unreachable!(),
        },
        Ok(todo) => Ok(HttpResponse::Ok().json(todo)),
    }
}

#[post("/users")]
async fn register(
    pool: web::Data<DbPool>,
    body: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");
    let result = web::block(move || register_user(body.into_inner(), &conn)).await;

    match result {
        Err(e) => {
            match e.into() {
                TodosError::DieselCrudError => return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while registering. Please try again later."
                    }))
                    .into()),
                TodosError::JwtTokenCreationError => {
                    return Err(HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "message": "Something went wrong!"
                        }))
                        .into())
                }
                _ => unreachable!(),
            }
        }
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
    }
}

#[post("/login")]
async fn login(pool: web::Data<DbPool>, body: web::Json<LoginBody>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("Could not get db conn from pool.");
    let result = web::block(move || login_user(body.into_inner(), &conn)).await;
    match result {
        Err(e) => match e.into() {
            TodosError::DieselCrudError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while performing DB operations."
                    }))
                    .into())
            }
            TodosError::BadCreds => {
                return Err(HttpResponse::NotFound()
                    .json(serde_json::json!({
                        "message": "Bad credentials"
                    }))
                    .into());
            }
            TodosError::JwtTokenCreationError => {
                return Err(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "message": "Something went wrong while creating the token."
                    }))
                    .into())
            }
            _ => unreachable!(),
        },
        Ok(jwt_user) => Ok(HttpResponse::Ok().json(jwt_user)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(get_todos)
            .service(get_todo)
            .service(add_todo)
            .service(update_todo)
            .service(delete_todo)
            .service(register)
            .service(login)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
