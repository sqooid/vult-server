use std::fmt::Display;

#[derive(Debug)]
pub struct DbError {
    error: DbErrorType,
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Debug)]
pub enum DbErrorType {
    DuplicateId { id: String },
    Unknown { message: String },
}

impl Display for DbErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId { id } => write!(f, "Duplicate id: {}", id),
            Self::Unknown { message } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for DbError {}
