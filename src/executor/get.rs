use crate::store::error::Result;
use crate::store::Store;

use super::CommandResult;

pub fn execute_get(store: &Store, id: &str) -> Result<CommandResult> {
    let memory = store.get_memory(id)?;
    Ok(CommandResult::MemoryRetrieved(memory))
}
