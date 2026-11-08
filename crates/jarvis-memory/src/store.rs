use crate::error::MemoryResult;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: MessageRole,
    pub content: String,
}

/// Owns the SQLite pool and applies embedded migrations on construction.
#[derive(Clone)]
pub struct MemoryStore {
    pool: SqlitePool,
}

impl MemoryStore {
    /// Opens (creating if necessary) the database at `path` and runs any
    /// pending migrations.
    pub async fn open(path: impl AsRef<Path>) -> MemoryResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", path.as_ref().display()))?
                .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("src/migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn start_conversation(&self) -> MemoryResult<i64> {
        let result = sqlx::query("INSERT INTO conversations DEFAULT VALUES")
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn add_message(
        &self,
        conversation_id: i64,
        role: MessageRole,
        content: &str,
    ) -> MemoryResult<i64> {
        let result =
            sqlx::query("INSERT INTO messages (conversation_id, role, content) VALUES (?, ?, ?)")
                .bind(conversation_id)
                .bind(role.as_str())
                .bind(content)
                .execute(&self.pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn set_memory(&self, key: &str, value: &str) -> MemoryResult<()> {
        sqlx::query(
            "INSERT INTO memory_entries (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_memory(&self, key: &str) -> MemoryResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM memory_entries WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(value,)| value))
    }
}
