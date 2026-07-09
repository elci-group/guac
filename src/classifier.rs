use crate::groq_client::GroqClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    KnowledgeGraph,
    Memory,
    Both,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Intent::KnowledgeGraph => "kg",
            Intent::Memory => "memory",
            Intent::Both => "both",
        }
    }
}

pub fn rule_classify(query: &str) -> Intent {
    let lower = query.to_lowercase();
    let kg_phrases = [
        "what is",
        "who is",
        "where is",
        "when is",
        "tell me about",
        "what are",
        "define",
        "lookup",
        "kg:",
    ];
    let memory_phrases = [
        "remember",
        "recall",
        "what did",
        "when did",
        "last time",
        "previously",
        "we discussed",
        "memory:",
    ];

    let is_kg = kg_phrases.iter().any(|p| lower.contains(p));
    let is_mem = memory_phrases.iter().any(|p| lower.contains(p));

    match (is_kg, is_mem) {
        (true, false) => Intent::KnowledgeGraph,
        (false, true) => Intent::Memory,
        _ => Intent::Both,
    }
}

pub async fn classify(query: &str, client: Option<&GroqClient>) -> Intent {
    if let Some(client) = client {
        match client.classify_intent(query).await {
            Ok(intent) => match intent.as_str() {
                "kg" => Intent::KnowledgeGraph,
                "memory" => Intent::Memory,
                _ => Intent::Both,
            },
            Err(_) => rule_classify(query),
        }
    } else {
        rule_classify(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_classify_kg() {
        assert_eq!(rule_classify("What is FireBAC?"), Intent::KnowledgeGraph);
        assert_eq!(rule_classify("Who is Rory?"), Intent::KnowledgeGraph);
        assert_eq!(rule_classify("tell me about MRMR"), Intent::KnowledgeGraph);
    }

    #[test]
    fn test_rule_classify_memory() {
        assert_eq!(rule_classify("remember my name"), Intent::Memory);
        assert_eq!(rule_classify("what did we discuss?"), Intent::Memory);
        assert_eq!(rule_classify("previously on FireBAC"), Intent::Memory);
    }

    #[test]
    fn test_rule_classify_both() {
        assert_eq!(rule_classify("hello"), Intent::Both);
        assert_eq!(rule_classify("what is FireBAC and what did we say about it?"), Intent::Both);
    }

    #[tokio::test]
    async fn test_classify_falls_back_without_client() {
        assert_eq!(classify("What is FireBAC?", None).await, Intent::KnowledgeGraph);
    }
}
