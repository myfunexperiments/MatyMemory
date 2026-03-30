use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::error::MatyError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    Pinned,
    Semantic,
    Episodic,
    Procedural,
    Session,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pinned => "pinned",
            Self::Semantic => "semantic",
            Self::Episodic => "episodic",
            Self::Procedural => "procedural",
            Self::Session => "session",
        };
        f.write_str(s)
    }
}

impl FromStr for MemoryType {
    type Err = MatyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pinned" => Ok(Self::Pinned),
            "semantic" => Ok(Self::Semantic),
            "episodic" => Ok(Self::Episodic),
            "procedural" => Ok(Self::Procedural),
            "session" => Ok(Self::Session),
            other => Err(MatyError::InvalidInput(format!(
                "Unknown memory type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryStatus {
    Active,
    Archived,
    Invalidated,
    Superseded,
}

impl fmt::Display for MemoryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Invalidated => "invalidated",
            Self::Superseded => "superseded",
        };
        f.write_str(s)
    }
}

impl FromStr for MemoryStatus {
    type Err = MatyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            "invalidated" => Ok(Self::Invalidated),
            "superseded" => Ok(Self::Superseded),
            other => Err(MatyError::InvalidInput(format!(
                "Unknown memory status: {other}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Three-state update wrapper
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub enum Update<T> {
    #[default]
    Unchanged,
    Set(T),
    Null,
}

// ---------------------------------------------------------------------------
// Core structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub base_importance: f64,
    pub confidence: f64,
    pub status: MemoryStatus,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub supersedes_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryWithTags {
    pub memory: Memory,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct CreateMemoryRequest {
    pub content: String,
    pub memory_type: MemoryType,
    pub tags: Vec<String>,
    pub base_importance: f64,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    // Provenance fields
    pub actor: String,
    pub session_id: Option<String>,
    pub model_id: Option<String>,
    pub write_reason: Option<String>,
}

#[derive(Debug, Default)]
pub struct MemoryUpdate {
    pub content: Update<String>,
    pub base_importance: Update<f64>,
    pub confidence: Update<f64>,
    pub valid_from: Update<DateTime<Utc>>,
    pub valid_to: Update<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Supporting structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct Provenance {
    pub id: i64,
    pub memory_id: String,
    pub actor: String,
    pub session_id: Option<String>,
    pub model_id: Option<String>,
    pub write_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Relation {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: String,
}

#[derive(Debug)]
pub struct RetrievalLogEntry {
    pub query: String,
    pub memory_id: String,
    pub score: f64,
    pub explanation: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub memory_id: String,
    pub owner: String,
    pub project: Option<String>,
    pub read_scope: String,
    pub write_scope: String,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct MemoryStats {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub by_status: HashMap<String, usize>,
}

#[derive(Debug, Default)]
pub struct SearchFilters {
    pub text: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub status: Option<MemoryStatus>,
    pub tag: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SearchFilters {
    pub fn limit_or_default(&self) -> u32 {
        self.limit.unwrap_or(50)
    }

    pub fn offset_or_default(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}
