use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "guac")]
#[command(about = "Git + Groq Augmented Cognition")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new GUAC memory repository
    Init,

    /// Start an interactive chat session
    Chat {
        #[arg(short, long, default_value = "main")]
        branch: String,
        #[arg(short, long, default_value = "default")]
        character: String,
    },

    /// Knowledge graph operations
    Kg {
        #[command(subcommand)]
        command: KgCommands,
    },

    /// Recall information from conversation memory
    Recall {
        query: String,
        #[arg(short, long, default_value = "main")]
        branch: String,
    },

    /// Branch operations
    Branch {
        #[command(subcommand)]
        command: BranchCommands,
    },

    /// Compress old conversation memory
    Compress {
        #[arg(short, long, default_value = "main")]
        branch: String,
    },

    /// Show memory repository status
    Status,

    /// Filesystem memory operations
    Fs {
        #[command(subcommand)]
        command: FsCommands,
    },
}

#[derive(Subcommand)]
pub enum FsCommands {
    /// Read a memory path
    Read { path: String },
    /// Write content to a memory path
    Write { path: String, content: String },
    /// List a memory directory
    List { path: String },
}

#[derive(Subcommand)]
pub enum KgCommands {
    /// Get a value from the knowledge graph
    Get {
        path: String,
    },
    /// Set a value in the knowledge graph
    Set {
        path: String,
        value: String,
    },
    /// List knowledge graph paths
    List,
}

#[derive(Subcommand)]
pub enum BranchCommands {
    /// Create and switch to a new branch
    Create { name: String },
    /// Switch to an existing branch
    Switch { name: String },
    /// List branches
    List,
    /// Show current branch
    Current,
}
