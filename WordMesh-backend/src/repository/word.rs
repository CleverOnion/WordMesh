use crate::domain::word::{UserSense, UserSenseError, UserWord, UserWordError};
use crate::domain::{CanonicalKey, CanonicalKeyError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row, postgres::PgRow};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WordRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("user word domain error: {0}")]
    UserWord(#[from] UserWordError),
    #[error("user sense domain error: {0}")]
    UserSense(#[from] UserSenseError),
    #[error("canonical key error: {0}")]
    Canonical(#[from] CanonicalKeyError),
}

#[derive(Debug, Clone)]
pub struct WordRecord {
    pub id: i64,
    pub text: String,
    pub canonical_key: CanonicalKey,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserWordAggregate {
    pub word: WordRecord,
    pub user_word: UserWord,
}

#[derive(Debug, Clone)]
pub struct UpsertUserWord {
    pub user_id: i64,
    pub word_text: String,
    pub canonical_key: CanonicalKey,
    pub tags: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewUserSense {
    pub user_word_id: i64,
    pub text: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SenseUpdate {
    pub text: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
    pub note: Option<Option<String>>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum SearchScope {
    Word,
    Sense,
    Both,
}

impl Default for SearchScope {
    fn default() -> Self {
        SearchScope::Both
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub user_id: i64,
    pub query: String,
    pub scope: SearchScope,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait]
pub trait WordRepository {
    async fn upsert_word(
        &self,
        canonical: &CanonicalKey,
        text: &str,
    ) -> Result<WordRecord, WordRepositoryError>;
    async fn upsert_user_word(
        &self,
        payload: UpsertUserWord,
    ) -> Result<UserWordAggregate, WordRepositoryError>;
    async fn find_user_word(
        &self,
        user_id: i64,
        user_word_id: i64,
    ) -> Result<Option<UserWordAggregate>, WordRepositoryError>;
    async fn remove_user_word(
        &self,
        user_id: i64,
        user_word_id: i64,
    ) -> Result<(), WordRepositoryError>;

    async fn add_user_sense(&self, sense: NewUserSense) -> Result<UserSense, WordRepositoryError>;
    async fn update_user_sense(
        &self,
        user_id: i64,
        sense_id: i64,
        update: SenseUpdate,
    ) -> Result<UserSense, WordRepositoryError>;
    async fn remove_user_sense(
        &self,
        user_id: i64,
        sense_id: i64,
    ) -> Result<UserSense, WordRepositoryError>;

    async fn search(
        &self,
        params: SearchParams,
    ) -> Result<Vec<UserWordAggregate>, WordRepositoryError>;
}

#[derive(Clone)]
pub struct PgWordRepository {
    pool: PgPool,
}

impl PgWordRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_word_row(row: &PgRow) -> Result<WordRecord, WordRepositoryError> {
        Ok(WordRecord {
            id: row.try_get("id")?,
            text: row.try_get("text")?,
            canonical_key: CanonicalKey::new(row.try_get::<String, _>("canonical_key")?)?,
            created_at: row.try_get("created_at")?,
        })
    }

    fn map_user_word(row: &PgRow) -> Result<UserWord, WordRepositoryError> {
        UserWord::from_parts(
            Some(row.try_get("user_word_id")?),
            row.try_get("user_id")?,
            row.try_get("word_id")?,
            row.try_get("tags")?,
            row.try_get("note")?,
            Vec::new(),
            row.try_get("user_word_created_at")?,
        )
        .map_err(WordRepositoryError::from)
    }

    fn map_senses(value: JsonValue) -> Result<Vec<UserSense>, WordRepositoryError> {
        let senses: Vec<JsonSenseRow> = serde_json::from_value(value).unwrap_or_default();
        let mut result = Vec::with_capacity(senses.len());
        for sense_row in senses {
            result.push(UserSense::from_parts(
                sense_row.id,
                sense_row.text,
                sense_row.is_primary,
                sense_row.sort_order,
                sense_row.note,
                sense_row.created_at,
            )?);
        }
        Ok(result)
    }

    fn build_aggregate(row: PgRow) -> Result<UserWordAggregate, WordRepositoryError> {
        let mut user_word = Self::map_user_word(&row)?;
        let senses_value: JsonValue = row.try_get("senses")?;
        let senses = Self::map_senses(senses_value)?;
        for sense in senses {
            user_word.add_sense(sense)?;
        }

        let word = WordRecord {
            id: row.try_get("word_id")?,
            text: row.try_get("word_text")?,
            canonical_key: CanonicalKey::new(row.try_get::<String, _>("word_canonical")?)?,
            created_at: row.try_get("word_created_at")?,
        };

        Ok(UserWordAggregate { word, user_word })
    }

    fn aggregate_query() -> &'static str {
        r#"
        SELECT
            uw.id                AS user_word_id,
            uw.user_id,
            uw.word_id,
            uw.tags,
            uw.note,
            uw.created_at        AS user_word_created_at,
            w.text               AS word_text,
            w.canonical_key      AS word_canonical,
            w.created_at         AS word_created_at,
            COALESCE(
                json_agg(
                    json_build_object(
                        'id', us.id,
                        'text', us.text,
                        'is_primary', us.is_primary,
                        'sort_order', us.sort_order,
                        'note', us.note,
                        'created_at', us.created_at
                    )
                    ORDER BY us.sort_order
                ) FILTER (WHERE us.id IS NOT NULL),
                '[]'
            ) AS senses
        FROM user_words uw
        JOIN words w ON w.id = uw.word_id
        LEFT JOIN user_senses us ON us.user_word_id = uw.id
        WHERE uw.user_id = $1
        "#
    }

    fn canonical_like_pattern(text: &str) -> String {
        text.trim().to_lowercase().replace(' ', "-")
    }
}

#[derive(Debug, Deserialize)]
struct JsonSenseRow {
    id: Option<i64>,
    text: String,
    is_primary: bool,
    sort_order: i32,
    note: Option<String>,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl WordRepository for PgWordRepository {
    async fn upsert_word(
        &self,
        canonical: &CanonicalKey,
        text: &str,
    ) -> Result<WordRecord, WordRepositoryError> {
        let row = sqlx::query(
            r#"
            INSERT INTO words (text, canonical_key)
            VALUES ($1, $2)
            ON CONFLICT (canonical_key)
            DO UPDATE SET text = EXCLUDED.text
            RETURNING id, text, canonical_key, created_at
            "#,
        )
        .bind(text)
        .bind(canonical.as_str())
        .fetch_one(&self.pool)
        .await?;

        Self::map_word_row(&row)
    }

    async fn upsert_user_word(
        &self,
        payload: UpsertUserWord,
    ) -> Result<UserWordAggregate, WordRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let word_row = sqlx::query(
            r#"
            INSERT INTO words (text, canonical_key)
            VALUES ($1, $2)
            ON CONFLICT (canonical_key) DO UPDATE SET text = EXCLUDED.text
            RETURNING id, text, canonical_key, created_at
            "#,
        )
        .bind(&payload.word_text)
        .bind(payload.canonical_key.as_str())
        .fetch_one(&mut *tx)
        .await?;
        let word = Self::map_word_row(&word_row)?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO user_words (user_id, word_id, tags, note)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, word_id)
            DO UPDATE SET tags = EXCLUDED.tags, note = EXCLUDED.note
            RETURNING id
            "#,
        )
        .bind(payload.user_id)
        .bind(word.id)
        .bind(&payload.tags)
        .bind(&payload.note)
        .fetch_one(&mut *tx)
        .await?;
        let user_word_id: i64 = inserted.try_get("id")?;

        tx.commit().await?;
        self.find_user_word(payload.user_id, user_word_id)
            .await?
            .ok_or_else(|| WordRepositoryError::Database(sqlx::Error::RowNotFound))
    }

    async fn find_user_word(
        &self,
        user_id: i64,
        user_word_id: i64,
    ) -> Result<Option<UserWordAggregate>, WordRepositoryError> {
        let sql = format!(
            "{} AND uw.id = $2 GROUP BY uw.id, w.id",
            Self::aggregate_query()
        );
        let maybe_row = sqlx::query(&sql)
            .bind(user_id)
            .bind(user_word_id)
            .fetch_optional(&self.pool)
            .await?;
        match maybe_row {
            Some(row) => Ok(Some(Self::build_aggregate(row)?)),
            None => Ok(None),
        }
    }

    async fn remove_user_word(
        &self,
        user_id: i64,
        user_word_id: i64,
    ) -> Result<(), WordRepositoryError> {
        sqlx::query(
            r#"
            DELETE FROM user_words
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(user_word_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn add_user_sense(&self, sense: NewUserSense) -> Result<UserSense, WordRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO user_senses (user_word_id, text, is_primary, sort_order, note)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, text, is_primary, sort_order, note, created_at
            "#,
        )
        .bind(sense.user_word_id)
        .bind(&sense.text)
        .bind(sense.is_primary)
        .bind(sense.sort_order)
        .bind(&sense.note)
        .fetch_one(&mut *tx)
        .await?;

        let mut created = UserSense::from_parts(
            Some(row.try_get("id")?),
            row.try_get("text")?,
            row.try_get("is_primary")?,
            row.try_get("sort_order")?,
            row.try_get("note")?,
            row.try_get("created_at")?,
        )?;

        if created.is_primary {
            sqlx::query(
                r#"
                UPDATE user_senses
                SET is_primary = FALSE
                WHERE user_word_id = $1 AND id <> $2
                "#,
            )
            .bind(sense.user_word_id)
            .bind(created.id().unwrap())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(created)
    }

    async fn update_user_sense(
        &self,
        user_id: i64,
        sense_id: i64,
        update: SenseUpdate,
    ) -> Result<UserSense, WordRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            SELECT us.id, us.user_word_id, us.text, us.is_primary, us.sort_order, us.note, us.created_at
            FROM user_senses us
            JOIN user_words uw ON uw.id = us.user_word_id
            WHERE us.id = $1 AND uw.user_id = $2
            "#,
        )
        .bind(sense_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        let mut sense = UserSense::from_parts(
            Some(row.try_get("id")?),
            row.try_get("text")?,
            row.try_get("is_primary")?,
            row.try_get("sort_order")?,
            row.try_get("note")?,
            row.try_get("created_at")?,
        )?;

        if let Some(text) = update.text {
            sense.set_text(text)?;
        }
        if let Some(sort_order) = update.sort_order {
            sense.set_sort_order(sort_order);
        }
        if let Some(note) = update.note {
            sense.set_note(note)?;
        }
        if let Some(is_primary) = update.is_primary {
            sense.set_primary(is_primary);
        }

        let updated = sqlx::query(
            r#"
            UPDATE user_senses
            SET text = $1, is_primary = $2, sort_order = $3, note = $4
            WHERE id = $5
            RETURNING id, text, is_primary, sort_order, note, created_at, user_word_id
            "#,
        )
        .bind(sense.text())
        .bind(sense.is_primary)
        .bind(sense.sort_order)
        .bind(sense.note())
        .bind(sense_id)
        .fetch_one(&mut *tx)
        .await?;

        let user_word_id: i64 = updated.try_get("user_word_id")?;
        if sense.is_primary {
            sqlx::query(
                r#"
                UPDATE user_senses
                SET is_primary = FALSE
                WHERE user_word_id = $1 AND id <> $2
                "#,
            )
            .bind(user_word_id)
            .bind(sense_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let result = UserSense::from_parts(
            Some(updated.try_get("id")?),
            updated.try_get("text")?,
            updated.try_get("is_primary")?,
            updated.try_get("sort_order")?,
            updated.try_get("note")?,
            updated.try_get("created_at")?,
        )?;

        Ok(result)
    }

    async fn remove_user_sense(
        &self,
        user_id: i64,
        sense_id: i64,
    ) -> Result<UserSense, WordRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            DELETE FROM user_senses
            USING user_words
            WHERE user_senses.id = $1
              AND user_senses.user_word_id = user_words.id
              AND user_words.user_id = $2
            RETURNING user_senses.id, user_senses.text, user_senses.is_primary, user_senses.sort_order, user_senses.note, user_senses.created_at
            "#,
        )
        .bind(sense_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        UserSense::from_parts(
            Some(row.try_get("id")?),
            row.try_get("text")?,
            row.try_get("is_primary")?,
            row.try_get("sort_order")?,
            row.try_get("note")?,
            row.try_get("created_at")?,
        )
        .map_err(WordRepositoryError::from)
    }

    async fn search(
        &self,
        params: SearchParams,
    ) -> Result<Vec<UserWordAggregate>, WordRepositoryError> {
        let base = format!("{}", Self::aggregate_query());
        let trimmed = params.query.trim();
        let sql;
        let rows = if trimmed.is_empty() {
            sql = format!(
                "{} GROUP BY uw.id, w.id ORDER BY w.canonical_key LIMIT $2 OFFSET $3",
                base
            );
            sqlx::query(&sql)
                .bind(params.user_id)
                .bind(params.limit)
                .bind(params.offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            let condition = match params.scope {
                SearchScope::Word => "w.canonical_key ILIKE $2".to_string(),
                SearchScope::Sense => {
                    "EXISTS (SELECT 1 FROM user_senses sub WHERE sub.user_word_id = uw.id AND sub.text ILIKE $2)".to_string()
                }
                SearchScope::Both => {
                    "(w.canonical_key ILIKE $2 OR EXISTS (SELECT 1 FROM user_senses sub WHERE sub.user_word_id = uw.id AND sub.text ILIKE $2))".to_string()
                }
            };

            sql = format!(
                "{} AND {} GROUP BY uw.id, w.id ORDER BY w.canonical_key LIMIT $3 OFFSET $4",
                base, condition
            );

            let pattern = match params.scope {
                SearchScope::Word => format!("%{}%", Self::canonical_like_pattern(trimmed)),
                _ => format!("%{}%", trimmed),
            };

            sqlx::query(&sql)
                .bind(params.user_id)
                .bind(pattern)
                .bind(params.limit)
                .bind(params.offset)
                .fetch_all(&self.pool)
                .await?
        };

        rows.into_iter().map(Self::build_aggregate).collect()
    }
}
