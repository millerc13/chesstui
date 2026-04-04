use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use dashmap::DashMap;

pub mod auth;
pub mod db;
pub mod game_room;
pub mod matchmaking;
pub mod ws;

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub resend_api_key: String,
    pub matchmaker: Arc<Mutex<matchmaking::Matchmaker>>,
    pub games: Arc<DashMap<String, Arc<Mutex<game_room::GameRoom>>>>,
    pub otp_rate_limits: Arc<Mutex<HashMap<String, Instant>>>,
}

pub async fn run(
    bind: String,
    database_url: String,
    resend_api_key: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Connecting to database...");
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // Run migrations by reading and executing the SQL file
    tracing::info!("Running migrations...");
    let migration_sql = tokio::fs::read_to_string("./migrations/001_initial.sql").await?;
    sqlx::query(&migration_sql).execute(&pool).await.ok(); // OK if tables already exist

    let state = Arc::new(AppState {
        pool,
        resend_api_key,
        matchmaker: Arc::new(Mutex::new(matchmaking::Matchmaker::new())),
        games: Arc::new(DashMap::new()),
        otp_rate_limits: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = axum::Router::new()
        .route("/ws", axum::routing::get(ws::ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("ChessTUI server listening on {}", bind);
    axum::serve(listener, app).await?;

    Ok(())
}
