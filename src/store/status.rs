use chrono::Utc;

use super::error::{MatyError, Result};
use super::models::{Memory, MemoryStatus};
use super::Store;

impl Store {
    pub fn archive_memory(&self, id: &str) -> Result<Memory> {
        self.transition_status(id, MemoryStatus::Active, MemoryStatus::Archived)
    }

    pub fn invalidate_memory(&self, id: &str) -> Result<Memory> {
        self.transition_status(id, MemoryStatus::Active, MemoryStatus::Invalidated)
    }

    pub fn supersede_memory(&self, old_id: &str, new_id: &str) -> Result<()> {
        if old_id == new_id {
            return Err(MatyError::InvalidInput(
                "Cannot supersede a memory with itself".to_string(),
            ));
        }

        // Validate both exist before mutating
        self.get_status_str(old_id)?;
        self.get_status_str(new_id)?;

        let tx = self.conn.unchecked_transaction()?;
        let now = now_rfc3339();

        // Atomic conditional update on old — only if currently active
        let old_rows = tx.execute(
            "UPDATE memories SET status = 'superseded', updated_at = ?2 \
             WHERE id = ?1 AND status = 'active'",
            rusqlite::params![old_id, now],
        )?;
        if old_rows == 0 {
            return Err(self.status_transition_error(old_id, "superseded")?);
        }

        // Atomic conditional update on new — only if currently active and not already superseding
        let new_rows = tx.execute(
            "UPDATE memories SET supersedes_id = ?2, updated_at = ?3 \
             WHERE id = ?1 AND status = 'active' AND supersedes_id IS NULL",
            rusqlite::params![new_id, old_id, now],
        )?;
        if new_rows == 0 {
            return Err(MatyError::InvalidInput(format!(
                "Memory {new_id} is not active or already supersedes another memory"
            )));
        }

        tx.commit()?;
        Ok(())
    }

    fn transition_status(
        &self,
        id: &str,
        expected_from: MemoryStatus,
        to: MemoryStatus,
    ) -> Result<Memory> {
        let now = now_rfc3339();
        let rows = self.conn.execute(
            "UPDATE memories SET status = ?2, updated_at = ?3 \
             WHERE id = ?1 AND status = ?4",
            rusqlite::params![id, to.to_string(), now, expected_from.to_string()],
        )?;

        if rows == 0 {
            return Err(self.status_transition_error(id, &to.to_string())?);
        }

        self.row_to_memory(id)
    }

    fn get_status_str(&self, id: &str) -> Result<String> {
        match self
            .conn
            .query_row("SELECT status FROM memories WHERE id = ?1", [id], |row| {
                row.get(0)
            }) {
            Ok(s) => Ok(s),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(MatyError::NotFound(id.to_string()))
            }
            Err(e) => Err(MatyError::Db(e)),
        }
    }

    fn status_transition_error(&self, id: &str, target: &str) -> Result<MatyError> {
        let current = self.get_status_str(id)?;
        Ok(MatyError::InvalidTransition {
            from: current,
            to: target.to_string(),
        })
    }
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
