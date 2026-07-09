#![allow(dead_code)]

mod assembler;
mod character;
mod classifier;
mod cli;
mod compression;
mod config;
mod fs_memory;
mod groq_client;
mod kg;
mod memory;
mod router;
mod validation;

use anyhow::Result;
use clap::Parser;
use cli::{BranchCommands, Cli, Commands, FsCommands, KgCommands};
use config::Config;
use std::io::{self, Write};
use tracing::{info, instrument, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Init => cmd_init(&config).await,
        Commands::Chat { branch, character } => cmd_chat(&config, &branch, &character).await,
        Commands::Kg { command } => cmd_kg(&config, command).await,
        Commands::Recall { query, branch } => cmd_recall(&config, &query, &branch).await,
        Commands::Branch { command } => cmd_branch(&config, command).await,
        Commands::Compress { branch } => cmd_compress(&config, &branch).await,
        Commands::Status => cmd_status(&config).await,
        Commands::Fs { command } => cmd_fs(&config, command).await,
    }
}

#[instrument(skip(config))]
async fn cmd_init(config: &Config) -> Result<()> {
    info!("initialising GUAC memory repository");
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    if !memory.git.is_repo() {
        memory.git.init()?;
    }

    character::Character::ensure_default(config)?;

    let mut kg = kg::KnowledgeGraph::load(&config.kg_dir)?;
    kg.set("guac.version", serde_yaml::Value::String("0.1.0".to_string()))?;
    let kg_path = kg.save()?;

    let conv = memory.load_conversation("main")?;
    let conv_path = memory.save_conversation(&conv)?;

    memory.commit_memory(
        "init: GUAC memory repository",
        &[kg_path, conv_path],
    )?;

    println!("Initialized GUAC memory repository at {}", config.memory_dir.display());
    println!("Current branch: {}", memory.git.current_branch()?);
    Ok(())
}

#[instrument(skip(config), fields(branch = %branch, character = %character_name))]
async fn cmd_chat(config: &Config, branch: &str, character_name: &str) -> Result<()> {
    validation::validate_branch(branch)?;
    info!("starting GUAC chat");
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    if !memory.git.is_repo() {
        return Err(anyhow::anyhow!("GUAC memory repo not initialized. Run 'guac init' first."));
    }

    let mut character = character::Character::load(config, character_name)?;
    let mut kg = kg::KnowledgeGraph::load(&config.kg_dir)?;
    let mut conv = memory.load_conversation(branch)?;

    // Ensure branch exists
    let branches = memory.git.list_branches()?;
    let branch_exists = branches.iter().any(|b| b == branch);
    if !branch_exists {
        if branches.is_empty() {
            // fresh repo, create main
        } else {
            memory.git.branch_create(branch)?;
        }
    }
    if memory.git.current_branch()? != branch {
        memory.git.checkout(branch)?;
    }

    let groq = config
        .groq_api_key
        .as_ref()
        .and_then(|_| groq_client::GroqClient::new(config).ok());

    if groq.is_none() {
        println!("[GUAC] Groq API key not set. Running in local mode (rule-based responses).");
    }

    println!("GUAC chat on branch '{}' with character '{}'", branch, character_name);
    println!("Type '/quit' or '/exit' to leave.");

    loop {
        print!("\nYou: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
            break;
        }
        if input.is_empty() {
            continue;
        }

        memory.append_message(&mut conv, "user", input)?;
        memory::update_recency_scores(&mut conv);

        let intent = classifier::classify(input, groq.as_ref()).await;
        let context = router::gather_context(
            intent,
            input,
            &kg,
            &memory,
            branch,
            config,
        )
        .await?;

        let system_prompt = assembler::assemble_prompt(&character, &context, input);

        let (response_text, delta) = if let Some(client) = &groq {
            match client.chat(&system_prompt, input).await {
                Ok(resp) => (resp.response, resp.delta),
                Err(e) => {
                    warn!(error = %e, "Groq inference failed; falling back to local echo");
                    local_response(input, &context)
                }
            }
        } else {
            local_response(input, &context)
        };

        // Apply memory delta
        let mut changed_paths = Vec::new();
        if !delta.kg_updates.is_empty() {
            info!(count = delta.kg_updates.len(), "applying knowledge graph updates");
            for update in &delta.kg_updates {
                kg.set(&update.path, update.value.clone())?;
            }
            let kg_path = kg.save()?;
            changed_paths.push(kg_path);
        }

        let char_updates = assembler::apply_delta(&delta, &mut character);
        if !char_updates.is_empty() {
            info!(updates = ?char_updates, "applying character memory updates");
            character.save_memory(config)?;
            changed_paths.push(config.character_memory_path(character_name));
        }

        memory.append_message(&mut conv, "assistant", &response_text)?;
        let conv_path = memory.save_conversation(&conv)?;
        changed_paths.push(conv_path);

        let commit_msg = format!(
            "memory: {} | topic: {}",
            summarize_input(input),
            character.memory.current_topic.as_deref().unwrap_or("general")
        );
        memory.commit_memory(&commit_msg, &changed_paths)?;
        info!("committed memory delta");

        println!("\n{}: {}", character.core.name, response_text);
    }

    println!("Goodbye.");
    Ok(())
}

async fn cmd_kg(config: &Config, command: KgCommands) -> Result<()> {
    config.ensure_dirs()?;
    let mut kg = kg::KnowledgeGraph::load(&config.kg_dir)?;

    match command {
        KgCommands::Get { path } => {
            validation::validate_kg_path(&path)?;
            match kg.get(&path) {
                Some(value) => println!("{}", serde_yaml::to_string(value)?.trim()),
                None => println!("(not set)"),
            }
        }
        KgCommands::Set { path, value } => {
            validation::validate_kg_path(&path)?;
            kg.set_str(&path, &value)?;
            let kg_path = kg.save()?;
            let memory = memory::MemoryManager::new(config);
            memory.commit_memory(&format!("kg: set {} = {}", path, value), &[kg_path])?;
            println!("Set {} = {}", path, value);
        }
        KgCommands::List => {
            for path in kg.paths() {
                if let Some(value) = kg.get(&path) {
                    println!("{}: {}", path, kg::describe_value(value));
                }
            }
        }
    }
    Ok(())
}

async fn cmd_recall(config: &Config, query: &str, branch: &str) -> Result<()> {
    validation::validate_branch(branch)?;
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    let conv = memory.load_conversation(branch)?;
    let top = conv.top_scored(&config.score_weights, 10, 0.0);

    println!("Recalling on branch '{}' for: {}", branch, query);
    for msg in top {
        println!(
            "[{:.2}] {} at {}: {}",
            msg.scores.combined(&config.score_weights),
            msg.role,
            msg.timestamp.format("%Y-%m-%d %H:%M"),
            msg.content
        );
    }
    Ok(())
}

async fn cmd_branch(config: &Config, command: BranchCommands) -> Result<()> {
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    if !memory.git.is_repo() {
        return Err(anyhow::anyhow!("GUAC memory repo not initialized. Run 'guac init' first."));
    }

    match command {
        BranchCommands::Create { name } => {
            validation::validate_branch(&name)?;
            memory.git.branch_create(&name)?;
            memory.git.checkout(&name)?;
            // Initialize conversation file for branch
            let mut conv = memory.load_conversation(&name)?;
            conv.branch = name.clone();
            let path = memory.save_conversation(&conv)?;
            memory.commit_memory(&format!("branch: create {}", name), &[path])?;
            println!("Created and switched to branch '{}'", name);
        }
        BranchCommands::Switch { name } => {
            validation::validate_branch(&name)?;
            memory.git.checkout(&name)?;
            println!("Switched to branch '{}'", name);
        }
        BranchCommands::List => {
            let current = memory.git.current_branch()?;
            for b in memory.git.list_branches()? {
                if b == current {
                    println!("* {}", b);
                } else {
                    println!("  {}", b);
                }
            }
        }
        BranchCommands::Current => {
            println!("{}", memory.git.current_branch()?);
        }
    }
    Ok(())
}

async fn cmd_compress(config: &Config, branch: &str) -> Result<()> {
    validation::validate_branch(branch)?;
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    if !memory.git.is_repo() {
        return Err(anyhow::anyhow!("GUAC memory repo not initialized. Run 'guac init' first."));
    }

    let mut conv = memory.load_conversation(branch)?;
    let groq = config
        .groq_api_key
        .as_ref()
        .and_then(|_| groq_client::GroqClient::new(config).ok());

    let count = compression::compress_branch(&mut conv, config, groq.as_ref()).await?;
    if count == 0 {
        println!("Nothing to compress on branch '{}' (<= {} messages)", branch, config.compression_threshold);
    } else {
        let conv_path = memory.save_conversation(&conv)?;
        let summary_path = config.summary_path(branch, 1);
        memory.commit_memory(
            &format!("compress: summarized {} messages on {}", count, branch),
            &[conv_path, summary_path],
        )?;
        println!("Compressed {} messages on branch '{}'", count, branch);
    }
    Ok(())
}

async fn cmd_fs(config: &Config, command: FsCommands) -> Result<()> {
    config.ensure_dirs()?;
    let fs = fs_memory::FsMemory::new(config);
    let memory = memory::MemoryManager::new(config);

    match command {
        FsCommands::Read { path } => {
            validation::validate_address(&path)?;
            let content = fs.read(&path)?;
            println!("{}", content);
        }
        FsCommands::Write { path, content } => {
            validation::validate_address(&path)?;
            let written = fs.write(&path, &content)?;
            memory.commit_memory(
                &format!("fs: write {}", path),
                &[written],
            )?;
            println!("Wrote {}", path);
        }
        FsCommands::List { path } => {
            validation::validate_address(&path)?;
            for entry in fs.list(&path)? {
                println!("{}", entry);
            }
        }
    }
    Ok(())
}

async fn cmd_status(config: &Config) -> Result<()> {
    config.ensure_dirs()?;
    let memory = memory::MemoryManager::new(config);
    if !memory.git.is_repo() {
        println!("GUAC memory repo not initialized.");
        return Ok(());
    }

    println!("Memory directory: {}", config.memory_dir.display());
    println!("Current branch: {}", memory.git.current_branch()?);
    println!("Recent commits:");
    println!("{}", memory.git.log_oneline(10)?);
    Ok(())
}

fn local_response(input: &str, context: &router::ContextQuery) -> (String, groq_client::MemoryDelta) {
    // Simple deterministic fallback
    let mut parts = Vec::new();

    if !context.kg_facts.is_empty() {
        parts.push("Based on the knowledge graph:".to_string());
        parts.extend(context.kg_facts.iter().cloned());
    }

    if !context.recent_messages.is_empty() {
        parts.push("Recent context is available.".to_string());
    }

    parts.push(format!(
        "(Groq not available. Echo:) {}",
        input
    ));

    (
        parts.join("\n"),
        groq_client::MemoryDelta::default(),
    )
}

fn summarize_input(input: &str) -> String {
    if input.len() > 50 {
        format!("{}...", &input[..50])
    } else {
        input.to_string()
    }
}
