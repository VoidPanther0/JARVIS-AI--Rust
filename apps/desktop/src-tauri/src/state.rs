use jarvis_ai::settings::JarvisAiSettings;
use jarvis_ai::Orchestrator;
use jarvis_core::AppConfig;
use jarvis_memory::MemoryStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, OnceCell};

/// The key `AppConfig::database_path()`'s key/value table stores the
/// persisted (non-secret) AI settings blob under.
pub const SETTINGS_MEMORY_KEY: &str = "jarvis.ai.settings";

/// Shared application state handed to every Tauri command.
pub struct AppState {
    pub config: AppConfig,
    pub ai: Arc<Orchestrator>,
    memory: OnceCell<MemoryStore>,
    /// Cancellation senders for in-flight chat requests, keyed by the
    /// request id the frontend generated when it issued the request.
    pending_requests: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            ai: Arc::new(Orchestrator::new()),
            memory: OnceCell::new(),
            pending_requests: Mutex::new(HashMap::new()),
        }
    }

    /// Opens the SQLite store on first use and reuses it afterwards.
    pub async fn memory(&self) -> Result<&MemoryStore, jarvis_memory::MemoryError> {
        self.memory
            .get_or_try_init(|| async { MemoryStore::open(self.config.database_path()).await })
            .await
    }

    /// Loads persisted AI settings (or defaults on first run) and applies
    /// them to the orchestrator. Call at startup and after `set_settings`.
    pub async fn load_and_apply_ai_settings(&self) -> Result<JarvisAiSettings, String> {
        let memory = self.memory().await.map_err(|e| e.to_string())?;
        let settings = match memory
            .get_memory(SETTINGS_MEMORY_KEY)
            .await
            .map_err(|e| e.to_string())?
        {
            Some(json) => serde_json::from_str(&json).map_err(|e| {
                format!("stored AI settings are corrupt, using defaults instead: {e}")
            })?,
            None => JarvisAiSettings::default(),
        };
        self.ai.configure(&settings).await.map_err(|e| e.to_string())?;
        Ok(settings)
    }

    pub async fn persist_ai_settings(&self, settings: &JarvisAiSettings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|e| e.to_string())?;
        self.memory()
            .await
            .map_err(|e| e.to_string())?
            .set_memory(SETTINGS_MEMORY_KEY, &json)
            .await
            .map_err(|e| e.to_string())
    }

    /// Registers a cancellation sender for `request_id`, replacing any
    /// stale entry with the same id (there shouldn't be one in practice).
    pub async fn register_request(&self, request_id: String) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(request_id, tx);
        rx
    }

    /// Removes the bookkeeping entry for a finished request.
    pub async fn finish_request(&self, request_id: &str) {
        self.pending_requests.lock().await.remove(request_id);
    }

    /// Fires the cancellation signal for `request_id`, if it's still
    /// in flight. Returns `true` if a request was actually cancelled.
    pub async fn cancel_request(&self, request_id: &str) -> bool {
        if let Some(tx) = self.pending_requests.lock().await.remove(request_id) {
            tx.send(()).is_ok()
        } else {
            false
        }
    }
}
