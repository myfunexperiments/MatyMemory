use thiserror::Error;

#[derive(Debug, Error)]
pub enum MatyError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Invalid status transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, MatyError>;
