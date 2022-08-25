use sqlx::Executor;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqlitePoolOptions, Sqlite};
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

pub async fn create_table(db_conn: &mut PoolConnection<Sqlite>){
        db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS url(
            id INTEGER PRIMARY KEY,
            shortened TEXT NOT NULL,
            url TEXT NOT NULL
        )",
        )
        .await
        .unwrap();

}

#[cfg(test)]
mod db_tests {
    use sqlx::Sqlite;
    use super::{init_db, query_db, insert_db, create_table};
    use actix_web::test;

    async fn setup_tables() -> sqlx::pool::PoolConnection<Sqlite> {
        let mut conn = init_db().await.acquire().await.unwrap();
        create_table(&mut conn).await;

        conn
    }

    #[test]
    async fn init_db_without_file() {
        std::fs::remove_file("urls.db").unwrap();
        init_db().await.acquire().await.unwrap();
    }

    #[test]
    async fn insert_and_query_data() {
        let mut conn = setup_tables().await;

        let (url, shortened) = ("https://www.gnu.org/", "9F9nd5");
        insert_db(&url, &shortened, &mut conn).await.unwrap();

        let shortened = "9F9nd5";
        let result = query_db(shortened, &mut conn).await.unwrap();

        assert_eq!("https://www.gnu.org/", result.as_str());
    }
}
