use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "Database error")]
    Database(sqlx::Error),

    #[display(fmt = "Server error")]
    Server(axum::Error),

    User(UserError)
}

#[derive(Debug, Display, Error)]
pub enum UserError {
    #[display(fmtr "This shortened link doesn't exist!")]
    NotFound,

    #[display(fmt= "This is not a valid Url!")]
    InvalidUrl
}
