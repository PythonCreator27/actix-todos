use std::{error::Error, fmt::Display};

use actix_web::error::BlockingError;

#[derive(Debug)]
pub enum TodosError {
    TodoNotFoundError,
    JwtTokenCreationError,
    DieselCrudError,
    JwtTokenDecodeError,
    BadCreds,
}

impl Error for TodosError {}

impl Display for TodosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TodoNotFoundError => {
                write!(f, "todo not found")
            }
            Self::DieselCrudError => {
                write!(f, "database operations failed")
            }
            Self::JwtTokenCreationError => {
                write!(f, "JWT token creation failed")
            }
            Self::JwtTokenDecodeError => {
                write!(f, "JWT token decoding failed")
            }
            Self::BadCreds => {
                write!(f, "bad credentials")
            }
        }
    }
}

impl From<BlockingError<TodosError>> for TodosError {
    fn from(e: BlockingError<TodosError>) -> Self {
        if let BlockingError::Error(e) = e {
            e
        } else {
            panic!()
        }
    }
}
