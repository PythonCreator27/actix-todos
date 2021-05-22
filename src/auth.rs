use std::{
    error::Error as StdError,
    future::{ready, Ready},
};

use actix_web::{dev, web, Error, FromRequest, HttpRequest, HttpResponse};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::TodosError;

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterResponse {
    pub token: String,
    pub id: i32,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    pub username: String,
    pub id: i32,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

pub fn create_jwt(uid: i32, uname: String) -> Result<String, Box<dyn StdError>> {
    let expiration = (chrono::Utc::now() + chrono::Duration::minutes(60)).timestamp();

    let claims = Claims {
        username: uname,
        id: uid,
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
    )
    .map_err(|_| TodosError::JwtTokenCreationError.into())
}

pub fn authorize(jwt: &str) -> Result<Claims, TodosError> {
    let decoded = jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|_| TodosError::JwtTokenDecodeError)?;

    Ok(decoded.claims)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let unauth_err = |message| {
            ready(Err(HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": message }))
                .into()))
        };
        if let Some(header) = req.headers().get("Authorization") {
            if let Ok(token) = header.to_str() {
                if let Ok(claims) = authorize(token) {
                    ready(Ok(Self {
                        id: claims.id,
                        username: claims.username,
                    }))
                } else {
                    unauth_err("Token is invalid or expired.")
                }
            } else {
                unauth_err("Auth header is malformed or contains non-ASCII characters.")
            }
        } else {
            unauth_err("Auth header not present.")
        }
    }
}

pub struct TodoIsOfUser {
    pub result: Result<super::models::Todo, TodosError>,
}

impl FromRequest for TodoIsOfUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        match futures::executor::block_on(AuthUser::from_request(req, payload)) {
            Err(e) => ready(Err(e)),
            Ok(claims) => {
                let unauth_err = |message| {
                    ready(Err(HttpResponse::NotFound()
                        .json(serde_json::json!({ "message": message }))
                        .into()))
                };
                if let Ok(todo_id) =
                    futures::executor::block_on(web::Path::<i32>::from_request(req, payload))
                {
                    let pool = futures::executor::block_on(
                        web::Data::<super::DbPool>::from_request(req, payload),
                    )
                    .unwrap();
                    let conn = pool.get().expect("Failed to get db conn from pool.");
                    use super::schema::todos::dsl::*;
                    let result = todos
                        .filter(id.eq(todo_id.into_inner()))
                        .first::<super::models::Todo>(&conn)
                        .map_err(|e| match e {
                            diesel::result::Error::NotFound => TodosError::TodoNotFoundError,
                            _ => TodosError::DieselCrudError,
                        });
                    if let Ok(todo) = &result {
                        if todo.user_id == claims.id {
                            ready(Ok(Self { result }))
                        } else {
                            unauth_err("The todo that you were trying to find does not exist.")
                        }
                    } else {
                        ready(Ok(Self { result }))
                    }
                } else {
                    unreachable!("This should never happen. `todo_id` should always exist when this is used, or else...")
                }
            }
        }
    }
}
