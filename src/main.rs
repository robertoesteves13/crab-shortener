use axum::Json;
use axum::extract::{State, Path};
use axum::http::StatusCode;
use serde::Deserialize;

use shuttle_axum::ShuttleAxum;
use shuttle_runtime::CustomError;

use axum::routing::{Router, get, post};
use axum::response::IntoResponse;

use std::sync::Arc;

use sqlx::postgres::PgPool;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use validator::Validate;

mod errors;

mod db;
use db::*;

struct ServiceState {
    db_conn: PgPool,
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

async fn get_url(
    Path(id): Path<String>,
    State(state): State<Arc<ServiceState>>
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_conn.acquire().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "The database is currently offline").into_response();
    };

    let Ok(string) = query_db(&id, &mut conn).await else {
        return (StatusCode::NOT_FOUND, "This link isn't registered").into_response();
    };

    (StatusCode::MOVED_PERMANENTLY, string).into_response()
}

#[axum::debug_handler]
async fn shorten_url(
    State(state): State<Arc<ServiceState>>,
    Json(payload): Json<UrlRequest>
) -> impl IntoResponse {
    let shortened = rand_string();

    let Ok(mut conn) = state.db_conn.acquire().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "The database is currently offline").into_response();
    };

    if let Ok(_) = insert_db(&payload.url, &shortened, &mut conn).await {
        (StatusCode::OK, shortened).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "An unknown error ocurred on the server").into_response()
    }


}

async fn index() -> &'static str {
    include_str!("../view/index.html")
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> shuttle_axum::ShuttleAxum {
    create_table(&mut pool.acquire().await.unwrap()).await;

    let router = Router::new()
        .route("/", get(index))
        .route("/shorten_url", post(shorten_url))
        .route("/:id", get(get_url))
        .with_state(Arc::new(ServiceState{ db_conn: pool }));

    router.into()
}
