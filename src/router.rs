use crate::classifier::Intent;
use crate::config::Config;
use crate::kg::KnowledgeGraph;
use crate::memory::{MemoryManager, Message};
use anyhow::Result;

pub struct ContextQuery {
    pub kg_facts: Vec<String>,
    pub recent_messages: Vec<Message>,
    pub top_memories: Vec<Message>,
}

pub async fn gather_context(
    intent: Intent,
    query: &str,
    kg: &KnowledgeGraph,
    memory: &MemoryManager,
    branch: &str,
    config: &Config,
) -> Result<ContextQuery> {
    let conv = memory.load_conversation(branch)?;

    let kg_facts = match intent {
        Intent::KnowledgeGraph | Intent::Both => extract_kg_facts(kg, query),
        Intent::Memory => Vec::new(),
    };

    let recent_messages = match intent {
        Intent::Memory | Intent::Both => conv.recent(config.max_context_messages).to_vec(),
        Intent::KnowledgeGraph => Vec::new(),
    };

    let top_memories = match intent {
        Intent::Memory | Intent::Both => conv
            .top_scored(&config.score_weights, config.max_memory_messages, config.memory_threshold)
            .into_iter()
            .cloned()
            .collect(),
        Intent::KnowledgeGraph => Vec::new(),
    };

    Ok(ContextQuery {
        kg_facts,
        recent_messages,
        top_memories,
    })
}

fn extract_kg_facts(kg: &KnowledgeGraph, query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let paths = kg.paths();
    let mut matches = Vec::new();

    // Direct path lookup if query contains a path-like string
    for path in &paths {
        if lower.contains(&path.replace('.', " ").to_lowercase())
            || lower.contains(&path.replace('.', "").to_lowercase())
            || lower.contains(&path.to_lowercase())
        {
            if let Some(value) = kg.get(path) {
                matches.push(format!("{}: {}", path, crate::kg::describe_value(value)));
            }
        }
    }

    // Fallback: include top-level keys if no direct matches
    if matches.is_empty() {
        if let Some(top) = kg.data().as_mapping() {
            for (k, v) in top.iter().take(5) {
                if let Some(key) = k.as_str() {
                    matches.push(format!(
                        "{}: {}",
                        key,
                        crate::kg::describe_value(v)
                    ));
                }
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::memory::MemoryManager;
    use serde_yaml::Value;
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
            max_context_messages: 2,
            max_memory_messages: 5,
            compression_threshold: 3,
            score_weights: crate::config::ScoreWeights::default(),
            memory_threshold: 0.0,
        }
    }

    #[tokio::test]
    async fn test_gather_context_kg_only() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        config.ensure_dirs().unwrap();

        let mut kg = KnowledgeGraph::load(&config.kg_dir).unwrap();
        kg.set("user.name", Value::String("Rory".into())).unwrap();

        let memory = MemoryManager::new(&config);
        memory.git.init().unwrap();

        let ctx = gather_context(Intent::KnowledgeGraph, "what is user.name", &kg, &memory, "main", &config)
            .await
            .unwrap();

        assert!(!ctx.kg_facts.is_empty());
        assert!(ctx.recent_messages.is_empty());
    }

    #[tokio::test]
    async fn test_gather_context_memory_only() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        config.ensure_dirs().unwrap();

        let kg = KnowledgeGraph::load(&config.kg_dir).unwrap();
        let memory = MemoryManager::new(&config);
        memory.git.init().unwrap();

        let mut conv = memory.load_conversation("main").unwrap();
        memory.append_message(&mut conv, "user", "hello").unwrap();
        memory.save_conversation(&conv).unwrap();

        let ctx = gather_context(Intent::Memory, "recall hello", &kg, &memory, "main", &config)
            .await
            .unwrap();

        assert!(ctx.kg_facts.is_empty());
        assert_eq!(ctx.recent_messages.len(), 1);
    }
}
