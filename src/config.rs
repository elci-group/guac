use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Config {
    pub base_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub kg_dir: PathBuf,
    pub conversations_dir: PathBuf,
    pub characters_dir: PathBuf,
    pub groq_api_key: Option<String>,
    pub groq_model: String,
    pub max_context_messages: usize,
    pub max_memory_messages: usize,
    pub compression_threshold: usize,
    pub score_weights: ScoreWeights,
    pub memory_threshold: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ScoreWeights {
    pub importance: f64,
    pub novelty: f64,
    pub recency: f64,
    pub repetition: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            importance: 0.35,
            novelty: 0.25,
            recency: 0.25,
            repetition: 0.15,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ConfigFile {
    groq_api_key: Option<String>,
    groq_model: Option<String>,
    max_context_messages: Option<usize>,
    max_memory_messages: Option<usize>,
    compression_threshold: Option<usize>,
    memory_threshold: Option<f64>,
    score_weights: Option<ScoreWeights>,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let base_dir = env::var("GUAC_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().expect("current dir"));

        let file_cfg = Self::load_config_file(&base_dir).unwrap_or_else(|e| {
            warn!(error = %e, "failed to load config file; using defaults");
            ConfigFile::default()
        });

        let memory_dir = base_dir.join("memory");
        let kg_dir = memory_dir.join("kg");
        let conversations_dir = memory_dir.join("conversations");
        let characters_dir = memory_dir.join("characters");

        let cfg = Self {
            memory_dir: memory_dir.clone(),
            kg_dir,
            conversations_dir,
            characters_dir,
            groq_api_key: Self::env_or_file("GROQ_API_KEY", file_cfg.groq_api_key)
                .filter(|s| !s.trim().is_empty()),
            groq_model: Self::env_or_file("GROQ_MODEL", file_cfg.groq_model)
                .unwrap_or_else(|| "llama-3.1-8b-instant".into()),
            max_context_messages: Self::env_or_file_parse("GUAC_MAX_CONTEXT", file_cfg.max_context_messages)
                .unwrap_or(10),
            max_memory_messages: Self::env_or_file_parse("GUAC_MAX_MEMORY", file_cfg.max_memory_messages)
                .unwrap_or(50),
            compression_threshold: Self::env_or_file_parse("GUAC_COMPRESS_THRESHOLD", file_cfg.compression_threshold)
                .unwrap_or(20),
            score_weights: file_cfg.score_weights.unwrap_or_default(),
            memory_threshold: Self::env_or_file_parse("GUAC_MEMORY_THRESHOLD", file_cfg.memory_threshold)
                .unwrap_or(0.25),
            base_dir,
        };

        info!(base_dir = %cfg.base_dir.display(), "loaded GUAC configuration");
        Ok(cfg)
    }

    fn load_config_file(base_dir: &Path) -> Result<ConfigFile> {
        let mut candidates = vec![
            base_dir.join("guac.toml"),
            base_dir.join(".guac.toml"),
        ];
        if let Some(config_dir) = dirs::config_dir() {
            candidates.push(config_dir.join("guac").join("guac.toml"));
        }

        let path = candidates
            .into_iter()
            .find(|p| p.exists())
            .context("no config file found")?;

        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cfg: ConfigFile = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        Ok(cfg)
    }

    fn env_or_file(key: &str, file_value: Option<String>) -> Option<String> {
        env::var(key).ok().or(file_value)
    }

    fn env_or_file_parse<T: std::str::FromStr>(key: &str, file_value: Option<T>) -> Option<T> {
        env::var(key)
            .ok()
            .and_then(|s| s.parse().ok())
            .or(file_value)
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            &self.memory_dir,
            &self.kg_dir,
            &self.conversations_dir,
            &self.characters_dir,
        ] {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("creating directory {}", dir.display()))?;
        }
        Ok(())
    }

    pub fn conversation_path(&self, branch: &str) -> PathBuf {
        self.conversations_dir
            .join(sanitize_branch(branch))
            .join("chat.yaml")
    }

    pub fn character_core_path(&self, name: &str) -> PathBuf {
        self.characters_dir.join(name).join("core.yaml")
    }

    pub fn character_memory_path(&self, name: &str) -> PathBuf {
        self.characters_dir.join(name).join("memory.yaml")
    }

    pub fn summary_path(&self, branch: &str, level: u32) -> PathBuf {
        self.conversations_dir
            .join(sanitize_branch(branch))
            .join(format!("summary-L{level}.yaml"))
    }
}

pub fn sanitize_branch(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '/' => c,
            ' ' => '-',
            _ => '-',
        })
        .collect()
}

pub fn relative_to_memory<P: AsRef<Path>>(memory_dir: &Path, path: P) -> PathBuf {
    path.as_ref()
        .strip_prefix(memory_dir)
        .unwrap_or(path.as_ref())
        .to_path_buf()
}

/// Write `content` to `path` atomically by creating a temporary file in the
/// same directory, syncing it, and renaming it over the target.
pub fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();
    let parent = path.parent().context("path has no parent directory")?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut tmp, content.as_bytes())?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .map_err(|e| anyhow::anyhow!("failed to persist temp file: {}", e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_branch() {
        assert_eq!(sanitize_branch("feature/new-ui"), "feature/new-ui");
        assert_eq!(sanitize_branch("firebac warfare"), "firebac-warfare");
        assert_eq!(sanitize_branch("weird@name"), "weird-name");
    }

    #[test]
    fn test_relative_to_memory() {
        let memory = Path::new("/home/user/project/memory");
        let file = Path::new("/home/user/project/memory/kg/core.yaml");
        assert_eq!(
            relative_to_memory(memory, file),
            PathBuf::from("kg/core.yaml")
        );
    }

    #[test]
    fn test_score_weights_default() {
        let w = ScoreWeights::default();
        let total = w.importance + w.novelty + w.recency + w.repetition;
        assert!((total - 1.0).abs() < 1e-9);
    }
}
