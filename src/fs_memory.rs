use crate::config::{atomic_write, Config};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsMemory<'a> {
    config: &'a Config,
}

impl<'a> FsMemory<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Resolve an address like `/memory/projects/firebac/` or `projects/firebac`
    /// to a real filesystem path under the memory directory.
    pub fn resolve(&self, address: &str) -> Result<PathBuf> {
        let trimmed = address.trim_start_matches('/').trim_end_matches('/');
        let relative = if trimmed == "memory" {
            ""
        } else if let Some(stripped) = trimmed.strip_prefix("memory/") {
            stripped
        } else {
            trimmed
        };
        Ok(self.config.memory_dir.join(relative))
    }

    pub fn read(&self, address: &str) -> Result<String> {
        let path = self.resolve(address)?;
        if path.is_dir() {
            let mut entries: Vec<String> = Vec::new();
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
            entries.sort();
            Ok(entries.join("\n"))
        } else {
            fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))
        }
    }

    pub fn write(&self, address: &str, content: &str) -> Result<PathBuf> {
        let path = self.resolve(address)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&path, content)
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    pub fn list(&self, address: &str) -> Result<Vec<String>> {
        let path = self.resolve(address)?;
        let mut out = Vec::new();
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                out.push(entry.file_name().to_string_lossy().to_string());
            }
            out.sort();
        }
        Ok(out)
    }

    pub fn exists(&self, address: &str) -> Result<bool> {
        Ok(self.resolve(address)?.exists())
    }
}

pub fn address_to_kg_path(address: &str) -> String {
    let trimmed = address.trim_start_matches('/').trim_end_matches('/');
    let without_memory = trimmed.strip_prefix("memory/").unwrap_or(trimmed);
    without_memory.replace('/', ".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;

    fn test_config(base: &std::path::Path) -> Config {
        Config {
            base_dir: base.to_path_buf(),
            memory_dir: base.join("memory"),
            kg_dir: base.join("memory/kg"),
            conversations_dir: base.join("memory/conversations"),
            characters_dir: base.join("memory/characters"),
            groq_api_key: None,
            groq_model: "test".into(),
            max_context_messages: 5,
            max_memory_messages: 10,
            compression_threshold: 3,
            score_weights: crate::config::ScoreWeights::default(),
            memory_threshold: 0.25,
        }
    }

    #[test]
    fn test_resolve_memory_root() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        let fs = FsMemory::new(&config);

        assert_eq!(fs.resolve("/memory/").unwrap(), config.memory_dir);
        assert_eq!(fs.resolve("/memory").unwrap(), config.memory_dir);
        assert_eq!(
            fs.resolve("/memory/projects/firebac").unwrap(),
            config.memory_dir.join("projects/firebac")
        );
        assert_eq!(
            fs.resolve("projects/firebac").unwrap(),
            config.memory_dir.join("projects/firebac")
        );
    }

    #[test]
    fn test_write_and_read() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        let fs = FsMemory::new(&config);

        fs.write("/memory/projects/firebac/warfare.yaml", "mode: tactics").unwrap();
        let content = fs.read("/memory/projects/firebac/warfare.yaml").unwrap();
        assert_eq!(content, "mode: tactics");
    }

    #[test]
    fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        let fs = FsMemory::new(&config);

        fs.write("/memory/projects/firebac/a.yaml", "a").unwrap();
        fs.write("/memory/projects/firebac/b.yaml", "b").unwrap();

        let entries = fs.list("/memory/projects/firebac/").unwrap();
        assert!(entries.contains(&"a.yaml".into()));
        assert!(entries.contains(&"b.yaml".into()));
    }

    #[test]
    fn test_address_to_kg_path() {
        assert_eq!(address_to_kg_path("/memory/projects/firebac"), "projects.firebac");
        assert_eq!(address_to_kg_path("memory/projects/firebac"), "projects.firebac");
    }
}
