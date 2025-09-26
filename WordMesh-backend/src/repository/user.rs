use crate::domain::{HashedPassword, User, UserDomainError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, RepositoryError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password_hash: HashedPassword,
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("domain error: {0}")]
    Domain(#[from] UserDomainError),
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, RepositoryError> {
        let record = sqlx::query(
            r#"
            INSERT INTO users (username, password, created_at)
            VALUES ($1, $2, $3)
            RETURNING id, username, password, created_at
            "#,
        )
        .bind(new_user.username)
        .bind(new_user.password_hash.as_str())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        map_row_to_user(record)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let maybe_row = sqlx::query(
            r#"
            SELECT id, username, password, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        maybe_row
            .map(map_row_to_user)
            .transpose()
    }

    async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, RepositoryError> {
        let maybe_row = sqlx::query(
            r#"
            SELECT id, username, password, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        maybe_row
            .map(map_row_to_user)
            .transpose()
    }
}

fn map_row_to_user(row: sqlx::postgres::PgRow) -> Result<User, RepositoryError> {
    let id: i64 = row.try_get("id")?;
    let username: String = row.try_get("username")?;
    let password: String = row.try_get("password")?;
    let created_at: DateTime<Utc> = row.try_get("created_at")?;

    let hashed_password = HashedPassword::new(password).map_err(UserDomainError::from)?;

    Ok(User::new(id, username, hashed_password, created_at)?)
}
