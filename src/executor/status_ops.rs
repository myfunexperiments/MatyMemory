use crate::store::error::Result;
use crate::store::models::MemoryType;
use crate::store::Store;

use super::CommandResult;

pub fn execute_archive(store: &Store, id: &str) -> Result<CommandResult> {
    let m = store.archive_memory(id)?;
    Ok(CommandResult::StatusChanged {
        id: m.id,
        new_status: m.status.to_string(),
    })
}

pub fn execute_invalidate(store: &Store, id: &str) -> Result<CommandResult> {
    let m = store.invalidate_memory(id)?;
    Ok(CommandResult::StatusChanged {
        id: m.id,
        new_status: m.status.to_string(),
    })
}

pub fn execute_supersede(
    store: &Store,
    old_id: &str,
    new_id: &str,
) -> Result<CommandResult> {
    store.supersede_memory(old_id, new_id)?;
    Ok(CommandResult::StatusChanged {
        id: old_id.to_string(),
        new_status: "superseded".to_string(),
    })
}

pub fn execute_forget(store: &Store, id: &str) -> Result<CommandResult> {
    store.delete_memory(id)?;
    Ok(CommandResult::Deleted {
        id: id.to_string(),
    })
}

pub fn execute_pin(store: &Store, id: &str) -> Result<CommandResult> {
    let m = store.change_type(id, MemoryType::Pinned)?;
    Ok(CommandResult::TypeChanged {
        id: m.id,
        new_type: m.memory_type.to_string(),
    })
}
