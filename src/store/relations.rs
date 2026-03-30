use super::error::Result;
use super::models::Relation;
use super::queries;
use super::Store;

impl Store {
    pub fn add_relation(&self, from_id: &str, to_id: &str, relation_type: &str) -> Result<()> {
        // Verify both memories exist
        self.row_to_memory(from_id)?;
        self.row_to_memory(to_id)?;
        self.conn.execute(
            queries::INSERT_RELATION,
            rusqlite::params![from_id, to_id, relation_type],
        )?;
        Ok(())
    }

    pub fn remove_relation(&self, from_id: &str, to_id: &str, relation_type: &str) -> Result<()> {
        self.conn.execute(
            queries::DELETE_RELATION,
            rusqlite::params![from_id, to_id, relation_type],
        )?;
        Ok(())
    }

    pub fn get_relations(&self, id: &str) -> Result<Vec<Relation>> {
        let mut stmt = self.conn.prepare(queries::SELECT_RELATIONS)?;
        let rows = stmt.query_map([id], |row| {
            Ok(Relation {
                from_id: row.get(0)?,
                to_id: row.get(1)?,
                relation_type: row.get(2)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}
