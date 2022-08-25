use actix_web::HttpResponse;
use actix_web::http::{header, StatusCode};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "Database error")]
    Database(sqlx::Error),

    #[display(fmt = "Server error")]
    Server(actix_web::Error),

    User(UserError)
}

impl actix_web::error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::User(err) => err.status_code(),
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<actix_web::Error> for ServiceError {
    fn from(error: actix_web::Error) -> Self {
        Self::Server(error)
    }
}

impl From<UserError> for ServiceError {
    fn from(error: UserError) -> Self {
        Self::User(error)
    }
}

#[derive(Debug, Display, Error)]
pub enum UserError {
    #[display(fmt= "This shortened link doesn't exist!")]
    NotFound,

    #[display(fmt= "This is not a valid Url!")]
    InvalidUrl
}

impl actix_web::error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidUrl => StatusCode::BAD_REQUEST,
        }
    }
}
