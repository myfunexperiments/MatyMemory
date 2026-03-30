use super::error::Result;
use super::models::RetrievalLogEntry;
use super::queries;
use super::Store;

impl Store {
    pub fn log_retrieval(&self, entry: &RetrievalLogEntry) -> Result<()> {
        self.conn.execute(
            queries::INSERT_RETRIEVAL_LOG,
            rusqlite::params![
                entry.query,
                entry.memory_id,
                entry.score,
                entry.explanation,
                entry.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}
