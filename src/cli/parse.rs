use crate::store::error::{MatyError, Result};
use crate::store::models::{MemoryStatus, MemoryType, SearchFilters};

pub fn parse_memory_type(s: &str) -> Result<MemoryType> {
    s.parse()
}

pub fn parse_memory_status(s: &str) -> Result<MemoryStatus> {
    s.parse()
}

pub fn parse_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

pub fn build_search_filters(
    query: Option<&str>,
    memory_type: Option<&str>,
    status: Option<&str>,
    tag: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchFilters> {
    let mt = memory_type.map(parse_memory_type).transpose()?;
    let st = status.map(parse_memory_status).transpose()?;
    Ok(SearchFilters {
        text: query.map(String::from),
        memory_type: mt,
        status: st,
        tag: tag.map(String::from),
        limit,
        offset,
    })
}

pub fn parse_importance(v: f64) -> Result<f64> {
    if !(0.0..=1.0).contains(&v) {
        return Err(MatyError::InvalidInput(
            "Importance must be between 0.0 and 1.0".to_string(),
        ));
    }
    Ok(v)
}

pub fn parse_confidence(v: f64) -> Result<f64> {
    if !(0.0..=1.0).contains(&v) {
        return Err(MatyError::InvalidInput(
            "Confidence must be between 0.0 and 1.0".to_string(),
        ));
    }
    Ok(v)
}
