use crate::AgentSessionMap;
use keel_tauri::keel_db::Project;
use keel_tauri::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    state.db.list_projects().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    folder_path: String,
) -> Result<Project, String> {
    state
        .db
        .create_project(&name, &folder_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    id: String,
    name: String,
    folder_path: String,
) -> Result<bool, String> {
    state
        .db
        .update_project(&id, &name, &folder_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    agent_sessions: State<'_, AgentSessionMap>,
    id: String,
) -> Result<bool, String> {
    let deleted = state
        .db
        .delete_project(&id)
        .await
        .map_err(|e| e.to_string())?;
    if deleted {
        agent_sessions.remove_agent(&id).await;
    }
    Ok(deleted)
}
