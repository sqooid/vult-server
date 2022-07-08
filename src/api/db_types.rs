use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Mutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
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

#[derive(Debug, Serialize, Deserialize)]
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
