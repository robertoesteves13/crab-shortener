use sqlx::Row;
use std::str::FromStr;

use serde::{Serialize, Deserialize};

use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use actix_web::http::header::HeaderValue;
use actix_web::http::{header, StatusCode};
use actix_files as fs;

use sqlx::ConnectOptions;
use sqlx::Executor;
use sqlx::Connection;
use sqlx::sqlite::SqliteConnection;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Serialize, Deserialize)]
struct UrlRequest {
    url: String
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect()
}

async fn get_db_connection() -> Result<SqliteConnection, sqlx::Error> {
    sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://urls.db").unwrap()
        .create_if_missing(true)
        .connect().await
}


#[get("/{id}")]
async fn get_url(id: web::Path<String>) -> impl Responder {
    let mut conn = get_db_connection().await.unwrap();
    let url = sqlx::query("SELECT url FROM url WHERE id = ?")
        .bind(id.into_inner())
        .fetch_one(&mut conn).await;

    if let Ok(row) = url {
        let string: &str = row.get(0);

        let mut res = HttpResponse::new(StatusCode::MOVED_PERMANENTLY);
        let header = res.headers_mut();
        header.append(header::LOCATION, HeaderValue::from_str(string).unwrap());

        res
    } else {
        let res = HttpResponse::new(StatusCode::NOT_FOUND);
        res
    }
}

#[post("/shorten-url")]
async fn shorten_url(req: web::Form<UrlRequest>) -> impl Responder {
    let shortened = rand_string();

    let mut conn = get_db_connection().await.unwrap();
    sqlx::query("INSERT INTO url (id, url) VALUES ($1, $2)")
        .bind(&shortened)
        .bind(&req.url)
        .execute(&mut conn).await.unwrap();

    format!("http://localhost:8080/{shortened}")
}

#[get("/")]
async fn index() -> Result<fs::NamedFile, std::io::Error> {
    fs::NamedFile::open("view/index.html")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut conn = get_db_connection().await.unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS url(
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL
            )").await.unwrap();
    conn.close();

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(shorten_url)
            .service(get_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
