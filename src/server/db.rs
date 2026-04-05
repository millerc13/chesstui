use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub elo: i32,
    pub password_hash: Option<String>,
    pub preferences: serde_json::Value,
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    sqlx::query_as::<_, DbUser>("SELECT id, email, display_name, elo, password_hash, preferences FROM public.users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(pool: &PgPool, email: &str) -> Result<DbUser, sqlx::Error> {
    sqlx::query_as::<_, DbUser>(
        "INSERT INTO public.users(email) VALUES ($1) RETURNING id, email, display_name, elo, password_hash, preferences",
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
    sqlx::query("UPDATE public.users SET display_name = $1 WHERE id = $2")
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
    sqlx::query("INSERT INTO public.otp_codes(email, code, expires_at) VALUES ($1, $2, $3)")
        .bind(email)
        .bind(code)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn verify_otp(pool: &PgPool, email: &str, code: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i32>(
        "UPDATE public.otp_codes SET used = TRUE \
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
    sqlx::query("INSERT INTO public.sessions(token, user_id, expires_at) VALUES ($1, $2, $3)")
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
        "SELECT u.id, u.email, u.display_name, u.elo, u.password_hash, u.preferences \
         FROM public.sessions s JOIN public.users u ON s.user_id = u.id \
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
        "INSERT INTO public.games(white_id, black_id, result, result_detail, moves_json, finished_at) \
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

pub async fn set_password(pool: &PgPool, user_id: Uuid, password_hash: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("UPDATE public.users SET password_hash = $1 WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_user_for_login(pool: &PgPool, email: &str) -> Result<Option<DbUser>, Box<dyn std::error::Error + Send + Sync>> {
    let user = sqlx::query_as::<_, DbUser>("SELECT id, email, display_name, elo, password_hash, preferences FROM public.users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn get_preferences(pool: &PgPool, user_id: Uuid) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let row: (serde_json::Value,) = sqlx::query_as("SELECT preferences FROM public.users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn update_preferences(pool: &PgPool, user_id: Uuid, preferences: &serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("UPDATE public.users SET preferences = $1 WHERE id = $2")
        .bind(preferences)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_friend(pool: &PgPool, user_id: Uuid, friend_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("INSERT INTO public.friends (user_id, friend_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(friend_id)
        .execute(pool)
        .await?;
    // Also add reverse direction
    sqlx::query("INSERT INTO public.friends (user_id, friend_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(friend_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_friend(pool: &PgPool, user_id: Uuid, friend_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("DELETE FROM public.friends WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)")
        .bind(user_id)
        .bind(friend_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_friends(pool: &PgPool, user_id: Uuid) -> Result<Vec<DbUser>, Box<dyn std::error::Error + Send + Sync>> {
    let friends = sqlx::query_as::<_, DbUser>("SELECT u.id, u.email, u.display_name, u.elo, u.password_hash, u.preferences FROM public.friends f JOIN public.users u ON u.id = f.friend_id WHERE f.user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(friends)
}
