use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "Database error")]
    Database(sqlx::Error),

    User(UserError)
}

#[derive(Debug, Display, Error)]
pub enum UserError {
    #[display(fmtr "This shortened link doesn't exist!")]
    NotFound,
}
