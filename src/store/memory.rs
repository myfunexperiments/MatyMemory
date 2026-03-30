use chrono::Utc;
use uuid::Uuid;

use super::error::{MatyError, Result};
use super::models::{
    CreateMemoryRequest, Memory, MemoryStats, MemoryType, MemoryWithTags, Provenance,
    SearchFilters, Update,
};
use super::queries;
use super::Store;

impl Store {
    pub fn create_memory(&self, req: CreateMemoryRequest) -> Result<MemoryWithTags> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let mt = req.memory_type.to_string();
        let vf = req.valid_from.map(|t| t.to_rfc3339());
        let vt = req.valid_to.map(|t| t.to_rfc3339());

        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            queries::INSERT_MEMORY,
            rusqlite::params![id, req.content, mt, req.base_importance, req.confidence, vf, vt, now],
        )?;

        tx.execute(
            queries::INSERT_PROVENANCE,
            rusqlite::params![id, req.actor, req.session_id, req.model_id, req.write_reason],
        )?;

        for tag in &req.tags {
            tx.execute(queries::INSERT_TAG, rusqlite::params![id, tag])?;
        }

        tx.commit()?;

        let memory = self.row_to_memory(&id)?;
        let tags = self.get_tags(&id)?;
        Ok(MemoryWithTags { memory, tags })
    }

    pub fn get_memory(&self, id: &str) -> Result<MemoryWithTags> {
        // Bump access count first, then read the updated row
        let now = now_rfc3339();
        let rows = self
            .conn
            .execute(queries::UPDATE_MEMORY_ACCESS, rusqlite::params![id, now])?;
        if rows == 0 {
            return Err(MatyError::NotFound(id.to_string()));
        }
        let memory = self.row_to_memory(id)?;
        let tags = self.get_tags(id)?;
        Ok(MemoryWithTags { memory, tags })
    }

    pub fn update_memory(&self, id: &str, updates: super::models::MemoryUpdate) -> Result<MemoryWithTags> {
        // Verify exists
        self.row_to_memory(id)?;

        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1u32;

        collect_update(&mut sets, &mut params, &mut idx, "content", &updates.content);
        collect_update_f64(&mut sets, &mut params, &mut idx, "base_importance", &updates.base_importance);
        collect_update_f64(&mut sets, &mut params, &mut idx, "confidence", &updates.confidence);
        collect_update_dt(&mut sets, &mut params, &mut idx, "valid_from", &updates.valid_from);
        collect_update_dt(&mut sets, &mut params, &mut idx, "valid_to", &updates.valid_to);

        if !sets.is_empty() {
            let now = now_rfc3339();
            sets.push(format!("updated_at = ?{idx}"));
            params.push(Box::new(now));
            idx += 1;

            let sql = format!(
                "UPDATE memories SET {} WHERE id = ?{idx}",
                sets.join(", ")
            );
            params.push(Box::new(id.to_string()));
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            self.conn.execute(&sql, param_refs.as_slice())?;
        }

        let memory = self.row_to_memory(id)?;
        let tags = self.get_tags(id)?;
        Ok(MemoryWithTags { memory, tags })
    }

    pub fn search(&self, filters: &SearchFilters) -> Result<Vec<Memory>> {
        let (sql, params) = queries::build_search_query(filters);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| Ok(parse_memory_row(row)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row??);
        }
        Ok(result)
    }

    pub fn list_memories(&self, filters: &SearchFilters) -> Result<Vec<Memory>> {
        let no_text = SearchFilters {
            text: None,
            memory_type: filters.memory_type,
            status: filters.status,
            tag: filters.tag.clone(),
            limit: filters.limit,
            offset: filters.offset,
        };
        self.search(&no_text)
    }

    pub fn get_stats(&self) -> Result<MemoryStats> {
        let mut stmt = self.conn.prepare(queries::SELECT_STATS)?;
        let mut stats = MemoryStats::default();
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)? as usize,
            ))
        })?;
        for row in rows {
            let (mt, st, count) = row?;
            stats.total += count;
            *stats.by_type.entry(mt).or_insert(0) += count;
            *stats.by_status.entry(st).or_insert(0) += count;
        }
        Ok(stats)
    }

    pub fn delete_memory(&self, id: &str) -> Result<()> {
        // Clear supersedes_id references pointing to this memory first
        self.conn.execute(
            "UPDATE memories SET supersedes_id = NULL WHERE supersedes_id = ?1",
            rusqlite::params![id],
        )?;
        let rows = self
            .conn
            .execute(queries::DELETE_MEMORY, rusqlite::params![id])?;
        if rows == 0 {
            return Err(MatyError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn change_type(&self, id: &str, new_type: MemoryType) -> Result<Memory> {
        let now = now_rfc3339();
        let rows = self.conn.execute(
            queries::UPDATE_MEMORY_TYPE,
            rusqlite::params![id, new_type.to_string(), now],
        )?;
        if rows == 0 {
            return Err(MatyError::NotFound(id.to_string()));
        }
        self.row_to_memory(id)
    }

    pub fn get_provenance(&self, memory_id: &str) -> Result<Provenance> {
        let result = self.conn.query_row(
            queries::SELECT_PROVENANCE,
            [memory_id],
            |row| {
                Ok(Provenance {
                    id: row.get(0)?,
                    memory_id: row.get(1)?,
                    actor: row.get(2)?,
                    session_id: row.get(3)?,
                    model_id: row.get(4)?,
                    write_reason: row.get(5)?,
                })
            },
        );
        match result {
            Ok(p) => Ok(p),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(MatyError::NotFound(memory_id.to_string()))
            }
            Err(e) => Err(MatyError::Db(e)),
        }
    }

    pub(super) fn row_to_memory(&self, id: &str) -> Result<Memory> {
        let result = self.conn.query_row(queries::SELECT_MEMORY_BY_ID, [id], |row| {
            Ok(parse_memory_row(row))
        });
        match result {
            Ok(inner) => inner,
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(MatyError::NotFound(id.to_string())),
            Err(e) => Err(MatyError::Db(e)),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn parse_memory_row(row: &rusqlite::Row<'_>) -> Result<Memory> {
    use chrono::DateTime;
    let parse_ts = |s: String| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.to_utc())
            .map_err(|e| MatyError::InvalidInput(format!("Bad timestamp: {e}")))
    };
    let parse_opt_ts = |s: Option<String>| -> Result<Option<chrono::DateTime<Utc>>> {
        s.map(&parse_ts).transpose()
    };

    Ok(Memory {
        id: row.get(0)?,
        content: row.get(1)?,
        memory_type: row.get::<_, String>(2)?.parse()?,
        base_importance: row.get(3)?,
        confidence: row.get(4)?,
        status: row.get::<_, String>(5)?.parse()?,
        valid_from: parse_opt_ts(row.get(6)?)?,
        valid_to: parse_opt_ts(row.get(7)?)?,
        supersedes_id: row.get(8)?,
        created_at: parse_ts(row.get(9)?)?,
        updated_at: parse_ts(row.get(10)?)?,
        last_accessed: parse_ts(row.get(11)?)?,
        access_count: row.get::<_, i32>(12)? as u32,
    })
}

fn collect_update(
    sets: &mut Vec<String>,
    params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
    idx: &mut u32,
    col: &str,
    upd: &Update<String>,
) {
    match upd {
        Update::Unchanged => {}
        Update::Set(v) => {
            sets.push(format!("{col} = ?{idx}"));
            params.push(Box::new(v.clone()));
            *idx += 1;
        }
        Update::Null => {
            sets.push(format!("{col} = NULL"));
        }
    }
}

fn collect_update_f64(
    sets: &mut Vec<String>,
    params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
    idx: &mut u32,
    col: &str,
    upd: &Update<f64>,
) {
    match upd {
        Update::Unchanged => {}
        Update::Set(v) => {
            sets.push(format!("{col} = ?{idx}"));
            params.push(Box::new(*v));
            *idx += 1;
        }
        Update::Null => {
            sets.push(format!("{col} = NULL"));
        }
    }
}

fn collect_update_dt(
    sets: &mut Vec<String>,
    params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
    idx: &mut u32,
    col: &str,
    upd: &Update<chrono::DateTime<Utc>>,
) {
    match upd {
        Update::Unchanged => {}
        Update::Set(v) => {
            sets.push(format!("{col} = ?{idx}"));
            params.push(Box::new(v.to_rfc3339()));
            *idx += 1;
        }
        Update::Null => {
            sets.push(format!("{col} = NULL"));
        }
    }
}
