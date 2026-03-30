pub mod output;
pub mod parse;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "maty", version, about = "LLM memory system")]
pub struct Cli {
    /// Start interactive REPL mode
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Quiet mode (IDs only)
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Database file path
    #[arg(long, global = true)]
    pub db: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Store a new memory
    Remember {
        /// Memory content
        content: String,
        /// Memory type (semantic, episodic, procedural, session, pinned)
        #[arg(short = 't', long = "type")]
        memory_type: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Importance score (0.0-1.0)
        #[arg(long)]
        importance: Option<f64>,
        /// Confidence score (0.0-1.0)
        #[arg(long)]
        confidence: Option<f64>,
        /// Actor identity
        #[arg(long, default_value = "cli")]
        actor: String,
        /// Session ID
        #[arg(long)]
        session: Option<String>,
        /// Model ID
        #[arg(long)]
        model: Option<String>,
        /// Write reason
        #[arg(long)]
        reason: Option<String>,
    },
    /// Search memories by text
    Recall {
        /// Search query text
        query: Option<String>,
        /// Filter by type
        #[arg(short = 't', long = "type")]
        memory_type: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Max results
        #[arg(long)]
        limit: Option<u32>,
        /// Offset
        #[arg(long)]
        offset: Option<u32>,
    },
    /// Get a memory by ID
    Get {
        /// Memory ID
        id: String,
    },
    /// Inspect a memory (full details)
    Inspect {
        /// Memory ID
        id: String,
    },
    /// Update a memory
    Update {
        /// Memory ID
        id: String,
        /// New content
        #[arg(long)]
        content: Option<String>,
        /// New importance
        #[arg(long)]
        importance: Option<f64>,
        /// New confidence
        #[arg(long)]
        confidence: Option<f64>,
    },
    /// Archive a memory
    Archive {
        /// Memory ID
        id: String,
    },
    /// Invalidate a memory
    Invalidate {
        /// Memory ID
        id: String,
    },
    /// Supersede a memory with another
    Supersede {
        /// Old memory ID
        old_id: String,
        /// New memory ID
        new_id: String,
    },
    /// Delete a memory permanently
    Forget {
        /// Memory ID
        id: String,
    },
    /// Pin a memory
    Pin {
        /// Memory ID
        id: String,
    },
    /// Add tags to a memory
    Tag {
        /// Memory ID
        id: String,
        /// Comma-separated tags to add
        tags: String,
    },
    /// Remove tags from a memory
    Untag {
        /// Memory ID
        id: String,
        /// Comma-separated tags to remove
        tags: String,
    },
    /// Add a relation between two memories
    Relate {
        /// Source memory ID
        from_id: String,
        /// Target memory ID
        to_id: String,
        /// Relation type
        #[arg(long, default_value = "related_to")]
        relation_type: String,
    },
    /// List memories (no text search)
    List {
        /// Filter by type
        #[arg(short = 't', long = "type")]
        memory_type: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Max results
        #[arg(long)]
        limit: Option<u32>,
        /// Offset
        #[arg(long)]
        offset: Option<u32>,
    },
    /// Show memory statistics
    Stats,
}
