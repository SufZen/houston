use keel_tauri::workspace as kw;
use std::path::Path;

pub fn seed_workspace(dir: &Path) -> Result<(), String> {
    kw::seed_file(dir, "CLAUDE.md", CLAUDE_MD)?;
    Ok(())
}

pub fn build_system_prompt(dir: &Path) -> String {
    kw::build_system_prompt(dir, BASE_SYSTEM_PROMPT, None, &PROMPT_FILES)
}

pub const KNOWN_FILES: &[(&str, &str)] = &[
    ("CLAUDE.md", "Agent instructions and behavior rules"),
];

const PROMPT_FILES: [(&str, &str); 1] = [
    ("CLAUDE.md", "CLAUDE.md — Agent Instructions"),
];

const BASE_SYSTEM_PROMPT: &str = "\
You are an AI assistant running inside {{APP_NAME_TITLE}}, \
a native desktop app. Your workspace files are injected below. Follow them.";

const CLAUDE_MD: &str = r#"# {{APP_NAME_TITLE}} Agent

## Role
You are a helpful AI assistant.

## Rules
- Be concise and direct
- Ask before making destructive changes
- Explain your reasoning when making decisions
"#;
