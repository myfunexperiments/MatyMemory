use crate::store::error::Result;
use crate::store::models::{CreateMemoryRequest, MemoryType};
use crate::store::Store;

use super::CommandResult;

pub struct RememberArgs {
    pub content: String,
    pub memory_type: MemoryType,
    pub tags: Vec<String>,
    pub importance: f64,
    pub confidence: f64,
    pub actor: String,
    pub session_id: Option<String>,
    pub model_id: Option<String>,
    pub write_reason: Option<String>,
}

pub fn execute_remember(store: &Store, args: RememberArgs) -> Result<CommandResult> {
    let req = CreateMemoryRequest {
        content: args.content,
        memory_type: args.memory_type,
        tags: args.tags,
        base_importance: args.importance,
        confidence: args.confidence,
        valid_from: None,
        valid_to: None,
        actor: args.actor,
        session_id: args.session_id,
        model_id: args.model_id,
        write_reason: args.write_reason,
    };
    let created = store.create_memory(req)?;
    Ok(CommandResult::MemoryCreated(created))
}
