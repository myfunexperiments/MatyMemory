use crate::store::error::Result;
use crate::store::models::{MemoryUpdate, Update};
use crate::store::Store;

use super::CommandResult;

pub fn execute_update(
    store: &Store,
    id: &str,
    content: Option<&str>,
    importance: Option<f64>,
    confidence: Option<f64>,
) -> Result<CommandResult> {
    let updates = MemoryUpdate {
        content: match content {
            Some(c) => Update::Set(c.to_string()),
            None => Update::Unchanged,
        },
        base_importance: match importance {
            Some(v) => Update::Set(v),
            None => Update::Unchanged,
        },
        confidence: match confidence {
            Some(v) => Update::Set(v),
            None => Update::Unchanged,
        },
        ..Default::default()
    };
    let updated = store.update_memory(id, updates)?;
    Ok(CommandResult::MemoryUpdated(updated))
}
