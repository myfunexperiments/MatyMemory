use crate::store::error::Result;
use crate::store::Store;

use super::CommandResult;

pub fn execute_relate(
    store: &Store,
    from_id: &str,
    to_id: &str,
    relation_type: &str,
) -> Result<CommandResult> {
    store.add_relation(from_id, to_id, relation_type)?;
    Ok(CommandResult::RelationAdded {
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        relation_type: relation_type.to_string(),
    })
}
