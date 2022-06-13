use std::{fmt::Display, time::SystemTimeError};

#[derive(Debug)]
pub enum Error {
    DuplicateId { id: String },
    Time { message: String },
    ExistingUser { message: String },
    Unknown { message: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId { id } => write!(f, "Duplicate id: {}", id),
            Self::Time { message } => write!(f, "Time error: {}", message),
            Self::Unknown { message } => write!(f, "Error: {}", message),
            Self::ExistingUser { message } => write!(f, "Existing user: {}", message),
        }
    }
}

impl std::error::Error for Error {}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Self::Time {
            message: format!("Duration difference {}s", e.duration().as_secs_f64()),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Unknown {
            message: e.to_string(),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Unknown {
            // message: "Error serializing/deserializing bincode".into(),
            message: e.to_string(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Unknown {
            message: e.to_string(),
        }
    }
}
