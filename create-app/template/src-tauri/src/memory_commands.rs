use keel_memory::{MemoryConfig, MemoryTarget};
use keel_tauri::paths::expand_tilde;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct MemoryEntryResponse {
    pub index: usize,
    pub text: String,
}

#[derive(Serialize)]
pub struct MemorySnapshotResponse {
    pub agent_entries: Vec<MemoryEntryResponse>,
    pub agent_chars: usize,
    pub agent_limit: usize,
    pub user_entries: Vec<MemoryEntryResponse>,
    pub user_chars: usize,
    pub user_limit: usize,
}

fn memory_dir(workspace_path: &str) -> PathBuf {
    expand_tilde(&PathBuf::from(workspace_path)).join(".keel/memory")
}

fn parse_target(target: &str) -> Result<MemoryTarget, String> {
    match target {
        "agent" => Ok(MemoryTarget::Agent),
        "user" => Ok(MemoryTarget::User),
        _ => Err(format!(
            "Invalid memory target '{target}'. Use 'agent' or 'user'."
        )),
    }
}

#[tauri::command]
pub async fn load_memory(
    workspace_path: String,
) -> Result<MemorySnapshotResponse, String> {
    let dir = memory_dir(&workspace_path);
    let config = MemoryConfig::default();
    let snapshot =
        keel_memory::load_snapshot(&dir, &config).map_err(|e| e.to_string())?;

    Ok(MemorySnapshotResponse {
        agent_entries: snapshot
            .agent_entries
            .into_iter()
            .map(|e| MemoryEntryResponse {
                index: e.index,
                text: e.text,
            })
            .collect(),
        agent_chars: snapshot.agent_chars,
        agent_limit: snapshot.agent_limit,
        user_entries: snapshot
            .user_entries
            .into_iter()
            .map(|e| MemoryEntryResponse {
                index: e.index,
                text: e.text,
            })
            .collect(),
        user_chars: snapshot.user_chars,
        user_limit: snapshot.user_limit,
    })
}

#[tauri::command]
pub async fn add_memory_entry(
    workspace_path: String,
    target: String,
    text: String,
) -> Result<(), String> {
    let dir = memory_dir(&workspace_path);
    let target = parse_target(&target)?;
    let config = MemoryConfig::default();
    keel_memory::add_entry(&dir, target, &text, &config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn replace_memory_entry(
    workspace_path: String,
    target: String,
    index: usize,
    text: String,
) -> Result<(), String> {
    let dir = memory_dir(&workspace_path);
    let target = parse_target(&target)?;
    let config = MemoryConfig::default();
    keel_memory::replace_entry(&dir, target, index, &text, &config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_memory_entry(
    workspace_path: String,
    target: String,
    index: usize,
) -> Result<(), String> {
    let dir = memory_dir(&workspace_path);
    let target = parse_target(&target)?;
    keel_memory::remove_entry(&dir, target, index)
        .map_err(|e| e.to_string())
}
