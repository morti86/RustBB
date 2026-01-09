use std::env::VarError;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use oauth2::{ErrorResponse, url::ParseError};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::http::StatusCode;
use validator::ValidationErrors; 

const DEFAULT_ERROR: (StatusCode, &str) = (StatusCode::BAD_REQUEST, "Dire stuff here");

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ForumError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("OAuth2")]
    OAuth2(String),
    #[error("HTTP")]
    Http((u16, String)),
    #[error("Invalid token")]
    InvalidToken,
    #[error("No such user: {0}")]
    NoSuchUser(String),
    #[error("Permission denied")]
    Forbidden,
    #[error("Server error")]
    ServerError(String),
    #[error("Bad request")]
    BadRequest,
    #[error("Old password")]
    OldPassword,
    #[error("Token error: {0}")]
    Token(String),
    #[error("Auth")]
    Auth(String),
    #[error("Validation error")]
    Validation(#[from] ValidationErrors),
    #[error("Error: {0}")]
    Forum(String),
    #[error("IO error")]
    Io(String),
    #[error("Smtp error: {0}")]
    Smtp(String),
    #[error("Empty password")]
    EmptyPassword,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Empty hash format")]
    InvalidHashFormat,
    #[error("Argon password hash error")]
    Argon2(String),
    #[error("TryResult thing OK")]
    Ok,
    #[error("TryResult Lock {0}")]
    Lock(String),
    #[error("You have been banned")]
    Banned,
    // Add more variants as needed
}

impl From<sqlx::Error> for ForumError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<ParseError> for ForumError {
    fn from(value: ParseError) -> Self {
        Self::OAuth2(value.to_string())
    }
}

impl From<reqwest::Error> for ForumError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http((value.status().map(|s| s.as_u16()).unwrap_or(500), value.to_string()))
    }
}

impl From<&str> for ForumError {
    fn from(value: &str) -> Self {
        Self::OAuth2(value.to_string())
    }
}

impl<E1, E2> From<oauth2::RequestTokenError<E1, E2>> for ForumError
where
    E1: std::error::Error + 'static,
    E2: oauth2::ErrorResponse + 'static,
{
    fn from(value: oauth2::RequestTokenError<E1, E2>) -> Self {
        Self::OAuth2(format!("{:?}", value))
    }
}

impl From<(axum::http::StatusCode,axum::Error)> for ForumError {
    fn from(value: (axum::http::StatusCode,axum::Error)) -> Self {
        Self::Http((value.0.as_u16(),value.1.to_string()))
    }
}

impl From<(axum::http::StatusCode,String)> for ForumError {
    fn from(value: (axum::http::StatusCode,String)) -> Self {
        Self::Http((value.0.as_u16(),value.1))
    }
}

impl From<(axum::http::StatusCode,&str)> for ForumError {
    fn from(value: (axum::http::StatusCode,&str)) -> Self {
        Self::Http((value.0.as_u16(),value.1.to_string()))
    }
}

impl From<jsonwebtoken::errors::Error> for ForumError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::Token(value.to_string())
    }
}

impl From<uuid::Error> for ForumError {
    fn from(value: uuid::Error) -> Self {
        Self::ServerError(value.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ForumError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Forum(value.to_string())
    }
}

impl From<VarError> for ForumError {
    fn from(value: VarError) -> Self {
        Self::ServerError(value.to_string())
    }
}

impl From<std::io::Error> for ForumError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for ForumError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self::Smtp(value.to_string())
    }
}

impl From<lettre::error::Error> for ForumError {
    fn from(value: lettre::error::Error) -> Self {
        Self::Smtp(value.to_string())
    }
}

impl From<lettre::address::AddressError> for ForumError {
    fn from(value: lettre::address::AddressError) -> Self {
        Self::Smtp(value.to_string())
    }
}

impl From<std::num::ParseIntError> for ForumError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ServerError(value.to_string())
    }
}

impl From<argon2::password_hash::Error> for ForumError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Argon2(value.to_string())
    }
}

impl From<dashmap::try_result::TryResult<crate::UserSession>> for ForumError {
    fn from(value: dashmap::try_result::TryResult<crate::UserSession>) -> Self {
        match value {
            dashmap::try_result::TryResult::Present(_) => Self::Ok,
            dashmap::try_result::TryResult::Absent => Self::Lock("Absent".to_string()),
            dashmap::try_result::TryResult::Locked => Self::Lock("Locked".to_string()),
        }
    }
}

impl From<axum::http::Error> for ForumError {
    fn from(value: axum::http::Error) -> Self {
        Self::ServerError(value.to_string())
    }
}

impl IntoResponse for ForumError {
    fn into_response(self) -> Response {
        match self {
            ForumError::Http((code,message)) => (StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), message),
            ForumError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ForumError::OAuth2(v) => (StatusCode::UNAUTHORIZED,v),
            ForumError::Banned => (StatusCode::UNAUTHORIZED, self.to_string()),
            ForumError::Database(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ForumError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            ForumError::Token(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::NoSuchUser(v) => (StatusCode::UNAUTHORIZED, format!("No such user: {}", v) ),
            ForumError::Forbidden => (StatusCode::FORBIDDEN, self.to_string() ),
            ForumError::ServerError(v) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Server error: {}", v)),
            ForumError::BadRequest => (StatusCode::BAD_REQUEST, self.to_string()),
            ForumError::OldPassword => (StatusCode::BAD_REQUEST, self.to_string()),
            ForumError::Auth(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::Forum(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::Validation(v) => (StatusCode::BAD_REQUEST, v.to_string()),
            ForumError::Io(v) => (StatusCode::INTERNAL_SERVER_ERROR, v),
            ForumError::Smtp(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::EmptyPassword => (StatusCode::BAD_REQUEST, self.to_string()),
            ForumError::InvalidPassword => (StatusCode::BAD_REQUEST, self.to_string()),
            ForumError::InvalidHashFormat => (StatusCode::BAD_REQUEST, self.to_string()),
            ForumError::Argon2(v) => (StatusCode::BAD_REQUEST, v),
            ForumError::Ok => (StatusCode::OK, self.to_string()),
            ForumError::Lock(v) => (StatusCode::LOCKED, v),
        }.into_response()
    }
}


impl ErrorResponse for ForumError {

}

pub type ForumResult<T> = Result<T, ForumError>;
