use std::path::Path;

use crate::entries::{char_count, limit_for, parse_entries, read_file_or_empty};
use crate::{MemoryConfig, MemoryError, MemoryTarget};

const SEPARATOR: &str =
    "\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\
     \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\
     \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\
     \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}";

/// Format a number with comma separators (e.g. 1375 -> "1,375").
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

/// Build a single memory section block.
fn build_section(
    target: MemoryTarget,
    entries: &[String],
    limit: usize,
) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    let chars = char_count(entries);
    let percent = if limit > 0 {
        (chars * 100) / limit
    } else {
        0
    };

    let label = match target {
        MemoryTarget::Agent => {
            format!(
                "{} (your personal notes) [{}% \u{2014} {}/{} chars]",
                target.label(),
                percent,
                format_number(chars),
                format_number(limit),
            )
        }
        MemoryTarget::User => {
            format!(
                "{} [{}% \u{2014} {}/{} chars]",
                target.label(),
                percent,
                format_number(chars),
                format_number(limit),
            )
        }
    };

    let content = entries.join(&format!("\n{}\n", crate::ENTRY_DELIMITER));

    Some(format!("{}\n{}\n{}\n{}", SEPARATOR, label, SEPARATOR, content))
}

/// Build formatted memory block for system prompt injection.
///
/// Returns empty string if both files are empty/don't exist.
pub fn build_memory_prompt(
    memory_dir: &Path,
    config: &MemoryConfig,
) -> Result<String, MemoryError> {
    let agent_content = read_file_or_empty(memory_dir, MemoryTarget::Agent)?;
    let user_content = read_file_or_empty(memory_dir, MemoryTarget::User)?;

    let agent_entries = parse_entries(&agent_content);
    let user_entries = parse_entries(&user_content);

    let agent_section = build_section(
        MemoryTarget::Agent,
        &agent_entries,
        limit_for(MemoryTarget::Agent, config),
    );
    let user_section = build_section(
        MemoryTarget::User,
        &user_entries,
        limit_for(MemoryTarget::User, config),
    );

    let sections: Vec<String> = [agent_section, user_section]
        .into_iter()
        .flatten()
        .collect();

    if sections.is_empty() {
        return Ok(String::new());
    }

    Ok(sections.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn format_number_small() {
        assert_eq!(format_number(42), "42");
    }

    #[test]
    fn format_number_with_commas() {
        assert_eq!(format_number(1375), "1,375");
        assert_eq!(format_number(2200), "2,200");
        assert_eq!(format_number(12345), "12,345");
    }

    #[test]
    fn build_section_empty_returns_none() {
        let entries: Vec<String> = vec![];
        assert!(build_section(MemoryTarget::Agent, &entries, 2200).is_none());
    }

    #[test]
    fn build_section_agent_format() {
        let entries = vec!["Entry one".to_string(), "Entry two".to_string()];
        let result = build_section(MemoryTarget::Agent, &entries, 2200).unwrap();
        assert!(result.contains("MEMORY (your personal notes)"));
        assert!(result.contains("chars]"));
        assert!(result.contains("Entry one"));
        assert!(result.contains("Entry two"));
    }

    #[test]
    fn build_section_user_format() {
        let entries = vec!["Name: James".to_string()];
        let result = build_section(MemoryTarget::User, &entries, 1375).unwrap();
        assert!(result.contains("USER PROFILE"));
        assert!(result.contains("Name: James"));
    }

    #[test]
    fn build_memory_prompt_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join("memory");
        fs::create_dir_all(&memory_dir).unwrap();

        let config = MemoryConfig::default();
        let result = build_memory_prompt(&memory_dir, &config).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn build_memory_prompt_missing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join("nonexistent");

        let config = MemoryConfig::default();
        let result = build_memory_prompt(&memory_dir, &config).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn build_memory_prompt_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join("memory");
        fs::create_dir_all(&memory_dir).unwrap();

        fs::write(
            memory_dir.join("MEMORY.md"),
            "Environment: macOS\n\u{00a7}\nVPN required",
        )
        .unwrap();
        fs::write(memory_dir.join("USER.md"), "Name: James").unwrap();

        let config = MemoryConfig::default();
        let result = build_memory_prompt(&memory_dir, &config).unwrap();

        assert!(result.contains("MEMORY (your personal notes)"));
        assert!(result.contains("USER PROFILE"));
        assert!(result.contains("Environment: macOS"));
        assert!(result.contains("VPN required"));
        assert!(result.contains("Name: James"));
    }

    #[test]
    fn build_memory_prompt_only_agent() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join("memory");
        fs::create_dir_all(&memory_dir).unwrap();

        fs::write(memory_dir.join("MEMORY.md"), "Some note").unwrap();

        let config = MemoryConfig::default();
        let result = build_memory_prompt(&memory_dir, &config).unwrap();

        assert!(result.contains("MEMORY"));
        assert!(!result.contains("USER PROFILE"));
    }
}
