use crate::config::{atomic_write, Config};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterCore {
    pub name: String,
    #[serde(default)]
    pub personality: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterMemory {
    #[serde(default)]
    pub learned: Vec<String>,
    #[serde(default)]
    pub current_topic: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub core: CharacterCore,
    pub memory: CharacterMemory,
    pub name: String,
}

impl Character {
    pub fn load(config: &Config, name: &str) -> Result<Self> {
        let core_path = config.character_core_path(name);
        let memory_path = config.character_memory_path(name);

        fs::create_dir_all(core_path.parent().unwrap())?;

        let core = if core_path.exists() {
            let text = fs::read_to_string(&core_path)?;
            serde_yaml::from_str(&text)
                .with_context(|| format!("parsing {}", core_path.display()))?
        } else {
            CharacterCore {
                name: name.to_string(),
                ..Default::default()
            }
        };

        let memory = if memory_path.exists() {
            let text = fs::read_to_string(&memory_path)?;
            serde_yaml::from_str(&text)
                .with_context(|| format!("parsing {}", memory_path.display()))?
        } else {
            CharacterMemory::default()
        };

        Ok(Self {
            core,
            memory,
            name: name.to_string(),
        })
    }

    pub fn ensure_default(config: &Config) -> Result<()> {
        let core_path = config.character_core_path("default");
        if !core_path.exists() {
            fs::create_dir_all(core_path.parent().unwrap())?;
            let default = CharacterCore {
                name: "GUAC".to_string(),
                personality: vec![
                    "curious".to_string(),
                    "analytical".to_string(),
                    "helpful".to_string(),
                ],
                constraints: vec![
                    "cannot alter core values".to_string(),
                    "must ground responses in memory and knowledge graph".to_string(),
                ],
            };
            fs::write(&core_path, serde_yaml::to_string(&default)?)
                .with_context(|| format!("writing {}", core_path.display()))?;
        }
        Ok(())
    }

    pub fn system_prompt(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("You are {}.", self.core.name));
        if !self.core.personality.is_empty() {
            parts.push(format!(
                "Personality: {}",
                self.core.personality.join(", ")
            ));
        }
        if !self.core.constraints.is_empty() {
            parts.push(format!(
                "Constraints: {}",
                self.core.constraints.join("; ")
            ));
        }
        if !self.memory.learned.is_empty() {
            parts.push(format!(
                "Learned facts: {}",
                self.memory.learned.join("; ")
            ));
        }
        if let Some(topic) = &self.memory.current_topic {
            parts.push(format!("Current topic: {}", topic));
        }
        parts.push(
            "You have access to a knowledge graph and conversation memory. \
             Respond helpfully and update memory when facts change."
                .to_string(),
        );
        parts.join("\n")
    }

    pub fn save_memory(&self, config: &Config) -> Result<()> {
        let memory_path = config.character_memory_path(&self.name);
        fs::create_dir_all(memory_path.parent().unwrap())?;
        atomic_write(&memory_path, &serde_yaml::to_string(&self.memory)?)
            .with_context(|| format!("writing {}", memory_path.display()))?;
        Ok(())
    }
}

pub fn apply_memory_updates(character: &mut Character, updates: &[Value]) {
    for update in updates {
        if let Some(s) = update.as_str() {
            if !character.memory.learned.contains(&s.to_string()) {
                character.memory.learned.push(s.to_string());
            }
        }
    }
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
    fn test_character_ensure_default() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        Character::ensure_default(&config).unwrap();

        let core_path = config.character_core_path("default");
        assert!(core_path.exists());

        let character = Character::load(&config, "default").unwrap();
        assert_eq!(character.core.name, "GUAC");
        assert!(!character.core.personality.is_empty());
    }

    #[test]
    fn test_character_memory_persistence() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        Character::ensure_default(&config).unwrap();

        let mut character = Character::load(&config, "default").unwrap();
        character.memory.learned.push("user likes concise answers".into());
        character.memory.current_topic = Some("FireBAC".into());
        character.save_memory(&config).unwrap();

        let character2 = Character::load(&config, "default").unwrap();
        assert!(character2.memory.learned.contains(&"user likes concise answers".into()));
        assert_eq!(character2.memory.current_topic, Some("FireBAC".into()));
    }

    #[test]
    fn test_system_prompt_contains_core_and_memory() {
        let dir = TempDir::new().unwrap();
        let config = test_config(dir.path());
        Character::ensure_default(&config).unwrap();

        let mut character = Character::load(&config, "default").unwrap();
        character.memory.learned.push("fact".into());
        character.memory.current_topic = Some("topic".into());

        let prompt = character.system_prompt();
        assert!(prompt.contains("GUAC"));
        assert!(prompt.contains("fact"));
        assert!(prompt.contains("topic"));
    }
}
