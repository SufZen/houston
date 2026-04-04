//! keel-memory — Character-bounded persistent memory for AI agents.
//!
//! Two markdown files in `.keel/memory/`:
//! - **MEMORY.md** — agent's personal notes (environment facts, tool quirks, lessons learned)
//! - **USER.md** — user profile (preferences, role, communication style)
//!
//! Entries are delimited by `§` (section sign). The character limit forces curation —
//! when full, the agent must replace stale entries to make room.

pub mod entries;
pub mod prompt;
mod types;

pub use types::*;

use std::path::Path;

use entries::{
    char_count, check_limit, check_limit_for_replace, parse_entries, read_file_or_empty,
    write_entries,
};

pub const ENTRY_DELIMITER: &str = "\u{00a7}";
pub const DEFAULT_MEMORY_LIMIT: usize = 2_200;
pub const DEFAULT_USER_LIMIT: usize = 1_375;

/// Load a frozen snapshot of both memory files for system prompt injection.
pub fn load_snapshot(
    memory_dir: &Path,
    config: &MemoryConfig,
) -> Result<MemorySnapshot, MemoryError> {
    let agent_content = read_file_or_empty(memory_dir, MemoryTarget::Agent)?;
    let user_content = read_file_or_empty(memory_dir, MemoryTarget::User)?;

    let agent_strings = parse_entries(&agent_content);
    let user_strings = parse_entries(&user_content);

    let agent_chars = char_count(&agent_strings);
    let user_chars = char_count(&user_strings);

    let agent_entries = agent_strings
        .into_iter()
        .enumerate()
        .map(|(i, text)| MemoryEntry { index: i, text })
        .collect();

    let user_entries = user_strings
        .into_iter()
        .enumerate()
        .map(|(i, text)| MemoryEntry { index: i, text })
        .collect();

    Ok(MemorySnapshot {
        agent_entries,
        agent_chars,
        agent_limit: config.memory_limit,
        user_entries,
        user_chars,
        user_limit: config.user_limit,
    })
}

/// List entries from a specific target file.
pub fn list_entries(
    memory_dir: &Path,
    target: MemoryTarget,
) -> Result<Vec<MemoryEntry>, MemoryError> {
    let content = read_file_or_empty(memory_dir, target)?;
    let strings = parse_entries(&content);
    Ok(strings
        .into_iter()
        .enumerate()
        .map(|(i, text)| MemoryEntry { index: i, text })
        .collect())
}

/// Add an entry to a target file. Returns error if it would exceed the limit.
pub fn add_entry(
    memory_dir: &Path,
    target: MemoryTarget,
    text: &str,
    config: &MemoryConfig,
) -> Result<(), MemoryError> {
    let content = read_file_or_empty(memory_dir, target)?;
    let mut entries = parse_entries(&content);
    check_limit(target, &entries, text, config)?;
    entries.push(text.trim().to_string());
    write_entries(memory_dir, target, &entries)
}

/// Replace an entry by index. Returns error if the new text would exceed the limit.
pub fn replace_entry(
    memory_dir: &Path,
    target: MemoryTarget,
    index: usize,
    new_text: &str,
    config: &MemoryConfig,
) -> Result<(), MemoryError> {
    let content = read_file_or_empty(memory_dir, target)?;
    let mut entries = parse_entries(&content);

    if index >= entries.len() {
        return Err(MemoryError::IndexOutOfRange {
            index,
            count: entries.len(),
        });
    }

    check_limit_for_replace(target, &entries, index, new_text, config)?;
    entries[index] = new_text.trim().to_string();
    write_entries(memory_dir, target, &entries)
}

/// Remove an entry by index.
pub fn remove_entry(
    memory_dir: &Path,
    target: MemoryTarget,
    index: usize,
) -> Result<(), MemoryError> {
    let content = read_file_or_empty(memory_dir, target)?;
    let mut entries = parse_entries(&content);

    if index >= entries.len() {
        return Err(MemoryError::IndexOutOfRange {
            index,
            count: entries.len(),
        });
    }

    entries.remove(index);
    write_entries(memory_dir, target, &entries)
}

/// Build formatted memory block for system prompt injection.
pub fn build_memory_prompt(
    memory_dir: &Path,
    config: &MemoryConfig,
) -> Result<String, MemoryError> {
    prompt::build_memory_prompt(memory_dir, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn memory_dir(base: &tempfile::TempDir) -> std::path::PathBuf {
        base.path().join(".keel").join("memory")
    }

    #[test]
    fn add_entry_creates_dir_and_file() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "First note", &config).unwrap();

        let content = std::fs::read_to_string(dir.join("MEMORY.md")).unwrap();
        assert_eq!(content, "First note");
    }

    #[test]
    fn add_multiple_entries() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Note 1", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "Note 2", &config).unwrap();

        let entries = list_entries(&dir, MemoryTarget::Agent).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Note 1");
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[1].text, "Note 2");
        assert_eq!(entries[1].index, 1);
    }

    #[test]
    fn add_entry_limit_exceeded() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig {
            memory_limit: 20,
            user_limit: 20,
        };

        add_entry(&dir, MemoryTarget::Agent, "Short", &config).unwrap();

        let result = add_entry(
            &dir,
            MemoryTarget::Agent,
            "This is a much longer entry that will exceed the limit",
            &config,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("MEMORY"));
        assert!(msg.contains("capacity"));
        assert!(msg.contains("replace_entry"));
    }

    #[test]
    fn replace_entry_success() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Old note", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "Keep this", &config).unwrap();

        replace_entry(&dir, MemoryTarget::Agent, 0, "New note", &config).unwrap();

        let entries = list_entries(&dir, MemoryTarget::Agent).unwrap();
        assert_eq!(entries[0].text, "New note");
        assert_eq!(entries[1].text, "Keep this");
    }

    #[test]
    fn replace_entry_index_out_of_range() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Only entry", &config).unwrap();

        let result = replace_entry(&dir, MemoryTarget::Agent, 5, "New text", &config);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("index 5"));
        assert!(msg.contains("1 entries"));
    }

    #[test]
    fn replace_entry_limit_exceeded() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig {
            memory_limit: 30,
            user_limit: 30,
        };

        add_entry(&dir, MemoryTarget::Agent, "Short", &config).unwrap();

        let result = replace_entry(
            &dir,
            MemoryTarget::Agent,
            0,
            "This replacement is way too long for the tiny limit we set",
            &config,
        );
        assert!(result.is_err());
    }

    #[test]
    fn remove_entry_success() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "First", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "Second", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "Third", &config).unwrap();

        remove_entry(&dir, MemoryTarget::Agent, 1).unwrap();

        let entries = list_entries(&dir, MemoryTarget::Agent).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "First");
        assert_eq!(entries[1].text, "Third");
    }

    #[test]
    fn remove_entry_index_out_of_range() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Only entry", &config).unwrap();

        let result = remove_entry(&dir, MemoryTarget::Agent, 3);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("index 3"));
    }

    #[test]
    fn list_entries_missing_file() {
        let base = setup_dir();
        let dir = memory_dir(&base);

        let entries = list_entries(&dir, MemoryTarget::Agent).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_entries_missing_dir() {
        let base = setup_dir();
        let dir = base.path().join("nonexistent").join("memory");

        let entries = list_entries(&dir, MemoryTarget::User).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn load_snapshot_missing_files() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        let snapshot = load_snapshot(&dir, &config).unwrap();
        assert!(snapshot.agent_entries.is_empty());
        assert!(snapshot.user_entries.is_empty());
        assert_eq!(snapshot.agent_chars, 0);
        assert_eq!(snapshot.user_chars, 0);
        assert_eq!(snapshot.agent_limit, DEFAULT_MEMORY_LIMIT);
        assert_eq!(snapshot.user_limit, DEFAULT_USER_LIMIT);
    }

    #[test]
    fn load_snapshot_with_data() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Agent note 1", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "Agent note 2", &config).unwrap();
        add_entry(&dir, MemoryTarget::User, "User pref 1", &config).unwrap();

        let snapshot = load_snapshot(&dir, &config).unwrap();
        assert_eq!(snapshot.agent_entries.len(), 2);
        assert_eq!(snapshot.user_entries.len(), 1);
        assert!(snapshot.agent_chars > 0);
        assert!(snapshot.user_chars > 0);
    }

    #[test]
    fn user_target_operations() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::User, "Name: James", &config).unwrap();
        add_entry(
            &dir,
            MemoryTarget::User,
            "Prefers action over discussion",
            &config,
        )
        .unwrap();

        let entries = list_entries(&dir, MemoryTarget::User).unwrap();
        assert_eq!(entries.len(), 2);

        let content = std::fs::read_to_string(dir.join("USER.md")).unwrap();
        assert!(content.contains("Name: James"));
        assert!(content.contains("Prefers action"));
    }

    #[test]
    fn memory_target_display() {
        assert_eq!(MemoryTarget::Agent.filename(), "MEMORY.md");
        assert_eq!(MemoryTarget::User.filename(), "USER.md");
        assert_eq!(MemoryTarget::Agent.label(), "MEMORY");
        assert_eq!(MemoryTarget::User.label(), "USER PROFILE");
    }

    #[test]
    fn build_memory_prompt_integration() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "Environment: macOS", &config).unwrap();
        add_entry(&dir, MemoryTarget::Agent, "VPN required", &config).unwrap();
        add_entry(&dir, MemoryTarget::User, "Name: James", &config).unwrap();

        let prompt = build_memory_prompt(&dir, &config).unwrap();
        assert!(prompt.contains("MEMORY (your personal notes)"));
        assert!(prompt.contains("USER PROFILE"));
        assert!(prompt.contains("Environment: macOS"));
        assert!(prompt.contains("VPN required"));
        assert!(prompt.contains("Name: James"));
    }

    #[test]
    fn build_memory_prompt_empty() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        let prompt = build_memory_prompt(&dir, &config).unwrap();
        assert_eq!(prompt, "");
    }

    #[test]
    fn add_entry_trims_whitespace() {
        let base = setup_dir();
        let dir = memory_dir(&base);
        let config = MemoryConfig::default();

        add_entry(&dir, MemoryTarget::Agent, "  Padded entry  ", &config).unwrap();

        let entries = list_entries(&dir, MemoryTarget::Agent).unwrap();
        assert_eq!(entries[0].text, "Padded entry");
    }
}
