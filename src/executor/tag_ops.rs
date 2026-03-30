use crate::store::error::Result;
use crate::store::Store;

use super::CommandResult;

pub fn execute_add_tags(store: &Store, id: &str, tags: &[&str]) -> Result<CommandResult> {
    store.add_tags(id, tags)?;
    let all_tags = store.get_tags(id)?;
    Ok(CommandResult::TagsUpdated {
        id: id.to_string(),
        tags: all_tags,
    })
}

pub fn execute_remove_tags(store: &Store, id: &str, tags: &[&str]) -> Result<CommandResult> {
    store.remove_tags(id, tags)?;
    let all_tags = store.get_tags(id)?;
    Ok(CommandResult::TagsUpdated {
        id: id.to_string(),
        tags: all_tags,
    })
}
