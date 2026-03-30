mod get;
mod inspect;
mod list;
mod recall;
mod relation_ops;
pub mod remember;
mod stats;
mod status_ops;
mod tag_ops;
mod update;

pub use get::execute_get;
pub use inspect::{execute_inspect, InspectResult};
pub use list::execute_list;
pub use recall::execute_recall;
pub use relation_ops::execute_relate;
pub use remember::execute_remember;
pub use stats::execute_stats;
pub use status_ops::{
    execute_archive, execute_forget, execute_invalidate, execute_pin, execute_supersede,
};
pub use tag_ops::{execute_add_tags, execute_remove_tags};
pub use update::execute_update;

use crate::store::models::{Memory, MemoryStats, MemoryWithTags};

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum CommandResult {
    MemoryCreated(MemoryWithTags),
    MemoryRetrieved(MemoryWithTags),
    MemoryList(Vec<Memory>),
    MemoryUpdated(MemoryWithTags),
    MemoryInspected(InspectResult),
    StatusChanged { id: String, new_status: String },
    TypeChanged { id: String, new_type: String },
    Deleted { id: String },
    TagsUpdated { id: String, tags: Vec<String> },
    RelationAdded { from_id: String, to_id: String, relation_type: String },
    Stats(MemoryStats),
}
