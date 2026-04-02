//! Per-agent session state: maps project_id → ChatSessionState.
//!
//! Each agent (project) maintains its own Claude session ID for `--resume`,
//! so switching agents doesn't clobber the conversation context.

use keel_tauri::chat_session::ChatSessionState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Managed Tauri state: one `ChatSessionState` per agent (project_id).
#[derive(Default, Clone)]
pub struct AgentSessionMap {
    inner: Arc<RwLock<HashMap<String, ChatSessionState>>>,
}

impl AgentSessionMap {
    /// Get (or lazily create) the `ChatSessionState` for a given agent.
    pub async fn get_for_agent(&self, project_id: &str) -> ChatSessionState {
        // Fast path: read lock.
        {
            let map = self.inner.read().await;
            if let Some(state) = map.get(project_id) {
                return state.clone();
            }
        }
        // Slow path: write lock, insert default.
        let mut map = self.inner.write().await;
        map.entry(project_id.to_string())
            .or_insert_with(ChatSessionState::default)
            .clone()
    }

    /// Remove session state for a deleted agent.
    pub async fn remove_agent(&self, project_id: &str) {
        let mut map = self.inner.write().await;
        map.remove(project_id);
    }
}
