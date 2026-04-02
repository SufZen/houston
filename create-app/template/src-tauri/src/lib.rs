mod commands;
mod workspace;

use keel_tauri::chat_session::ChatSessionState;
use keel_tauri::keel_db::Database;
use keel_tauri::keel_events::EventQueue;
use keel_tauri::keel_memory::MemoryStore;
use keel_tauri::keel_scheduler::Scheduler;
use keel_tauri::state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = keel_tauri::keel_db::db::default_data_dir("{{APP_NAME_SNAKE}}");
            let db_path = data_dir.join("{{APP_NAME_SNAKE}}.db");

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
            let chat_session = ChatSessionState::default();

            app.manage(AppState {
                db,
                event_queue: Some(queue_handle),
                scheduler: Some(Arc::new(Mutex::new(scheduler))),
            });
            app.manage(memory_store);
            app.manage(chat_session);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::delete_project,
            commands::issues::list_issues,
            commands::issues::create_issue,
            commands::sessions::start_session,
            commands::sessions::load_chat_feed,
            commands::workspace::list_workspace_files,
            commands::workspace::read_workspace_file,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
