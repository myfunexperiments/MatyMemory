use crate::executor::CommandResult;

pub fn format_result(result: &CommandResult, json: bool, quiet: bool) -> String {
    if json {
        return serde_json::to_string_pretty(result).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"));
    }

    match result {
        CommandResult::MemoryCreated(mwt) => {
            if quiet {
                return mwt.memory.id.clone();
            }
            format!(
                "Created memory {}\n  type: {}\n  tags: [{}]",
                mwt.memory.id,
                mwt.memory.memory_type,
                mwt.tags.join(", ")
            )
        }
        CommandResult::MemoryRetrieved(mwt) => {
            if quiet {
                return mwt.memory.id.clone();
            }
            format!(
                "[{}] ({}, {})\n  {}\n  tags: [{}]",
                mwt.memory.id,
                mwt.memory.memory_type,
                mwt.memory.status,
                mwt.memory.content,
                mwt.tags.join(", ")
            )
        }
        CommandResult::MemoryList(memories) => {
            if quiet {
                return memories.iter().map(|m| m.id.as_str()).collect::<Vec<_>>().join("\n");
            }
            if memories.is_empty() {
                return "No memories found.".to_string();
            }
            memories
                .iter()
                .map(|m| {
                    format!(
                        "  {} ({}, {}) {}",
                        &m.id[..8],
                        m.memory_type,
                        m.status,
                        truncate(&m.content, 60)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        CommandResult::MemoryUpdated(mwt) => {
            if quiet {
                return mwt.memory.id.clone();
            }
            format!("Updated memory {}", mwt.memory.id)
        }
        CommandResult::MemoryInspected(inspect) => {
            if quiet {
                return inspect.memory.memory.id.clone();
            }
            let m = &inspect.memory.memory;
            let p = &inspect.provenance;
            let mut out = format!(
                "Memory {}\n  content: {}\n  type: {}\n  status: {}\n  importance: {}\n  confidence: {}\n  created: {}\n  tags: [{}]\n  actor: {}",
                m.id, m.content, m.memory_type, m.status,
                m.base_importance, m.confidence, m.created_at,
                inspect.memory.tags.join(", "),
                p.actor
            );
            if let Some(ref reason) = p.write_reason {
                out.push_str(&format!("\n  reason: {reason}"));
            }
            if !inspect.relations.is_empty() {
                out.push_str("\n  relations:");
                for r in &inspect.relations {
                    out.push_str(&format!("\n    {} -> {} ({})", r.from_id, r.to_id, r.relation_type));
                }
            }
            out
        }
        CommandResult::StatusChanged { id, new_status } => {
            if quiet { return "ok".to_string(); }
            format!("Memory {id} status changed to {new_status}")
        }
        CommandResult::TypeChanged { id, new_type } => {
            if quiet { return "ok".to_string(); }
            format!("Memory {id} type changed to {new_type}")
        }
        CommandResult::Deleted { id } => {
            if quiet { return "ok".to_string(); }
            format!("Deleted memory {id}")
        }
        CommandResult::TagsUpdated { id, tags } => {
            if quiet { return "ok".to_string(); }
            format!("Memory {id} tags: [{}]", tags.join(", "))
        }
        CommandResult::RelationAdded { from_id, to_id, relation_type } => {
            if quiet { return "ok".to_string(); }
            format!("Added relation: {from_id} -> {to_id} ({relation_type})")
        }
        CommandResult::Stats(stats) => {
            if quiet {
                return stats.total.to_string();
            }
            let mut out = format!("Total memories: {}", stats.total);
            if !stats.by_type.is_empty() {
                out.push_str("\n  By type:");
                for (k, v) in &stats.by_type {
                    out.push_str(&format!("\n    {k}: {v}"));
                }
            }
            if !stats.by_status.is_empty() {
                out.push_str("\n  By status:");
                for (k, v) in &stats.by_status {
                    out.push_str(&format!("\n    {k}: {v}"));
                }
            }
            out
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{truncated}...")
    }
}
