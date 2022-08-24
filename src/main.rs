use serde::{Deserialize, Serialize};

use actix_files as fs;
use actix_web::http::header::HeaderValue;
use actix_web::http::{header, StatusCode};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Executor;
use sqlx::Row;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use derive_more::{Display, Error};

struct ServiceState {
    db_conn: SqlitePool,
}

#[derive(Debug, Display, Error)]
enum ServiceError {
    #[display(fmt = "Database error")]
    Database(sqlx::Error),

    #[display(fmt = "Server error")]
    Server(actix_web::Error),
}

impl actix_web::error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            ServiceError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

#[derive(Serialize, Deserialize)]
struct UrlRequest {
    url: String,
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect()
}

#[get("/{id}")]
async fn get_url(
    id: web::Path<String>,
    data: web::Data<ServiceState>,
) -> Result<impl Responder, ServiceError> {
    let mut conn = data.db_conn.acquire().await?;
    let url = sqlx::query("SELECT url FROM url WHERE id = ?")
        .bind(id.into_inner())
        .fetch_one(&mut *conn)
        .await;

    match url {
        Ok(row) => {
            let string: &str = row.get(0);

            let mut res = HttpResponse::new(StatusCode::MOVED_PERMANENTLY);
            let header = res.headers_mut();
            header.append(header::LOCATION, HeaderValue::from_str(string).unwrap());

            Ok(res)
        }
        Err(_err) => {
            let res = HttpResponse::new(StatusCode::NOT_FOUND);
            Ok(res)
        }
    }
}

#[post("/shorten-url")]
async fn shorten_url(
    req: web::Form<UrlRequest>,
    data: web::Data<ServiceState>,
) -> Result<impl Responder, ServiceError> {
    let shortened = rand_string();

    let mut conn = data.db_conn.acquire().await?;
    sqlx::query("INSERT INTO url (id, url) VALUES ($1, $2)")
        .bind(&shortened)
        .bind(&req.url)
        .execute(&mut *conn)
        .await?;

    Ok(format!("http://localhost:8080/{shortened}"))
}

#[get("/")]
async fn index() -> Result<fs::NamedFile, std::io::Error> {
    fs::NamedFile::open("view/index.html")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_state = web::Data::new(ServiceState {
        db_conn: SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://urls.db?mode=rwc")
            .await
            .unwrap(),
    });

    service_state
        .db_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS url(
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL
        )",
        )
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(service_state.clone())
            .service(index)
            .service(shorten_url)
            .service(get_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
