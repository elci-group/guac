use crate::config::Config;
use anyhow::{Context, Result};
use backon::{ExponentialBuilder, Retryable};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryDelta {
    #[serde(default)]
    pub kg_updates: Vec<KgUpdate>,
    #[serde(default)]
    pub memory_updates: Vec<String>,
    #[serde(default)]
    pub current_topic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgUpdate {
    pub path: String,
    pub value: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqResponse {
    pub response: String,
    #[serde(flatten)]
    pub delta: MemoryDelta,
}

pub struct GroqClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    max_retries: usize,
}

impl GroqClient {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .groq_api_key
            .clone()
            .context("GROQ_API_KEY not set")?;
        Ok(Self {
            client: Client::new(),
            api_key,
            model: config.groq_model.clone(),
            base_url: GROQ_API_URL.to_string(),
            max_retries: 3,
        })
    }

    #[cfg(test)]
    pub fn with_base_url(config: &Config, base_url: &str) -> Result<Self> {
        let api_key = config
            .groq_api_key
            .clone()
            .context("GROQ_API_KEY not set")?;
        Ok(Self {
            client: Client::new(),
            api_key,
            model: config.groq_model.clone(),
            base_url: base_url.to_string(),
            max_retries: 3,
        })
    }

    pub fn is_available(config: &Config) -> bool {
        config.groq_api_key.is_some()
    }

    #[instrument(skip(self, system, user), fields(model = %self.model))]
    pub async fn chat(&self, system: &str, user: &str) -> Result<GroqResponse> {
        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": 0.7,
            "max_tokens": 1024,
            "response_format": { "type": "json_object" }
        });

        debug!("sending chat request to Groq");

        let do_request = || async {
            let resp = self
                .client
                .post(&self.base_url)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .context("sending request to Groq")?;

            let status = resp.status();
            if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS {
                let text = resp.text().await.unwrap_or_default();
                warn!(status = %status, "transient Groq error; retrying");
                return Err::<reqwest::Response, anyhow::Error>(anyhow::anyhow!(
                    "transient Groq error {}: {}",
                    status,
                    text
                ));
            }

            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Groq API error {}: {}", status, text));
            }

            Ok(resp)
        };

        let retry = ExponentialBuilder::default()
            .with_min_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(5))
            .with_jitter()
            .with_max_times(self.max_retries);

        let resp = do_request
            .retry(&retry)
            .when(|e| {
                // Retry on network errors and transient server errors.
                let s = e.to_string();
                s.contains("transient") || s.contains("sending request")
            })
            .await
            .map_err(|e| {
                error!(error = %e, "Groq request failed after retries");
                e
            })?;

        info!(status = %resp.status(), "received Groq response");

        let json: Value = resp.json().await.context("parsing Groq response")?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("missing content in Groq response")?;

        Self::parse_response(content)
    }

    pub async fn classify_intent(&self, query: &str) -> Result<String> {
        let system = "You are an intent classifier. Return ONLY a JSON object with a single field 'intent' whose value is one of: 'kg', 'memory', or 'both'. 'kg' means the user wants a fact from a knowledge graph. 'memory' means the user wants to recall a past conversation. 'both' means reasoning over both.";
        let user = format!("Classify this query: {}", query);
        let resp = self.chat(system, &user).await?;
        let intent = resp.response.to_lowercase();
        Ok(match intent.as_str() {
            "kg" => "kg".to_string(),
            "memory" => "memory".to_string(),
            _ => "both".to_string(),
        })
    }

    pub async fn summarize(&self, text: &str) -> Result<String> {
        let system = "You are a memory compression engine. Summarize the following conversation segment into 1-3 concise sentences preserving key facts, decisions, and topics. Return ONLY a JSON object with field 'summary'.";
        let resp = self.chat(system, text).await?;
        Ok(resp.response)
    }

    fn parse_response(content: &str) -> Result<GroqResponse> {
        let cleaned = content.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        let value: Value = serde_json::from_str(cleaned).context("parsing JSON response")?;

        // Accept either { response, kg_updates, ... } or { summary, ... }
        let response = value
            .get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                value
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "(no response field)".to_string());

        let kg_updates: Vec<KgUpdate> = value
            .get("kg_updates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let path = item.get("path")?.as_str()?.to_string();
                        let json_value = item.get("value")?;
                        let yaml_value = serde_yaml::to_value(json_value).ok()?;
                        Some(KgUpdate { path, value: yaml_value })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let memory_updates: Vec<String> = value
            .get("memory_updates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let current_topic = value
            .get("current_topic")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(GroqResponse {
            response,
            delta: MemoryDelta {
                kg_updates,
                memory_updates,
                current_topic,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use mockito::Server;

    fn test_config() -> Config {
        Config {
            base_dir: std::path::PathBuf::from("/tmp"),
            memory_dir: std::path::PathBuf::from("/tmp/memory"),
            kg_dir: std::path::PathBuf::from("/tmp/memory/kg"),
            conversations_dir: std::path::PathBuf::from("/tmp/memory/conversations"),
            characters_dir: std::path::PathBuf::from("/tmp/memory/characters"),
            groq_api_key: Some("test-key".into()),
            groq_model: "test-model".into(),
            max_context_messages: 5,
            max_memory_messages: 10,
            compression_threshold: 3,
            score_weights: crate::config::ScoreWeights::default(),
            memory_threshold: 0.25,
        }
    }

    #[tokio::test]
    async fn test_chat_parses_response_and_delta() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/openai/v1/chat/completions")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "content": "{\"response\":\"Hello Rory\",\"kg_updates\":[{\"path\":\"user.name\",\"value\":\"Rory\"}],\"memory_updates\":[\"likes concise answers\"],\"current_topic\":\"greeting\"}"
                    }
                }]
            }"#)
            .create();

        let config = test_config();
        let url = format!("{}/openai/v1/chat/completions", server.url());
        let client = GroqClient::with_base_url(&config, &url).unwrap();
        let resp = client.chat("system", "hi").await.unwrap();

        assert_eq!(resp.response, "Hello Rory");
        assert_eq!(resp.delta.kg_updates.len(), 1);
        assert_eq!(resp.delta.kg_updates[0].path, "user.name");
        assert_eq!(
            resp.delta.kg_updates[0].value,
            serde_yaml::Value::String("Rory".into())
        );
        assert!(resp.delta.memory_updates.contains(&"likes concise answers".into()));
        assert_eq!(resp.delta.current_topic, Some("greeting".into()));

        mock.assert();
    }

    #[tokio::test]
    async fn test_chat_handles_markdown_json() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/openai/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "content": "```json\n{\"response\":\"ok\"}\n```"
                    }
                }]
            }"#)
            .create();

        let config = test_config();
        let url = format!("{}/openai/v1/chat/completions", server.url());
        let client = GroqClient::with_base_url(&config, &url).unwrap();
        let resp = client.chat("system", "hi").await.unwrap();
        assert_eq!(resp.response, "ok");
        mock.assert();
    }

    #[tokio::test]
    async fn test_classify_intent() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/openai/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "content": "{\"response\":\"kg\"}"
                    }
                }]
            }"#)
            .create();

        let config = test_config();
        let url = format!("{}/openai/v1/chat/completions", server.url());
        let client = GroqClient::with_base_url(&config, &url).unwrap();
        let intent = client.classify_intent("What is FireBAC?").await.unwrap();
        assert_eq!(intent, "kg");
        mock.assert();
    }

    #[tokio::test]
    async fn test_api_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/openai/v1/chat/completions")
            .with_status(401)
            .with_body(r#"{"error":{"message":"Invalid API Key"}}"#)
            .create();

        let config = test_config();
        let url = format!("{}/openai/v1/chat/completions", server.url());
        let client = GroqClient::with_base_url(&config, &url).unwrap();
        assert!(client.chat("system", "hi").await.is_err());
        mock.assert();
    }
}
