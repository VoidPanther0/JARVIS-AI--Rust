use jarvis_ai::ProviderRegistry;
use jarvis_core::AppConfig;
use jarvis_memory::MemoryStore;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Shared application state handed to every Tauri command.
pub struct AppState {
    pub config: AppConfig,
    pub providers: Arc<ProviderRegistry>,
    memory: OnceCell<MemoryStore>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            providers: Arc::new(ProviderRegistry::new()),
            memory: OnceCell::new(),
        }
    }

    /// Opens the SQLite store on first use and reuses it afterwards.
    pub async fn memory(&self) -> Result<&MemoryStore, jarvis_memory::MemoryError> {
        self.memory
            .get_or_try_init(|| async { MemoryStore::open(self.config.database_path()).await })
            .await
    }
}
