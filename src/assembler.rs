use crate::character::Character;
use crate::groq_client::MemoryDelta;
use crate::router::ContextQuery;

pub fn assemble_prompt(
    character: &Character,
    context: &ContextQuery,
    user_query: &str,
) -> String {
    let mut sections = Vec::new();

    sections.push(character.system_prompt());

    if !context.kg_facts.is_empty() {
        sections.push("## Knowledge Graph\n".to_string());
        sections.extend(context.kg_facts.iter().map(|f| f.to_string()));
    }

    if !context.recent_messages.is_empty() {
        sections.push("\n## Recent Conversation".to_string());
        for msg in &context.recent_messages {
            sections.push(format!(
                "{}: {}",
                capitalize(&msg.role),
                msg.content
            ));
        }
    }

    if !context.top_memories.is_empty() {
        sections.push("\n## Relevant Memories".to_string());
        for msg in &context.top_memories {
            sections.push(format!(
                "[score={:.2}] {}: {}",
                msg.scores.combined(&crate::config::ScoreWeights::default()),
                capitalize(&msg.role),
                msg.content
            ));
        }
    }

    sections.push(format!("\n## User Query\n{}", user_query));
    sections.push(format_instruction());

    sections.join("\n")
}

fn format_instruction() -> String {
    r#"
## Response Format
Return a JSON object with these fields:
- "response": your natural language reply to the user
- "kg_updates": array of objects { "path": "dot.path", "value": any } for new stable facts
- "memory_updates": array of strings for learned mutable facts about the user or conversation
- "current_topic": optional string describing the active topic

Be concise and grounded in the provided context.
"#
    .to_string()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn apply_delta(
    delta: &MemoryDelta,
    character: &mut Character,
) -> Vec<String> {
    let mut updates = Vec::new();

    for fact in &delta.memory_updates {
        if !character.memory.learned.contains(fact) {
            character.memory.learned.push(fact.clone());
            updates.push(format!("learned: {}", fact));
        }
    }

    if let Some(topic) = &delta.current_topic {
        character.memory.current_topic = Some(topic.clone());
        updates.push(format!("topic: {}", topic));
    }

    updates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{CharacterCore, CharacterMemory};
    use crate::groq_client::MemoryDelta;
    use crate::memory::{Message, MemoryScores};
    use chrono::Utc;

    fn dummy_character() -> Character {
        Character {
            core: CharacterCore {
                name: "Test".into(),
                personality: vec!["helpful".into()],
                constraints: vec!["be concise".into()],
            },
            memory: CharacterMemory::default(),
            name: "test".into(),
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

    #[test]
    fn test_assemble_prompt_includes_context() {
        let character = dummy_character();
        let ctx = ContextQuery {
            kg_facts: vec!["user.name: Rory".into()],
            recent_messages: vec![dummy_message("user", "hello")],
            top_memories: vec![],
        };

        let prompt = assemble_prompt(&character, &ctx, "what is my name?");
        assert!(prompt.contains("Rory"));
        assert!(prompt.contains("hello"));
        assert!(prompt.contains("what is my name?"));
        assert!(prompt.contains("Response Format"));
    }

    #[test]
    fn test_apply_delta_updates_memory() {
        let mut character = dummy_character();
        let delta = MemoryDelta {
            kg_updates: vec![],
            memory_updates: vec!["likes rust".into()],
            current_topic: Some("testing".into()),
        };

        let updates = apply_delta(&delta, &mut character);
        assert!(character.memory.learned.contains(&"likes rust".into()));
        assert_eq!(character.memory.current_topic, Some("testing".into()));
        assert!(!updates.is_empty());
    }
}
