use crate::config::Config;
use crate::groq_client::GroqClient;
use crate::memory::{Conversation, Message};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummaryEntry {
    pub level: u32,
    pub timestamp: String,
    pub source_range: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummaryFile {
    pub branch: String,
    pub entries: Vec<SummaryEntry>,
}

impl SummaryFile {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let text = fs::read_to_string(path)?;
            Ok(serde_yaml::from_str(&text)?)
        } else {
            Ok(SummaryFile {
                branch: "main".to_string(),
                entries: Vec::new(),
            })
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_yaml::to_string(self)?)
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

pub async fn compress_branch(
    conv: &mut Conversation,
    config: &Config,
    client: Option<&GroqClient>,
) -> Result<usize> {
    let threshold = config.compression_threshold;
    if conv.messages.len() <= threshold {
        return Ok(0);
    }

    let to_compress_count = conv.messages.len() - threshold;
    let to_compress: Vec<Message> = conv.messages.drain(0..to_compress_count).collect();

    let text = to_compress
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let summary = if let Some(client) = client {
        client.summarize(&text).await.unwrap_or_else(|_| extractive_summary(&to_compress))
    } else {
        extractive_summary(&to_compress)
    };

    let summary_path = config.summary_path(&conv.branch, 1);
    let mut summary_file = SummaryFile::load(&summary_path)?;
    summary_file.branch = conv.branch.clone();
    summary_file.entries.push(SummaryEntry {
        level: 1,
        timestamp: Utc::now().to_rfc3339(),
        source_range: format!("messages {}..{}", 0, to_compress_count),
        summary,
    });
    summary_file.save(&summary_path)?;

    Ok(to_compress_count)
}

fn extractive_summary(messages: &[Message]) -> String {
    let mut parts: Vec<String> = messages
        .iter()
        .filter(|m| m.role == "assistant")
        .map(|m| {
            let content = m.content.trim();
            if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content.to_string()
            }
        })
        .collect();
    parts.dedup();
    if parts.is_empty() {
        "(conversation continued)".to_string()
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::memory::{Conversation, Message, MemoryScores};
    use chrono::Utc;
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
            compression_threshold: 2,
            score_weights: crate::config::ScoreWeights::default(),
            memory_threshold: 0.25,
        }
    }

    fn dummy_message(role: &str, content: &str) -> Message {
        Message {
            role: role.into(),
            content: content.into(),
            timestamp: Utc::now(),
            scores: MemoryScores {
                importance: 0.5,
                novelty: 0.5,
                recency: 1.0,
                repetition: 0.0,
            },
            summary: None,
        }
    }

    #[tokio::test]
    async fn test_compress_branch_without_groq() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        config.ensure_dirs().unwrap();

        let mut conv = Conversation {
            branch: "main".into(),
            messages: vec![
                dummy_message("user", "hi"),
                dummy_message("assistant", "hello there"),
                dummy_message("user", "bye"),
                dummy_message("assistant", "goodbye"),
            ],
        };

        let count = compress_branch(&mut conv, &config, None).await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(conv.messages.len(), 2);

        let summary_path = config.summary_path("main", 1);
        assert!(summary_path.exists());
        let summary = SummaryFile::load(&summary_path).unwrap();
        assert_eq!(summary.entries.len(), 1);
        assert!(summary.entries[0].summary.contains("hello there"));
    }

    #[tokio::test]
    async fn test_compress_branch_no_op_when_under_threshold() {
        let dir = TempDir::new().unwrap();
        let mut config = test_config(dir.path());
        config.compression_threshold = 10;
        config.ensure_dirs().unwrap();

        let mut conv = Conversation {
            branch: "main".into(),
            messages: vec![dummy_message("user", "hi")],
        };

        let count = compress_branch(&mut conv, &config, None).await.unwrap();
        assert_eq!(count, 0);
    }
}
