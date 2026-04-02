use keel_tauri::channel_manager::ChannelManager;
use keel_tauri::keel_channels::ChannelConfig;
use keel_tauri::state::AppState;
use tauri::State;

/// List all configured channels from the database.
#[tauri::command]
pub async fn list_channels(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = state
        .db
        .conn()
        .query(
            "SELECT id, channel_type, name, status, config, created_at \
             FROM channels ORDER BY created_at DESC",
            libsql::params![],
        )
        .await;

    match rows {
        Ok(mut rows) => {
            let mut results = Vec::new();
            while let Ok(Some(row)) = rows.next().await {
                let entry = serde_json::json!({
                    "id": row.get::<String>(0).unwrap_or_default(),
                    "channel_type": row.get::<String>(1).unwrap_or_default(),
                    "name": row.get::<String>(2).unwrap_or_default(),
                    "status": row.get::<String>(3).unwrap_or_default(),
                    "config": row.get::<String>(4).unwrap_or_default(),
                    "created_at": row.get::<String>(5).unwrap_or_default(),
                });
                results.push(entry);
            }
            Ok(results)
        }
        Err(_) => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn add_channel(
    state: State<'_, AppState>,
    channel_type: String,
    name: String,
    config: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let config_str = serde_json::to_string(&config).map_err(|e| e.to_string())?;

    state
        .db
        .conn()
        .execute(
            "INSERT INTO channels (id, channel_type, name, status, config, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'disconnected', ?4, ?5, ?5)",
            libsql::params![id.clone(), channel_type.clone(), name.clone(), config_str.clone(), now.clone()],
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "id": id, "channel_type": channel_type, "name": name,
        "status": "disconnected", "config": config_str, "created_at": now,
    }))
}

#[tauri::command]
pub async fn remove_channel(
    state: State<'_, AppState>,
    mgr: State<'_, ChannelManager>,
    channel_id: String,
) -> Result<(), String> {
    mgr.stop_channel(&channel_id).await.ok();
    state
        .db
        .conn()
        .execute("DELETE FROM channels WHERE id = ?1", libsql::params![channel_id])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Actually connect a channel adapter (Telegram polling, Slack Socket Mode).
#[tauri::command]
pub async fn connect_channel(
    state: State<'_, AppState>,
    mgr: State<'_, ChannelManager>,
    channel_id: String,
) -> Result<(), String> {
    // Read config from DB.
    let mut rows = state
        .db
        .conn()
        .query(
            "SELECT channel_type, config FROM channels WHERE id = ?1",
            libsql::params![channel_id.clone()],
        )
        .await
        .map_err(|e| e.to_string())?;

    let row = rows.next().await.map_err(|e| e.to_string())?
        .ok_or("Channel not found")?;

    let channel_type: String = row.get(0).map_err(|e| e.to_string())?;
    let config_str: String = row.get(1).map_err(|e| e.to_string())?;
    let config_val: serde_json::Value =
        serde_json::from_str(&config_str).map_err(|e| e.to_string())?;

    let token = config_val.get("token")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'token' in channel config")?
        .to_string();

    let chan_config = ChannelConfig {
        channel_type: channel_type.clone(),
        token,
        extra: config_val.clone(),
    };

    mgr.start_channel(channel_id.clone(), chan_config).await?;

    // Update DB status.
    state
        .db
        .conn()
        .execute(
            "UPDATE channels SET status = 'connected' WHERE id = ?1",
            libsql::params![channel_id],
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn disconnect_channel(
    state: State<'_, AppState>,
    mgr: State<'_, ChannelManager>,
    channel_id: String,
) -> Result<(), String> {
    mgr.stop_channel(&channel_id).await?;
    state
        .db
        .conn()
        .execute(
            "UPDATE channels SET status = 'disconnected' WHERE id = ?1",
            libsql::params![channel_id],
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
