use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Mutation {
    Add { credential: Credential },
    Delete { id: String },
    Modify { credential: Credential },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credential {
    id: String,
    value: String,
}
