use std::collections::HashMap;

use super::error::Result;
use super::queries;
use super::Store;

impl Store {
    pub fn get_tags(&self, id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(queries::SELECT_TAGS_BY_MEMORY)?;
        let rows = stmt.query_map([id], |row| row.get(0))?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    pub fn add_tags(&self, id: &str, tags: &[&str]) -> Result<()> {
        // Verify memory exists
        self.row_to_memory(id)?;
        for tag in tags {
            self.conn
                .execute(queries::INSERT_TAG, rusqlite::params![id, tag])?;
        }
        Ok(())
    }

    pub fn remove_tags(&self, id: &str, tags: &[&str]) -> Result<()> {
        for tag in tags {
            self.conn
                .execute(queries::DELETE_TAG, rusqlite::params![id, tag])?;
        }
        Ok(())
    }

    pub fn batch_get_tags(&self, ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT memory_id, tag FROM tags WHERE memory_id IN ({}) ORDER BY tag",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let (mid, tag) = row?;
            map.entry(mid).or_default().push(tag);
        }
        Ok(map)
    }
}
