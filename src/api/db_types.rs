use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Mutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ClientMutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
}

impl From<Mutation> for ClientMutation {
    fn from(m: Mutation) -> Self {
        match m {
            Mutation::Add { credential } => Self::Add { credential },
            Mutation::Delete { id } => Self::Delete { id },
            Mutation::Modify { credential } => Self::Modify { credential },
        }
    }
}

impl From<ClientMutation> for Mutation {
    fn from(m: ClientMutation) -> Self {
        match m {
            ClientMutation::Add { credential } => Self::Add { credential },
            ClientMutation::Delete { id } => Self::Delete { id },
            ClientMutation::Modify { credential } => Self::Modify { credential },
        }
    }
}

impl Display for Mutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mutation::Add { credential } => write!(f, "Add\n{}", credential),
            Mutation::Delete { id } => write!(f, "Delete\n{}", id),
            Mutation::Modify { credential } => write!(f, "Modify\n{}", credential),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Credential {
    pub id: String,
    pub value: String,
}

impl Display for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}
