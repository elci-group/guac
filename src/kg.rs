use crate::config::atomic_write;
use anyhow::{Context, Result};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct KnowledgeGraph {
    root: PathBuf,
    data: Value,
}

impl KnowledgeGraph {
    pub fn load(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        let core = root.join("core.yaml");
        let data = if core.exists() {
            let text = fs::read_to_string(&core)?;
            serde_yaml::from_str(&text).unwrap_or_else(|_| Value::Mapping(serde_yaml::Mapping::new()))
        } else {
            Value::Mapping(serde_yaml::Mapping::new())
        };
        Ok(Self {
            root: root.to_path_buf(),
            data,
        })
    }

    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut current = &self.data;
        for segment in path.split('.') {
            match current {
                Value::Mapping(m) => current = m.get(segment)?,
                Value::Sequence(s) => {
                    let idx: usize = segment.parse().ok()?;
                    current = s.get(idx)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get(path).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => Some(serde_yaml::to_string(v).ok()?.trim().to_string()),
        })
    }

    pub fn set(&mut self, path: &str, value: Value) -> Result<()> {
        let mut current = &mut self.data;
        let segments: Vec<&str> = path.split('.').collect();
        if segments.is_empty() {
            self.data = value;
            return Ok(());
        }

        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;
            if is_last {
                match current {
                    Value::Mapping(m) => {
                        m.insert(Value::String(segment.to_string()), value);
                    }
                    _ => return Err(anyhow::anyhow!("cannot set path {}: not a mapping", path)),
                }
                break;
            }

            match current {
                Value::Mapping(m) => {
                    let key = Value::String(segment.to_string());
                    if !m.contains_key(&key) {
                        m.insert(
                            key.clone(),
                            Value::Mapping(serde_yaml::Mapping::new()),
                        );
                    }
                    current = m
                        .get_mut(&key)
                        .ok_or_else(|| anyhow::anyhow!("path creation failed"))?;
                }
                _ => return Err(anyhow::anyhow!("cannot set path {}: not a mapping", path)),
            }
        }
        Ok(())
    }

    pub fn set_str(&mut self, path: &str, value: &str) -> Result<()> {
        let parsed = serde_yaml::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()));
        self.set(path, parsed)
    }

    pub fn merge(&mut self, updates: HashMap<String, Value>) -> Result<()> {
        for (path, value) in updates {
            self.set(&path, value)?;
        }
        Ok(())
    }

    pub fn paths(&self) -> Vec<String> {
        let mut out = Vec::new();
        Self::collect_paths(&self.data, "", &mut out);
        out
    }

    fn collect_paths(value: &Value, prefix: &str, out: &mut Vec<String>) {
        match value {
            Value::Mapping(m) => {
                for (k, v) in m {
                    if let Some(key) = k.as_str() {
                        let new_prefix = if prefix.is_empty() {
                            key.to_string()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        if !v.is_mapping() && !v.is_sequence() {
                            out.push(new_prefix.clone());
                        }
                        Self::collect_paths(v, &new_prefix, out);
                    }
                }
            }
            Value::Sequence(s) => {
                for (i, v) in s.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        i.to_string()
                    } else {
                        format!("{}.{}", prefix, i)
                    };
                    if !v.is_mapping() && !v.is_sequence() {
                        out.push(new_prefix.clone());
                    }
                    Self::collect_paths(v, &new_prefix, out);
                }
            }
            _ => {}
        }
    }

    pub fn save(&self) -> Result<PathBuf> {
        let core = self.root.join("core.yaml");
        let text = serde_yaml::to_string(&self.data)?;
        atomic_write(&core, &text).with_context(|| format!("writing {}", core.display()))?;
        Ok(core)
    }

    pub fn data(&self) -> &Value {
        &self.data
    }
}

pub fn describe_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => serde_yaml::to_string(value).unwrap_or_default().trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_kg_set_and_get() {
        let dir = TempDir::new().unwrap();
        let mut kg = KnowledgeGraph::load(dir.path()).unwrap();

        kg.set("user.name", Value::String("Rory".into())).unwrap();
        kg.set("user.age", Value::Number(42.into())).unwrap();

        assert_eq!(
            kg.get("user.name"),
            Some(&Value::String("Rory".into()))
        );
        assert_eq!(kg.get("user.age"), Some(&Value::Number(42.into())));
    }

    #[test]
    fn test_kg_set_str_parses_yaml() {
        let dir = TempDir::new().unwrap();
        let mut kg = KnowledgeGraph::load(dir.path()).unwrap();

        kg.set_str("projects", "[Marina, FireBAC]").unwrap();
        let projects = kg.get("projects").unwrap();
        assert!(projects.is_sequence());
        assert_eq!(projects.as_sequence().unwrap().len(), 2);

        kg.set_str("user.active", "true").unwrap();
        assert_eq!(kg.get("user.active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_kg_merge() {
        let dir = TempDir::new().unwrap();
        let mut kg = KnowledgeGraph::load(dir.path()).unwrap();

        let mut updates = HashMap::new();
        updates.insert("user.name".into(), Value::String("Rory".into()));
        updates.insert("user.timezone".into(), Value::String("Europe/London".into()));
        kg.merge(updates).unwrap();

        assert_eq!(
            kg.get("user.name"),
            Some(&Value::String("Rory".into()))
        );
        assert_eq!(
            kg.get("user.timezone"),
            Some(&Value::String("Europe/London".into()))
        );
    }

    #[test]
    fn test_kg_save_and_load() {
        let dir = TempDir::new().unwrap();
        {
            let mut kg = KnowledgeGraph::load(dir.path()).unwrap();
            kg.set("user.name", Value::String("Rory".into())).unwrap();
            kg.save().unwrap();
        }

        let kg2 = KnowledgeGraph::load(dir.path()).unwrap();
        assert_eq!(
            kg2.get("user.name"),
            Some(&Value::String("Rory".into()))
        );
    }

    #[test]
    fn test_kg_paths() {
        let dir = TempDir::new().unwrap();
        let mut kg = KnowledgeGraph::load(dir.path()).unwrap();
        kg.set_str("projects", "[Marina, FireBAC]").unwrap();
        kg.set("user.name", Value::String("Rory".into())).unwrap();

        let paths = kg.paths();
        assert!(paths.contains(&"user.name".into()));
        assert!(paths.contains(&"projects.0".into()));
        assert!(paths.contains(&"projects.1".into()));
    }

    #[test]
    fn test_describe_value() {
        assert_eq!(describe_value(&Value::String("hello".into())), "hello");
        assert!(describe_value(&Value::Number(42.into())).contains("42"));
    }
}
