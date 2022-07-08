use std::{fmt::Display, time::SystemTimeError};

#[derive(Debug)]
pub enum Error {
    DuplicateId { id: String, new_id: String },
    MissingItem { id: String },
    Time { message: String },
    ExistingUser { message: String },
    Database { error: rusqlite::Error },
    Unknown { error: Box<dyn std::error::Error> },
    Config { message: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId { id, new_id } => {
                write!(f, "Duplicate id: {id} changed to {new_id}")
            }
            Self::MissingItem { id } => write!(f, "Missing item: {id}"),
            Self::Time { message } => write!(f, "Time error: {}", message),
            Self::Unknown { error } => write!(f, "Error: {}", error),
            Self::ExistingUser { message } => write!(f, "Already existing user: {}", message),
            Self::Database { error } => write!(f, "Database error: {}", error),
            Self::Config { message } => write!(f, "Config error: {}", message),
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
        Self::Database { error: e }
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Unknown {
            // message: "Error serializing/deserializing bincode".into(),
            error: e,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Unknown { error: Box::new(e) }
    }
}

impl From<toml::de::Error> for Error {
    fn from(_: toml::de::Error) -> Self {
        Self::Config {
            message: "Failed to parse config file".into(),
        }
    }
}
