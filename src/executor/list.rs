use crate::store::error::Result;
use crate::store::models::SearchFilters;
use crate::store::Store;

use super::CommandResult;

pub fn execute_list(store: &Store, filters: SearchFilters) -> Result<CommandResult> {
    let results = store.list_memories(&filters)?;
    Ok(CommandResult::MemoryList(results))
}
