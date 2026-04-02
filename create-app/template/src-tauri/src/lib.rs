mod agent_sessions;
mod channel_routing;
mod commands;
mod workspace;

pub use agent_sessions::AgentSessionMap;

use keel_tauri::channel_manager::ChannelManager;
use keel_tauri::events::KeelEvent;
use keel_tauri::keel_db::Database;
use keel_tauri::keel_events::EventQueue;
use keel_tauri::keel_memory::MemoryStore;
use keel_tauri::keel_scheduler::Scheduler;
use keel_tauri::state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = keel_tauri::keel_db::db::default_data_dir("{{APP_NAME}}");
            let db_path = data_dir.join("{{APP_NAME}}.db");

            let db = tauri::async_runtime::block_on(async {
                Database::connect(&db_path)
                    .await
                    .expect("Failed to open database")
            });

            let memory_dir = data_dir.join("memories");
            let memory_store = tauri::async_runtime::block_on(async {
                let mem_db = libsql::Builder::new_local(&db_path)
                    .build()
                    .await
                    .expect("Failed to open memory DB connection");
                let mem_conn = Arc::new(
                    mem_db.connect().expect("Failed to connect for memory store"),
                );
                MemoryStore::new_with_markdown_dir(mem_conn, memory_dir)
                    .await
                    .expect("Failed to initialize memory store")
            });

            let (_event_queue, queue_handle) = EventQueue::new();
            let scheduler = Scheduler::new(queue_handle.clone());

            app.manage(AppState {
                db,
                event_queue: Some(queue_handle),
                scheduler: Some(Arc::new(Mutex::new(scheduler))),
            });
            app.manage(memory_store);
            app.manage(AgentSessionMap::default());

            // Channel manager: start adapters and route incoming messages.
            let (mgr, mut channel_rx) = ChannelManager::new();

            // Auto-reconnect channels that were connected before restart.
            tauri::async_runtime::block_on(async {
                let db_ref = &app.state::<AppState>().db;
                if let Ok(mut rows) = db_ref.conn().query(
                    "SELECT id, channel_type, config FROM channels WHERE status = 'connected'",
                    libsql::params![],
                ).await {
                    while let Ok(Some(row)) = rows.next().await {
                        let id: String = row.get(0).unwrap_or_default();
                        let ch_type: String = row.get(1).unwrap_or_default();
                        let config_str: String = row.get(2).unwrap_or_default();
                        if let Ok(config_val) = serde_json::from_str::<serde_json::Value>(&config_str) {
                            let token = config_val.get("token").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let config = keel_tauri::keel_channels::ChannelConfig {
                                channel_type: ch_type.clone(),
                                token,
                                extra: config_val,
                            };
                            if let Err(e) = mgr.start_channel(id.clone(), config).await {
                                eprintln!("[channels] auto-reconnect failed for {id} ({ch_type}): {e}");
                            } else {
                                eprintln!("[channels] auto-reconnected {id} ({ch_type})");
                            }
                        }
                    }
                }
            });

            app.manage(mgr);

            // System tray: keep app alive in background when window is closed.
            keel_tauri::tray::setup_tray(app, "{{APP_NAME_TITLE}}")?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some((registry_id, msg)) = channel_rx.recv().await {
                    let _ = handle.emit(
                        "keel-event",
                        KeelEvent::ChannelMessageReceived {
                            channel_type: msg.source.clone(),
                            channel_id: msg.channel_id.clone(),
                            sender_name: msg.sender_name.clone(),
                            text: msg.text.clone(),
                        },
                    );

                    channel_routing::route_channel_message(&handle, registry_id, msg).await;
                }
            });

            // DEBUG: test event after 2s
            let test_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let result = test_handle.emit("keel-event", serde_json::json!({
                    "type": "Toast",
                    "data": { "message": "{{APP_NAME_TITLE}} connected!", "variant": "info" }
                }));
                eprintln!("[debug] test emit result: {:?}", result);
            });

            Ok(())
        })
        .on_window_event(keel_tauri::tray::hide_on_close)
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::issues::list_issues,
            commands::issues::create_issue,
            commands::sessions::ensure_workspace,
            commands::sessions::start_session,
            commands::sessions::load_chat_feed,
            commands::memory::list_memories,
            commands::memory::create_memory,
            commands::memory::delete_memory,
            commands::memory::search_memories,
            commands::events::list_events,
            commands::scheduler::add_heartbeat,
            commands::scheduler::remove_heartbeat,
            commands::scheduler::add_cron,
            commands::scheduler::remove_cron,
            commands::channels::list_channels,
            commands::channels::add_channel,
            commands::channels::remove_channel,
            commands::channels::connect_channel,
            commands::channels::disconnect_channel,
            commands::workspace::list_workspace_files,
            commands::workspace::read_workspace_file,
            keel_tauri::workspace_commands::list_project_files,
            keel_tauri::workspace_commands::open_file,
            keel_tauri::workspace_commands::reveal_file,
            keel_tauri::workspace_commands::delete_file,
            keel_tauri::workspace_commands::import_files,
            keel_tauri::workspace_commands::create_workspace_folder,
            keel_tauri::workspace_commands::reveal_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
