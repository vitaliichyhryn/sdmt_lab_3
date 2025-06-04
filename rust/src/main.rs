use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use rand::random;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Debug, Serialize, Deserialize)]
struct CoinToss {
    id: i32,
    result: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct TossResponse {
    result: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug)]
struct AppError(sqlx::Error);

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError(err)
    }
}

impl From<AppError> for StatusCode {
    fn from(_: AppError) -> Self {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

type AppState = Arc<PgPool>;
type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = create_database_connection().await?;
    setup_database(&pool).await?;

    let app_state = Arc::new(pool);
    let app = create_router().with_state(app_state);

    start_server(app).await?;
    Ok(())
}

async fn create_database_connection() -> anyhow::Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:password@localhost:5432/heads_or_tails".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))
}

async fn setup_database(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS coin_tosses (
            id SERIAL PRIMARY KEY,
            result VARCHAR(10) NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create table: {}", e))?;

    Ok(())
}

fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/toss", post(toss_coin))
        .route("/api/history", get(get_history))
        .fallback_service(ServeDir::new("static"))
}

async fn start_server(app: Router) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port 3000: {}", e))?;

    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;

    Ok(())
}

async fn toss_coin(
    State(pool): State<AppState>,
) -> std::result::Result<Json<TossResponse>, StatusCode> {
    let result = generate_coin_result();
    let timestamp = Utc::now();

    insert_coin_toss(&pool, &result, timestamp)
        .await
        .map(|_| {
            Json(TossResponse {
                result: result.to_string(),
                timestamp,
            })
        })
        .map_err(StatusCode::from)
}

async fn get_history(
    State(pool): State<AppState>,
) -> std::result::Result<Json<Vec<CoinToss>>, StatusCode> {
    fetch_recent_tosses(&pool)
        .await
        .map(Json)
        .map_err(StatusCode::from)
}

fn generate_coin_result() -> &'static str {
    if random::<bool>() { "heads" } else { "tails" }
}

async fn insert_coin_toss(pool: &PgPool, result: &str, timestamp: DateTime<Utc>) -> Result<()> {
    sqlx::query("INSERT INTO coin_tosses (result, timestamp) VALUES ($1, $2)")
        .bind(result)
        .bind(timestamp)
        .execute(pool)
        .await?;

    Ok(())
}

async fn fetch_recent_tosses(pool: &PgPool) -> Result<Vec<CoinToss>> {
    let rows = sqlx::query(
        "SELECT id, result, timestamp FROM coin_tosses ORDER BY timestamp DESC LIMIT 20",
    )
    .fetch_all(pool)
    .await?;

    let tosses = rows
        .into_iter()
        .map(|row| CoinToss {
            id: row.get("id"),
            result: row.get("result"),
            timestamp: row.get("timestamp"),
        })
        .collect();

    Ok(tosses)
}
