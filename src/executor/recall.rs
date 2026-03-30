use crate::store::error::Result;
use crate::store::models::SearchFilters;
use crate::store::Store;

use super::CommandResult;

pub fn execute_recall(store: &Store, filters: SearchFilters) -> Result<CommandResult> {
    let results = store.search(&filters)?;
    Ok(CommandResult::MemoryList(results))
}
