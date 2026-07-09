use anyhow::{bail, Result};
use std::path::{Component, Path};

const MAX_BRANCH_LEN: usize = 128;
const MAX_PATH_LEN: usize = 256;

/// Validate a branch name for filesystem and git safety.
pub fn validate_branch(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("branch name cannot be empty");
    }
    if name.len() > MAX_BRANCH_LEN {
        bail!("branch name too long (max {} characters)", MAX_BRANCH_LEN);
    }
    if name.starts_with('-') {
        bail!("branch name cannot start with '-'");
    }
    if name == "." || name == ".." {
        bail!("branch name cannot be '.' or '..'");
    }
    if name.contains("..") {
        bail!("branch name cannot contain '..'");
    }
    if name.contains(|c: char| c.is_control()) {
        bail!("branch name cannot contain control characters");
    }
    Ok(())
}

/// Validate a knowledge-graph dot-path.
pub fn validate_kg_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("knowledge graph path cannot be empty");
    }
    if path.len() > MAX_PATH_LEN {
        bail!("knowledge graph path too long (max {} characters)", MAX_PATH_LEN);
    }
    if path.starts_with('.') || path.ends_with('.') {
        bail!("knowledge graph path cannot start or end with '.'");
    }
    if path.contains("..") {
        bail!("knowledge graph path cannot contain '..'");
    }
    for segment in path.split('.') {
        if segment.is_empty() {
            bail!("knowledge graph path cannot contain empty segments");
        }
        if segment.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            bail!("knowledge graph segment '{}' contains invalid characters", segment);
        }
    }
    Ok(())
}

/// Validate a filesystem memory address and prevent path traversal.
pub fn validate_address(address: &str) -> Result<()> {
    if address.is_empty() {
        bail!("memory address cannot be empty");
    }
    if address.len() > MAX_PATH_LEN {
        bail!("memory address too long (max {} characters)", MAX_PATH_LEN);
    }

    let trimmed = address.trim_start_matches('/').trim_end_matches('/');
    let relative = if trimmed == "memory" {
        ""
    } else if let Some(stripped) = trimmed.strip_prefix("memory/") {
        stripped
    } else {
        trimmed
    };

    let path = Path::new(relative);
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::RootDir => {}
            _ => bail!("memory address contains invalid path component: {:?}", component),
        }
    }
    Ok(())
}

/// Sanitize a string to be safe inside a git commit message.
pub fn sanitize_commit_message(message: &str) -> String {
    message
        .chars()
        .map(|c| if c.is_control() && c != '\n' { ' ' } else { c })
        .collect::<String>()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch() {
        assert!(validate_branch("main").is_ok());
        assert!(validate_branch("feature/new-ui").is_ok());
        assert!(validate_branch("").is_err());
        assert!(validate_branch("-foo").is_err());
        assert!(validate_branch("..").is_err());
        assert!(validate_branch("foo/../bar").is_err());
    }

    #[test]
    fn test_validate_kg_path() {
        assert!(validate_kg_path("user.name").is_ok());
        assert!(validate_kg_path("projects.0").is_ok());
        assert!(validate_kg_path("").is_err());
        assert!(validate_kg_path(".user").is_err());
        assert!(validate_kg_path("user..name").is_err());
        assert!(validate_kg_path("user@name").is_err());
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address("/memory/projects/firebac").is_ok());
        assert!(validate_address("projects/firebac/../marina").is_err());
        assert!(validate_address("memory/../../../etc/passwd").is_err());
    }

    #[test]
    fn test_sanitize_commit_message() {
        let raw = "hello\tworld\nsecond line\n";
        assert_eq!(sanitize_commit_message(raw), "hello world\nsecond line");
    }
}
