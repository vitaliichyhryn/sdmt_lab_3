mod handlers;
mod models;
mod database;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/cointoss".to_string());

    let db_pool = database::setup_database(&database_url).await?;
    database::run_migrations(&db_pool).await?;

    let app_state = Arc::new(db_pool);

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/toss", post(handlers::toss_coin))
        .route("/history", get(handlers::get_history))
        .route("/api/toss", post(handlers::api_toss_coin))
        .route("/api/history", get(handlers::api_get_history))
        .with_state(app_state)
        .layer(ServiceBuilder::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
