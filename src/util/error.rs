use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Missing credential with id: {0}")]
    MissingId(String),
    #[error("User with alias {0} already exists")]
    ExistingUser(String),
    #[error("Internal server error")]
    Server(#[source] anyhow::Error),
    #[error("Invalid server configuration")]
    Config(#[source] anyhow::Error),
}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Self::Server(e.into())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Server(e.into())
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Server(e.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Server(e.into())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::Config(e.into())
    }
}
