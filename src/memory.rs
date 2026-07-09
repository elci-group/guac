use crate::config::{atomic_write, relative_to_memory, Config, ScoreWeights};
use crate::validation::{sanitize_commit_message, validate_branch};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryScores {
    pub importance: f64,
    pub novelty: f64,
    pub recency: f64,
    pub repetition: f64,
}

impl MemoryScores {
    pub fn combined(&self, weights: &ScoreWeights) -> f64 {
        weights.importance * self.importance
            + weights.novelty * self.novelty
            + weights.recency * self.recency
            + weights.repetition * self.repetition
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub scores: MemoryScores,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub branch: String,
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let text = fs::read_to_string(path)?;
            let conv: Conversation = serde_yaml::from_str(&text)
                .with_context(|| format!("parsing {}", path.display()))?;
            Ok(conv)
        } else {
            Ok(Conversation {
                branch: "main".to_string(),
                messages: Vec::new(),
            })
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_yaml::to_string(self)?;
        atomic_write(path, &text).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    pub fn recent(&self, n: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }

    pub fn top_scored(&self, weights: &ScoreWeights, n: usize, threshold: f64) -> Vec<&Message> {
        let mut scored: Vec<(f64, &Message)> = self
            .messages
            .iter()
            .map(|m| (m.scores.combined(weights), m))
            .filter(|(s, _)| *s >= threshold)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(n);
        scored.into_iter().map(|(_, m)| m).collect()
    }
}

pub struct GitMemory {
    pub repo_path: PathBuf,
}

impl GitMemory {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
        }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.repo_path)?;
        self.git(&["init"])?;
        self.git(&["config", "user.email", "guac@local"])?;
        self.git(&["config", "user.name", "GUAC"])?;
        // Ensure the default branch is named 'main' regardless of git defaults.
        if let Ok(branch) = self.current_branch() {
            if branch == "master" {
                self.git(&["branch", "-m", "main"])?;
            }
        }
        Ok(())
    }

    pub fn is_repo(&self) -> bool {
        self.repo_path.join(".git").is_dir()
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .with_context(|| format!("running git {:?}", args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git {:?} failed: {}", args, stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn add_all(&self) -> Result<()> {
        self.git(&["add", "."])?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        self.add_all()?;
        match self.git(&["commit", "-m", message]) {
            Ok(_) => Ok(()),
            Err(e) => {
                let status = self.git(&["status", "--porcelain"])?;
                if status.trim().is_empty() {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn branch_create(&self, name: &str) -> Result<()> {
        self.git(&["branch", name])?;
        Ok(())
    }

    pub fn checkout(&self, name: &str) -> Result<()> {
        self.git(&["checkout", name])?;
        Ok(())
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let out = self.git(&["branch", "--format=%(refname:short)"])?;
        Ok(out.lines().map(|s| s.trim().to_string()).collect())
    }

    pub fn current_branch(&self) -> Result<String> {
        self.git(&["branch", "--show-current"])
    }

    pub fn log_oneline(&self, n: usize) -> Result<String> {
        self.git(&["log", "--oneline", "-n", &n.to_string()])
    }
}

pub struct MemoryManager {
    pub git: GitMemory,
    pub config: Config,
}

impl MemoryManager {
    pub fn new(config: &Config) -> Self {
        Self {
            git: GitMemory::new(&config.memory_dir),
            config: config.clone(),
        }
    }

    pub fn load_conversation(&self, branch: &str) -> Result<Conversation> {
        validate_branch(branch)?;
        let path = self.config.conversation_path(branch);
        let mut conv = Conversation::load(&path)?;
        conv.branch = branch.to_string();
        Ok(conv)
    }

    pub fn save_conversation(&self, conv: &Conversation) -> Result<PathBuf> {
        validate_branch(&conv.branch)?;
        let path = self.config.conversation_path(&conv.branch);
        conv.save(&path)?;
        Ok(path)
    }

    pub fn append_message(
        &self,
        conv: &mut Conversation,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let timestamp = Utc::now();
        let scores = score_message(content, &conv.messages, &self.config.score_weights);
        conv.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp,
            scores,
            summary: None,
        });
        Ok(())
    }

    pub fn commit_memory(&self, message: &str, paths: &[PathBuf]) -> Result<()> {
        let message = sanitize_commit_message(message);
        let rel_paths: Vec<String> = paths
            .iter()
            .map(|p| relative_to_memory(&self.config.memory_dir, p).display().to_string())
            .collect();
        let full = format!("{}\n\nFiles:\n{}", message, rel_paths.join("\n"));
        self.git.commit(&full)?;
        Ok(())
    }
}

pub fn score_message(content: &str, history: &[Message], _weights: &ScoreWeights) -> MemoryScores {
    let words: HashSet<String> = tokenize(content);

    // Importance: length, punctuation, question/exclamation signals
    let mut importance = 0.3;
    if content.contains('?') {
        importance += 0.15;
    }
    if content.contains('!') {
        importance += 0.05;
    }
    let word_count = words.len().max(1);
    importance += (word_count as f64 / 50.0).min(0.3);
    importance = importance.min(1.0);

    // Novelty: average Jaccard distance from previous messages
    let novelty = if history.is_empty() {
        1.0
    } else {
        let mut total = 0.0;
        for msg in history.iter().rev().take(10) {
            let prev: HashSet<String> = tokenize(&msg.content);
            let union_len = words.union(&prev).count().max(1);
            let inter_len = words.intersection(&prev).count();
            total += 1.0 - (inter_len as f64 / union_len as f64);
        }
        (total / history.len().min(10) as f64).min(1.0)
    };

    // Recency: position-based, newest = 1.0
    let recency = 1.0;

    // Repetition: how many previous messages share significant words
    let repetition = if history.is_empty() {
        0.0
    } else {
        let similar = history
            .iter()
            .rev()
            .take(20)
            .filter(|msg| {
                let prev: HashSet<String> = tokenize(&msg.content);
                let inter = words.intersection(&prev).count() as f64;
                let min_len = words.len().min(prev.len()).max(1) as f64;
                inter / min_len > 0.5
            })
            .count();
        (similar as f64 / 10.0).min(1.0)
    };

    MemoryScores {
        importance,
        novelty,
        recency,
        repetition,
    }
}

fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 2)
        .map(|s| s.to_string())
        .collect()
}

pub fn update_recency_scores(conv: &mut Conversation) {
    let n = conv.messages.len();
    for (i, msg) in conv.messages.iter_mut().enumerate() {
        let age = (n - i - 1) as f64;
        msg.scores.recency = (-age / 10.0).exp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_config(base: &std::path::Path) -> Config {
        Config {
            base_dir: base.to_path_buf(),
            memory_dir: base.join("memory"),
            kg_dir: base.join("memory/kg"),
            conversations_dir: base.join("memory/conversations"),
            characters_dir: base.join("memory/characters"),
            groq_api_key: None,
            groq_model: "test-model".into(),
            max_context_messages: 5,
            max_memory_messages: 10,
            compression_threshold: 3,
            score_weights: ScoreWeights::default(),
            memory_threshold: 0.25,
        }
    }

    #[test]
    fn test_git_memory_init_and_commit() {
        let dir = TempDir::new().unwrap();
        let git = GitMemory::new(dir.path());
        assert!(!git.is_repo());
        git.init().unwrap();
        assert!(git.is_repo());
        assert_eq!(git.current_branch().unwrap(), "main");

        std::fs::write(dir.path().join("file.txt"), "hello").unwrap();
        git.commit("test commit").unwrap();
        assert!(git.log_oneline(1).unwrap().contains("test commit"));
    }

    #[test]
    fn test_git_branch_and_checkout() {
        let dir = TempDir::new().unwrap();
        let git = GitMemory::new(dir.path());
        git.init().unwrap();
        std::fs::write(dir.path().join("file.txt"), "main").unwrap();
        git.commit("main commit").unwrap();

        git.branch_create("feature").unwrap();
        git.checkout("feature").unwrap();
        assert_eq!(git.current_branch().unwrap(), "feature");

        git.checkout("main").unwrap();
        assert_eq!(git.current_branch().unwrap(), "main");

        let branches = git.list_branches().unwrap();
        assert!(branches.contains(&"main".into()));
        assert!(branches.contains(&"feature".into()));
    }

    #[test]
    fn test_conversation_save_and_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("chat.yaml");
        let mut conv = Conversation {
            branch: "main".into(),
            messages: vec![],
        };
        conv.messages.push(Message {
            role: "user".into(),
            content: "hello".into(),
            timestamp: Utc::now(),
            scores: MemoryScores {
                importance: 0.5,
                novelty: 0.5,
                recency: 1.0,
                repetition: 0.0,
            },
            summary: None,
        });
        conv.save(&path).unwrap();

        let loaded = Conversation::load(&path).unwrap();
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "hello");
    }

    #[test]
    fn test_memory_manager_append_and_commit() {
        let dir = TempDir::new().unwrap();
        let config = make_config(dir.path());
        config.ensure_dirs().unwrap();

        let memory = MemoryManager::new(&config);
        memory.git.init().unwrap();

        let mut conv = memory.load_conversation("main").unwrap();
        memory.append_message(&mut conv, "user", "hello").unwrap();
        let path = memory.save_conversation(&conv).unwrap();
        memory.commit_memory("test message", &[path]).unwrap();

        assert!(memory.git.log_oneline(1).unwrap().contains("test message"));
    }

    #[test]
    fn test_score_message() {
        let scores = score_message("What is my name?", &[], &ScoreWeights::default());
        assert!(scores.importance > 0.4);
        assert_eq!(scores.novelty, 1.0);

        let history = vec![Message {
            role: "user".into(),
            content: "What is my name?".into(),
            timestamp: Utc::now(),
            scores: MemoryScores {
                importance: 0.5,
                novelty: 0.5,
                recency: 1.0,
                repetition: 0.0,
            },
            summary: None,
        }];
        let scores2 = score_message("What is my name?", &history, &ScoreWeights::default());
        assert!(scores2.repetition > 0.0);
        assert!(scores2.novelty < scores.novelty);
    }

    #[test]
    fn test_update_recency_scores() {
        let mut conv = Conversation {
            branch: "main".into(),
            messages: vec![
                dummy_message("old"),
                dummy_message("middle"),
                dummy_message("new"),
            ],
        };
        update_recency_scores(&mut conv);
        assert!(conv.messages[2].scores.recency > conv.messages[0].scores.recency);
    }

    fn dummy_message(content: &str) -> Message {
        Message {
            role: "user".into(),
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
}
