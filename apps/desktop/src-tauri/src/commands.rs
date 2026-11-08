use crate::state::AppState;
use jarvis_ai::{ChatMessage, ChatRequest, Role};
use jarvis_memory::MessageRole;
use tauri::State;

/// Sends a user message to the configured AI provider and persists both
/// sides of the exchange. The provider name comes from configuration; no
/// provider is hardcoded.
#[tauri::command]
pub async fn send_message(state: State<'_, AppState>, message: String) -> Result<String, String> {
    let provider_name = state.config.ai_provider.as_deref().unwrap_or("null");
    let provider = state.providers.get(provider_name).map_err(|e| e.to_string())?;

    let response = provider
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: Role::User,
                content: message.clone(),
            }],
        })
        .await
        .map_err(|e| e.to_string())?;

    let memory = state.memory().await;
    let conversation_id = memory.start_conversation().await.map_err(|e| e.to_string())?;
    memory
        .add_message(conversation_id, MessageRole::User, &message)
        .await
        .map_err(|e| e.to_string())?;
    memory
        .add_message(conversation_id, MessageRole::Assistant, &response.message.content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.message.content)
}
