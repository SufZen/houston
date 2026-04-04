use thiserror::Error;

use crate::{DEFAULT_MEMORY_LIMIT, DEFAULT_USER_LIMIT};

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "{target} is at {percent}% capacity ({current}/{limit} chars). \
         This entry ({entry_size} chars) would exceed the limit. \
         Replace a stale entry with `replace_entry()` or `remove_entry()` to free space first."
    )]
    LimitExceeded {
        target: String,
        current: usize,
        limit: usize,
        entry_size: usize,
        percent: usize,
    },

    #[error("Entry index {index} out of range (file has {count} entries)")]
    IndexOutOfRange { index: usize, count: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryTarget {
    /// MEMORY.md — agent's personal notes
    Agent,
    /// USER.md — user profile
    User,
}

impl MemoryTarget {
    pub fn filename(&self) -> &str {
        match self {
            Self::Agent => "MEMORY.md",
            Self::User => "USER.md",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Agent => "MEMORY",
            Self::User => "USER PROFILE",
        }
    }
}

impl std::fmt::Display for MemoryTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub agent_entries: Vec<MemoryEntry>,
    pub agent_chars: usize,
    pub agent_limit: usize,
    pub user_entries: Vec<MemoryEntry>,
    pub user_chars: usize,
    pub user_limit: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub memory_limit: usize,
    pub user_limit: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            user_limit: DEFAULT_USER_LIMIT,
        }
    }
}
