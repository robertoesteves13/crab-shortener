use serde::Deserialize;

use actix_files as fs;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use actix_web::http::{StatusCode};
use actix_web::http::header;
use actix_web::http::header::HeaderValue;

use sqlx::sqlite::SqlitePool;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use validator::Validate;

mod errors;
use errors::*;

mod db;
use db::*;

struct ServiceState {
    db_conn: SqlitePool,
    domain: String,
    port: Option<u16>,
}

#[derive(Deserialize, Validate)]
struct UrlRequest {
    #[validate(url)]
    url: String,
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

#[get("/{id}")]
async fn get_url(
    id: web::Path<String>,
    data: web::Data<ServiceState>,
) -> Result<impl Responder, ServiceError> {
    let mut conn = data.db_conn.acquire().await?;

    let string = query_db(&id.into_inner(), &mut conn).await?;

    let mut res = HttpResponse::new(StatusCode::MOVED_PERMANENTLY);
    let header = res.headers_mut();
    header.append(header::LOCATION, HeaderValue::from_str(&string).unwrap());
    
    Ok(res)
}

#[post("/shorten-url")]
async fn shorten_url(
    req: web::Json<UrlRequest>,
    data: web::Data<ServiceState>,
) -> Result<impl Responder, ServiceError> {
    let shortened = rand_string();

    if let Err(_err) = req.validate() {
        return Err(ServiceError::User(UserError::InvalidUrl));
    }

    let mut conn = data.db_conn.acquire().await?;
    insert_db(&req.url, &shortened, &mut conn).await?;

    if let Some(port) = data.port {
        Ok(format!("http://{}{}/{shortened}", data.domain, port))
    } else {
        Ok(format!("http://{}/{shortened}", data.domain))
    }
}

#[get("/")]
async fn index() -> Result<fs::NamedFile, std::io::Error> {
    fs::NamedFile::open("view/index.html")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_state = web::Data::new(ServiceState {
        db_conn: init_db().await,
        domain: std::env::var("DOMAIN").unwrap_or("localhost".to_owned()),
        port: match std::env::var("PORT") {
            Ok(port) =>  {
                if let Ok(num) = port.parse::<u16>() {
                    Some(num)
                } else {
                    panic!("The port specified is not a number")
                }
            }
            Err(_) => None
        }
    });

    create_table(&mut service_state.db_conn.acquire().await.unwrap()).await;
    let port = service_state.port;

    HttpServer::new(move || {
        App::new()
            .app_data(service_state.clone())
            .service(index)
            .service(shorten_url)
            .service(get_url)
    })
    .bind(("127.0.0.1", port.unwrap_or(80)))?
    .run()
    .await
}
