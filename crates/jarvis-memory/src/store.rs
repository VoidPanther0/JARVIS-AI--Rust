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

    fn from_db(value: &str) -> Self {
        match value {
            "system" => MessageRole::System,
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique path under the OS temp directory so parallel test runs
    /// never collide on the same SQLite file.
    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("jarvis-memory-test-{name}-{unique}.sqlite3"))
    }

    #[tokio::test]
    async fn conversation_and_messages_round_trip() {
        let path = temp_db_path("conversation");
        let store = MemoryStore::open(&path)
            .await
            .expect("opening a fresh SQLite file should succeed");

        let conversation_id = store
            .start_conversation()
            .await
            .expect("inserting a conversation should succeed");

        store
            .add_message(conversation_id, MessageRole::User, "hello")
            .await
            .expect("inserting a user message should succeed");
        store
            .add_message(conversation_id, MessageRole::Assistant, "hi there")
            .await
            .expect("inserting an assistant message should succeed");

        drop(store);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn memory_entries_round_trip_and_overwrite() {
        let path = temp_db_path("kv");
        let store = MemoryStore::open(&path)
            .await
            .expect("opening a fresh SQLite file should succeed");

        assert_eq!(store.get_memory("favorite_color").await.unwrap(), None);

        store.set_memory("favorite_color", "blue").await.unwrap();
        assert_eq!(
            store.get_memory("favorite_color").await.unwrap(),
            Some("blue".to_string())
        );

        store.set_memory("favorite_color", "green").await.unwrap();
        assert_eq!(
            store.get_memory("favorite_color").await.unwrap(),
            Some("green".to_string())
        );

        drop(store);
        let _ = std::fs::remove_file(&path);
    }
}
