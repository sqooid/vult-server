use std::{fmt::Display, time::SystemTimeError};

#[derive(Debug)]
pub enum Error {
    DuplicateId { id: String },
    Time { message: String },
    Unknown { message: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId { id } => write!(f, "Duplicate id: {}", id),
            Self::Time { message } => write!(f, "Time error: {}", message),
            Self::Unknown { message } => write!(f, "{}", message),
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

impl From<sqlite::Error> for Error {
    fn from(e: sqlite::Error) -> Self {
        Self::Unknown {
            message: e.message.unwrap_or_default(),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(_: bincode::Error) -> Self {
        Self::Unknown {
            message: "Error serializing/deserializing bincode".into(),
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
