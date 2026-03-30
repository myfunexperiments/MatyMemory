use crate::store::error::Result;
use crate::store::models::{MemoryWithTags, Provenance, Relation};
use crate::store::Store;

use super::CommandResult;

#[derive(Debug, serde::Serialize)]
pub struct InspectResult {
    pub memory: MemoryWithTags,
    pub provenance: Provenance,
    pub relations: Vec<Relation>,
}

pub fn execute_inspect(store: &Store, id: &str) -> Result<CommandResult> {
    let memory = store.get_memory(id)?;
    let provenance = store.get_provenance(id)?;
    let relations = store.get_relations(id)?;
    Ok(CommandResult::MemoryInspected(InspectResult {
        memory,
        provenance,
        relations,
    }))
}
