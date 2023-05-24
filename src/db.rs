use sqlx::Executor;
use sqlx::pool::PoolConnection;
use sqlx::postgres::Postgres;
use sqlx::Row;

use crate::errors::{ServiceError, UserError};

pub async fn query_db(id: &str, conn: &mut PoolConnection<Postgres>) -> Result<String, ServiceError> {
    let url = sqlx::query("SELECT url FROM url WHERE shortened = $1")
        .bind(id)
        .fetch_one(&mut *conn)
        .await;

    match url {
        Err(err) => Err(ServiceError::Database(err)),
        Ok(row) if row.is_empty() => {
            Err(ServiceError::User(UserError::NotFound))
        }
        Ok(row) => Ok(row.get(0))
    }
}

pub async fn insert_db(url: &str, shortened: &str, conn: &mut PoolConnection<Postgres>) -> Result<(), ServiceError> {
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

pub async fn create_table(db_conn: &mut PoolConnection<Postgres>){
        db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS url(
            id SERIAL PRIMARY KEY,
            shortened TEXT NOT NULL,
            url TEXT NOT NULL
        )",
        )
        .await
        .unwrap();
}
