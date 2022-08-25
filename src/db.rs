use sqlx::sqlite::{SqlitePoolOptions, Sqlite};
use sqlx::pool::PoolConnection;
use sqlx::Row;

use crate::errors::{ServiceError, UserError};

pub async fn init_db() -> sqlx::Pool<Sqlite> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://urls.db?mode=rwc")
        .await
        .unwrap()
}

pub async fn query_db(id: &str, conn: &mut PoolConnection<Sqlite>) -> Result<String, ServiceError> {
    let url = sqlx::query("SELECT url FROM url WHERE shortened = ?")
        .bind(id)
        .fetch_one(&mut *conn)
        .await;

    match url {
        Err(err) => Err(ServiceError::Database(err)),
        Ok(row) if row.is_empty() => Err(ServiceError::User(UserError::NotFound)),
        Ok(row) => Ok(row.get(0))
    }
}

pub async fn insert_db(url: &str, shortened: &str, conn: &mut PoolConnection<Sqlite>) -> Result<(), ServiceError> {
    let result = sqlx::query("INSERT INTO url (shortened, url) VALUES ($1, $2)")
        .bind(shortened)
        .bind(url)
        .execute(&mut *conn)
        .await;
    
    match result {
        Ok(_code) => Ok(()),
        Err(err) => Err(ServiceError::Database(err))
    }
}
