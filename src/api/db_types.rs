use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum DbMutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[repr(C)]
pub enum Mutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
}

impl From<DbMutation> for Mutation {
    fn from(m: DbMutation) -> Self {
        unsafe { std::mem::transmute::<DbMutation, Mutation>(m) }
    }
}

impl From<Mutation> for DbMutation {
    fn from(m: Mutation) -> Self {
        unsafe { std::mem::transmute::<Mutation, DbMutation>(m) }
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
