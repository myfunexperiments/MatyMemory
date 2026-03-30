use rusqlite::types::ToSql;

use super::models::SearchFilters;

// ---------------------------------------------------------------------------
// Memories
// ---------------------------------------------------------------------------

pub const INSERT_MEMORY: &str = "\
    INSERT INTO memories \
    (id, content, memory_type, base_importance, confidence, status, \
     valid_from, valid_to, supersedes_id, created_at, updated_at, \
     last_accessed, access_count) \
    VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7, NULL, ?8, ?8, ?8, 0)";

pub const SELECT_MEMORY_BY_ID: &str = "\
    SELECT id, content, memory_type, base_importance, confidence, status, \
           valid_from, valid_to, supersedes_id, created_at, updated_at, \
           last_accessed, access_count \
    FROM memories WHERE id = ?1";

pub const UPDATE_MEMORY_ACCESS: &str = "\
    UPDATE memories SET last_accessed = ?2, access_count = access_count + 1 \
    WHERE id = ?1";

pub const SELECT_MEMORY_STATUS: &str = "\
    SELECT status FROM memories WHERE id = ?1";

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

pub const INSERT_PROVENANCE: &str = "\
    INSERT INTO provenance (memory_id, actor, session_id, model_id, write_reason) \
    VALUES (?1, ?2, ?3, ?4, ?5)";

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

pub const INSERT_TAG: &str = "\
    INSERT OR IGNORE INTO tags (memory_id, tag) VALUES (?1, ?2)";

pub const DELETE_TAG: &str = "\
    DELETE FROM tags WHERE memory_id = ?1 AND tag = ?2";

pub const SELECT_TAGS_BY_MEMORY: &str = "\
    SELECT tag FROM tags WHERE memory_id = ?1 ORDER BY tag";

// ---------------------------------------------------------------------------
// Relations
// ---------------------------------------------------------------------------

pub const INSERT_RELATION: &str = "\
    INSERT OR IGNORE INTO relations (from_id, to_id, relation_type) \
    VALUES (?1, ?2, ?3)";

pub const DELETE_RELATION: &str = "\
    DELETE FROM relations WHERE from_id = ?1 AND to_id = ?2 AND relation_type = ?3";

pub const SELECT_RELATIONS: &str = "\
    SELECT from_id, to_id, relation_type FROM relations \
    WHERE from_id = ?1 OR to_id = ?1";

// ---------------------------------------------------------------------------
// Retrieval log
// ---------------------------------------------------------------------------

pub const INSERT_RETRIEVAL_LOG: &str = "\
    INSERT INTO retrieval_log (query, memory_id, score, explanation, timestamp) \
    VALUES (?1, ?2, ?3, ?4, ?5)";

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

pub const SELECT_STATS: &str = "\
    SELECT memory_type, status, COUNT(*) as cnt \
    FROM memories GROUP BY memory_type, status";

// ---------------------------------------------------------------------------
// Search helpers
// ---------------------------------------------------------------------------

pub fn escape_like(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '%' => out.push_str("\\%"),
            '_' => out.push_str("\\_"),
            other => out.push(other),
        }
    }
    out
}

pub fn build_search_query(filters: &SearchFilters) -> (String, Vec<Box<dyn ToSql>>) {
    let mut sql = String::from(
        "SELECT id, content, memory_type, base_importance, confidence, status, \
         valid_from, valid_to, supersedes_id, created_at, updated_at, \
         last_accessed, access_count FROM memories",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();
    let mut idx = 1u32;

    if let Some(ref text) = filters.text {
        let escaped = escape_like(text);
        conditions.push(format!("content LIKE '%' || ?{idx} || '%' ESCAPE '\\'"));
        params.push(Box::new(escaped));
        idx += 1;
    }

    if let Some(ref mt) = filters.memory_type {
        conditions.push(format!("memory_type = ?{idx}"));
        params.push(Box::new(mt.to_string()));
        idx += 1;
    }

    if let Some(ref st) = filters.status {
        conditions.push(format!("status = ?{idx}"));
        params.push(Box::new(st.to_string()));
        idx += 1;
    }

    if let Some(ref tag) = filters.tag {
        conditions.push(format!(
            "id IN (SELECT memory_id FROM tags WHERE tag = ?{idx})"
        ));
        params.push(Box::new(tag.clone()));
        idx += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC");

    sql.push_str(&format!(
        " LIMIT ?{} OFFSET ?{}",
        idx,
        idx + 1
    ));
    params.push(Box::new(filters.limit_or_default()));
    params.push(Box::new(filters.offset_or_default()));

    (sql, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_like_special_chars() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
        assert_eq!(escape_like("normal"), "normal");
    }

    #[test]
    fn build_query_no_filters() {
        let filters = SearchFilters::default();
        let (sql, params) = build_search_query(&filters);
        assert!(sql.contains("FROM memories"));
        assert!(!sql.contains("WHERE"));
        assert_eq!(params.len(), 2); // limit + offset
    }

    #[test]
    fn build_query_with_text_and_type() {
        let filters = SearchFilters {
            text: Some("auth".to_string()),
            memory_type: Some(super::super::models::MemoryType::Semantic),
            ..Default::default()
        };
        let (sql, params) = build_search_query(&filters);
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("content LIKE"));
        assert!(sql.contains("memory_type ="));
        assert_eq!(params.len(), 4); // text + type + limit + offset
    }
}
