use rusqlite::Connection;

use super::error::Result;

const CURRENT_VERSION: u32 = 1;

const MIGRATION_1: &str = r#"
CREATE TABLE memories (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    memory_type TEXT NOT NULL
        CHECK(memory_type IN ('pinned','semantic','episodic','procedural','session')),
    base_importance REAL NOT NULL
        CHECK(base_importance >= 0.0 AND base_importance <= 1.0),
    confidence REAL NOT NULL
        CHECK(confidence >= 0.0 AND confidence <= 1.0),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK(status IN ('active','archived','invalidated','superseded')),
    valid_from TEXT,
    valid_to TEXT,
    supersedes_id TEXT REFERENCES memories(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE provenance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id TEXT NOT NULL UNIQUE REFERENCES memories(id) ON DELETE CASCADE,
    actor TEXT NOT NULL,
    session_id TEXT,
    model_id TEXT,
    write_reason TEXT
);

CREATE TABLE tags (
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (memory_id, tag)
);

CREATE TABLE relations (
    from_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    to_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, relation_type)
);

CREATE TABLE retrieval_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    explanation TEXT,
    timestamp TEXT NOT NULL
);

CREATE TABLE scopes (
    memory_id TEXT PRIMARY KEY NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    owner TEXT NOT NULL,
    project TEXT,
    read_scope TEXT NOT NULL DEFAULT 'private',
    write_scope TEXT NOT NULL DEFAULT 'private'
);

CREATE INDEX idx_memories_status ON memories(status);
CREATE INDEX idx_memories_type ON memories(memory_type);
CREATE INDEX idx_memories_created ON memories(created_at);
CREATE INDEX idx_tags_tag ON tags(tag);
CREATE INDEX idx_relations_to ON relations(to_id);
"#;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(MIGRATION_1)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
    }

    debug_assert_eq!(
        conn.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        CURRENT_VERSION
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn
    }

    #[test]
    fn migration_sets_version() {
        let conn = fresh_conn();
        run_migrations(&conn).unwrap();
        let v: u32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, CURRENT_VERSION);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = fresh_conn();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // second run is a no-op
    }

    #[test]
    fn rejects_invalid_memory_type() {
        let conn = fresh_conn();
        run_migrations(&conn).unwrap();
        let err = conn.execute(
            "INSERT INTO memories (id, content, memory_type, base_importance, confidence, \
             status, created_at, updated_at, last_accessed, access_count) \
             VALUES ('x','test','bogus',0.5,0.5,'active','t','t','t',0)",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn rejects_importance_out_of_range() {
        let conn = fresh_conn();
        run_migrations(&conn).unwrap();
        let err = conn.execute(
            "INSERT INTO memories (id, content, memory_type, base_importance, confidence, \
             status, created_at, updated_at, last_accessed, access_count) \
             VALUES ('x','test','semantic',1.5,0.5,'active','t','t','t',0)",
            [],
        );
        assert!(err.is_err());
    }
}
