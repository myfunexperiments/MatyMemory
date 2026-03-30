use std::path::PathBuf;

pub fn resolve_db_path(cli_override: Option<&str>) -> PathBuf {
    if let Some(path) = cli_override {
        return PathBuf::from(path);
    }

    if let Ok(env_path) = std::env::var("MATY_DB_PATH") && !env_path.is_empty() {
        return PathBuf::from(env_path);
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".matymemory").join("memory.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_override_takes_priority() {
        let path = resolve_db_path(Some("/tmp/test.db"));
        assert_eq!(path, PathBuf::from("/tmp/test.db"));
    }

    #[test]
    fn default_uses_home_dir() {
        let path = resolve_db_path(None);
        assert!(path.ends_with(".matymemory/memory.db"));
    }
}
