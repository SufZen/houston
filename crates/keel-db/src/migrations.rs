use crate::db::Database;
use anyhow::Result;

impl Database {
    /// Run base migrations for the generic Keel tables.
    /// Application-specific migrations should be run separately by the consuming app.
    pub(crate) async fn run_migrations(&self) -> Result<()> {
        // chat_feed table with claude_session_id column.
        self.conn()
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS chat_feed (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                feed_key TEXT NOT NULL DEFAULT 'main',
                feed_type TEXT NOT NULL,
                data_json TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'desktop',
                claude_session_id TEXT,
                timestamp TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_chat_feed_project_key
                ON chat_feed(project_id, feed_key);
            CREATE INDEX IF NOT EXISTS idx_chat_feed_session
                ON chat_feed(claude_session_id);",
            )
            .await
            .ok();

        Ok(())
    }
}
