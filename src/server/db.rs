use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub elo: i32,
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>("SELECT id, email, display_name, elo FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(pool: &PgPool, email: &str) -> Result<DbUser, sqlx::Error> {
    sqlx::query_as::<_, DbUser>(
        "INSERT INTO users (email) VALUES ($1) RETURNING id, email, display_name, elo",
    )
    .bind(email)
    .fetch_one(pool)
    .await
}

pub async fn find_or_create_user(pool: &PgPool, email: &str) -> Result<DbUser, sqlx::Error> {
    if let Some(user) = find_user_by_email(pool, email).await? {
        return Ok(user);
    }
    create_user(pool, email).await
}

pub async fn update_display_name(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET display_name = $1 WHERE id = $2")
        .bind(name)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn store_otp(
    pool: &PgPool,
    email: &str,
    code: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO otp_codes (email, code, expires_at) VALUES ($1, $2, $3)")
        .bind(email)
        .bind(code)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn verify_otp(pool: &PgPool, email: &str, code: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        "UPDATE otp_codes SET used = TRUE \
         WHERE email = $1 AND code = $2 AND used = FALSE AND expires_at > NOW() \
         RETURNING 1",
    )
    .bind(email)
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(token)
        .bind(user_id)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn validate_session(
    pool: &PgPool,
    token: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>(
        "SELECT u.id, u.email, u.display_name, u.elo \
         FROM sessions s JOIN users u ON s.user_id = u.id \
         WHERE s.token = $1 AND s.expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn save_finished_game(
    pool: &PgPool,
    white_id: Uuid,
    black_id: Uuid,
    result: &str,
    result_detail: &str,
    moves_json: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO games (white_id, black_id, result, result_detail, moves_json, finished_at) \
         VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(white_id)
    .bind(black_id)
    .bind(result)
    .bind(result_detail)
    .bind(moves_json)
    .execute(pool)
    .await?;
    Ok(())
}
