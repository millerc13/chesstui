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
    pub connected_users: Arc<DashMap<uuid::Uuid, ()>>,
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
    use sqlx::postgres::{PgConnectOptions, PgSslMode};
    use std::str::FromStr;
    let opts = PgConnectOptions::from_str(&database_url)?
        .ssl_mode(PgSslMode::Require)
        .options([("search_path", "public")]);
    let pool = sqlx::PgPool::connect_with(opts).await?;

    // Run migrations — execute each statement individually
    tracing::info!("Running migrations...");
    for path in &["./migrations/001_initial.sql", "./migrations/002_accounts.sql"] {
        match tokio::fs::read_to_string(path).await {
            Ok(sql) => {
                for statement in sql.split(';') {
                    let stmt = statement.trim();
                    if stmt.is_empty() || stmt.starts_with("--") {
                        continue;
                    }
                    if let Err(e) = sqlx::query(stmt).execute(&pool).await {
                        // Ignore "already exists" errors
                        let msg = e.to_string();
                        if !msg.contains("already exists") && !msg.contains("duplicate") {
                            tracing::warn!("Migration statement error: {}", msg);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not read migration {}: {} (tables may already exist)", path, e);
            }
        }
    }

    let state = Arc::new(AppState {
        pool,
        resend_api_key,
        matchmaker: Arc::new(Mutex::new(matchmaking::Matchmaker::new())),
        games: Arc::new(DashMap::new()),
        otp_rate_limits: Arc::new(Mutex::new(HashMap::new())),
        connected_users: Arc::new(DashMap::new()),
    });

    let app = axum::Router::new()
        .route("/ws", axum::routing::get(ws::ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("ChessTUI server listening on {}", bind);
    axum::serve(listener, app).await?;

    Ok(())
}
