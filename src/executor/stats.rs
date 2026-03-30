use crate::store::error::Result;
use crate::store::Store;

use super::CommandResult;

pub fn execute_stats(store: &Store) -> Result<CommandResult> {
    let stats = store.get_stats()?;
    Ok(CommandResult::Stats(stats))
}
