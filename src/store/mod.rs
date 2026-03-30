pub mod error;
pub mod models;
mod queries;
mod schema;

mod memory;
mod relations;
mod retrieval;
mod status;
mod tags;

use std::path::Path;

use error::Result;
use rusqlite::Connection;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn, true)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn, false)
    }

    fn init(conn: Connection, is_file: bool) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
        if is_file {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }
        schema::run_migrations(&conn)?;
        Ok(Self { conn })
    }
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use super::*;

    fn store() -> Store {
        Store::open_in_memory().unwrap()
    }

    fn sample_request(content: &str) -> CreateMemoryRequest {
        CreateMemoryRequest {
            content: content.to_string(),
            memory_type: MemoryType::Semantic,
            tags: vec!["test".to_string()],
            base_importance: 0.5,
            confidence: 0.8,
            valid_from: None,
            valid_to: None,
            actor: "test-user".to_string(),
            session_id: None,
            model_id: None,
            write_reason: Some("unit test".to_string()),
        }
    }

    // -- CRUD tests --

    #[test]
    fn create_and_get_roundtrip() {
        let s = store();
        let created = s.create_memory(sample_request("Auth uses JWT")).unwrap();
        assert_eq!(created.memory.content, "Auth uses JWT");
        assert_eq!(created.memory.memory_type, MemoryType::Semantic);
        assert_eq!(created.memory.status, MemoryStatus::Active);
        assert_eq!(created.tags, vec!["test"]);

        let fetched = s.get_memory(&created.memory.id).unwrap();
        assert_eq!(fetched.memory.content, "Auth uses JWT");
        assert_eq!(fetched.memory.access_count, 1);
    }

    #[test]
    fn get_memory_not_found() {
        let s = store();
        let result = s.get_memory("nonexistent");
        assert!(matches!(result, Err(super::error::MatyError::NotFound(_))));
    }

    #[test]
    fn update_partial_fields() {
        let s = store();
        let created = s.create_memory(sample_request("original")).unwrap();

        let updates = MemoryUpdate {
            content: Update::Set("updated".to_string()),
            confidence: Update::Set(0.95),
            ..Default::default()
        };
        let updated = s.update_memory(&created.memory.id, updates).unwrap();
        assert_eq!(updated.memory.content, "updated");
        assert_eq!(updated.memory.confidence, 0.95);
        assert_eq!(updated.memory.base_importance, 0.5); // unchanged
    }

    #[test]
    fn update_null_clears_field() {
        let s = store();
        let req = CreateMemoryRequest {
            valid_from: Some(chrono::Utc::now()),
            ..sample_request("with date")
        };
        let created = s.create_memory(req).unwrap();
        assert!(created.memory.valid_from.is_some());

        let updates = MemoryUpdate {
            valid_from: Update::Null,
            ..Default::default()
        };
        let updated = s.update_memory(&created.memory.id, updates).unwrap();
        assert!(updated.memory.valid_from.is_none());
    }

    #[test]
    fn update_unchanged_is_noop() {
        let s = store();
        let created = s.create_memory(sample_request("noop")).unwrap();
        let updates = MemoryUpdate::default();
        let result = s.update_memory(&created.memory.id, updates).unwrap();
        assert_eq!(result.memory.content, "noop");
    }

    #[test]
    fn access_count_increments() {
        let s = store();
        let created = s.create_memory(sample_request("counter")).unwrap();
        assert_eq!(created.memory.access_count, 0);

        s.get_memory(&created.memory.id).unwrap();
        let m = s.get_memory(&created.memory.id).unwrap();
        assert_eq!(m.memory.access_count, 2);
    }

    // -- Status tests --

    #[test]
    fn archive_active_memory() {
        let s = store();
        let created = s.create_memory(sample_request("to archive")).unwrap();
        let archived = s.archive_memory(&created.memory.id).unwrap();
        assert_eq!(archived.status, MemoryStatus::Archived);
    }

    #[test]
    fn invalidate_active_memory() {
        let s = store();
        let created = s.create_memory(sample_request("to invalidate")).unwrap();
        let inv = s.invalidate_memory(&created.memory.id).unwrap();
        assert_eq!(inv.status, MemoryStatus::Invalidated);
    }

    #[test]
    fn supersede_memory_chain() {
        let s = store();
        let old = s.create_memory(sample_request("old fact")).unwrap();
        let new = s.create_memory(sample_request("new fact")).unwrap();
        s.supersede_memory(&old.memory.id, &new.memory.id).unwrap();

        let old_m = s.row_to_memory(&old.memory.id).unwrap();
        assert_eq!(old_m.status, MemoryStatus::Superseded);

        let new_m = s.row_to_memory(&new.memory.id).unwrap();
        assert_eq!(new_m.supersedes_id.as_deref(), Some(old.memory.id.as_str()));
    }

    #[test]
    fn reject_archive_non_active() {
        let s = store();
        let created = s.create_memory(sample_request("test")).unwrap();
        s.archive_memory(&created.memory.id).unwrap();
        let err = s.archive_memory(&created.memory.id);
        assert!(matches!(
            err,
            Err(super::error::MatyError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn reject_supersede_already_superseded() {
        let s = store();
        let a = s.create_memory(sample_request("a")).unwrap();
        let b = s.create_memory(sample_request("b")).unwrap();
        let c = s.create_memory(sample_request("c")).unwrap();
        s.supersede_memory(&a.memory.id, &b.memory.id).unwrap();
        let err = s.supersede_memory(&a.memory.id, &c.memory.id);
        assert!(matches!(
            err,
            Err(super::error::MatyError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn reject_supersede_nonexistent() {
        let s = store();
        let a = s.create_memory(sample_request("a")).unwrap();
        let err = s.supersede_memory(&a.memory.id, "nonexistent");
        assert!(matches!(err, Err(super::error::MatyError::NotFound(_))));
    }

    // -- Tag tests --

    #[test]
    fn tag_operations() {
        let s = store();
        let created = s.create_memory(sample_request("tagged")).unwrap();
        let id = &created.memory.id;

        s.add_tags(id, &["rust", "memory"]).unwrap();
        let tags = s.get_tags(id).unwrap();
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"memory".to_string()));
        assert!(tags.contains(&"test".to_string())); // from creation

        s.remove_tags(id, &["test"]).unwrap();
        let tags = s.get_tags(id).unwrap();
        assert!(!tags.contains(&"test".to_string()));
    }

    #[test]
    fn add_tags_idempotent() {
        let s = store();
        let created = s.create_memory(sample_request("idem")).unwrap();
        s.add_tags(&created.memory.id, &["dup"]).unwrap();
        s.add_tags(&created.memory.id, &["dup"]).unwrap(); // no error
        let tags = s.get_tags(&created.memory.id).unwrap();
        assert_eq!(tags.iter().filter(|t| *t == "dup").count(), 1);
    }

    #[test]
    fn batch_get_tags_groups_correctly() {
        let s = store();
        let a = s.create_memory(sample_request("a")).unwrap();
        let b = s.create_memory(sample_request("b")).unwrap();
        s.add_tags(&a.memory.id, &["alpha"]).unwrap();
        s.add_tags(&b.memory.id, &["beta"]).unwrap();

        let map = s
            .batch_get_tags(&[&a.memory.id, &b.memory.id])
            .unwrap();
        assert!(map[&a.memory.id].contains(&"alpha".to_string()));
        assert!(map[&b.memory.id].contains(&"beta".to_string()));
    }

    // -- Relation tests --

    #[test]
    fn relation_crud() {
        let s = store();
        let a = s.create_memory(sample_request("a")).unwrap();
        let b = s.create_memory(sample_request("b")).unwrap();

        s.add_relation(&a.memory.id, &b.memory.id, "related_to")
            .unwrap();
        let rels = s.get_relations(&a.memory.id).unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].relation_type, "related_to");

        // Idempotent
        s.add_relation(&a.memory.id, &b.memory.id, "related_to")
            .unwrap();
        let rels = s.get_relations(&a.memory.id).unwrap();
        assert_eq!(rels.len(), 1);

        s.remove_relation(&a.memory.id, &b.memory.id, "related_to")
            .unwrap();
        let rels = s.get_relations(&a.memory.id).unwrap();
        assert!(rels.is_empty());
    }

    // -- Search tests --

    #[test]
    fn search_by_text() {
        let s = store();
        s.create_memory(sample_request("Auth uses JWT RS256")).unwrap();
        s.create_memory(sample_request("Prefers Rust over Go")).unwrap();

        let results = s
            .search(&SearchFilters {
                text: Some("JWT".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("JWT"));
    }

    #[test]
    fn search_by_type() {
        let s = store();
        s.create_memory(sample_request("semantic one")).unwrap();
        let mut req = sample_request("episodic one");
        req.memory_type = MemoryType::Episodic;
        s.create_memory(req).unwrap();

        let results = s
            .search(&SearchFilters {
                memory_type: Some(MemoryType::Episodic),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memory_type, MemoryType::Episodic);
    }

    #[test]
    fn search_pagination() {
        let s = store();
        for i in 0..10 {
            s.create_memory(sample_request(&format!("mem {i}"))).unwrap();
        }
        let page1 = s
            .search(&SearchFilters {
                limit: Some(3),
                offset: Some(0),
                ..Default::default()
            })
            .unwrap();
        let page2 = s
            .search(&SearchFilters {
                limit: Some(3),
                offset: Some(3),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page2.len(), 3);
        assert_ne!(page1[0].id, page2[0].id);
    }

    #[test]
    fn search_special_chars() {
        let s = store();
        s.create_memory(sample_request("100% complete")).unwrap();
        s.create_memory(sample_request("normal text")).unwrap();

        let results = s
            .search(&SearchFilters {
                text: Some("100%".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    // -- Stats tests --

    #[test]
    fn stats_empty_db() {
        let s = store();
        let stats = s.get_stats().unwrap();
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn stats_counts() {
        let s = store();
        s.create_memory(sample_request("a")).unwrap();
        s.create_memory(sample_request("b")).unwrap();
        let mut req = sample_request("c");
        req.memory_type = MemoryType::Episodic;
        s.create_memory(req).unwrap();

        let stats = s.get_stats().unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.by_type["semantic"], 2);
        assert_eq!(stats.by_type["episodic"], 1);
        assert_eq!(stats.by_status["active"], 3);
    }

    // -- Delete tests --

    #[test]
    fn delete_memory_removes_it() {
        let s = store();
        let created = s.create_memory(sample_request("to delete")).unwrap();
        s.delete_memory(&created.memory.id).unwrap();
        let err = s.get_memory(&created.memory.id);
        assert!(matches!(err, Err(super::error::MatyError::NotFound(_))));
    }

    #[test]
    fn delete_nonexistent_returns_not_found() {
        let s = store();
        let err = s.delete_memory("nonexistent");
        assert!(matches!(err, Err(super::error::MatyError::NotFound(_))));
    }

    // -- Change type tests --

    #[test]
    fn change_type_updates_memory() {
        let s = store();
        let created = s.create_memory(sample_request("typed")).unwrap();
        assert_eq!(created.memory.memory_type, MemoryType::Semantic);
        let updated = s.change_type(&created.memory.id, MemoryType::Episodic).unwrap();
        assert_eq!(updated.memory_type, MemoryType::Episodic);
    }

    #[test]
    fn change_type_nonexistent_returns_not_found() {
        let s = store();
        let err = s.change_type("nonexistent", MemoryType::Episodic);
        assert!(matches!(err, Err(super::error::MatyError::NotFound(_))));
    }

    // -- Provenance tests --

    #[test]
    fn get_provenance_returns_actor() {
        let s = store();
        let created = s.create_memory(sample_request("with provenance")).unwrap();
        let prov = s.get_provenance(&created.memory.id).unwrap();
        assert_eq!(prov.actor, "test-user");
        assert_eq!(prov.write_reason.as_deref(), Some("unit test"));
    }

    #[test]
    fn get_provenance_nonexistent_returns_not_found() {
        let s = store();
        let err = s.get_provenance("nonexistent");
        assert!(matches!(err, Err(super::error::MatyError::NotFound(_))));
    }

    // -- File persistence test --

    #[test]
    fn file_store_persists() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("sub").join("memory.db");

        let id = {
            let s = Store::open(&db_path).unwrap();
            let created = s.create_memory(sample_request("persistent")).unwrap();
            created.memory.id
        };

        // Re-open and verify data persists
        let s = Store::open(&db_path).unwrap();
        let fetched = s.get_memory(&id).unwrap();
        assert_eq!(fetched.memory.content, "persistent");
    }
}
