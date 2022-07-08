use std::collections::HashSet;

use log::{error, info, warn};
use rocket::{response, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        db_types::{Credential, Mutation},
        guards::user::User,
    },
    database::traits::Databases,
    util::error::Error,
};

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub state: String,
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    status: String,
    state_id: String,
    mutations: Option<Vec<Mutation>>,
    store: Option<Vec<Credential>>,
    id_changes: Option<Vec<(String, String)>>,
}

impl SyncResponse {
    fn new() -> Self {
        Self {
            status: String::new(),
            state_id: String::new(),
            mutations: None,
            store: None,
            id_changes: None,
        }
    }
    fn add_id_change(&mut self, id: &str, new_id: &str) {
        if self.id_changes.is_none() {
            self.id_changes = Some(Vec::new());
        }
        if let Some(changes) = &mut self.id_changes {
            changes.push((id.into(), new_id.into()));
        }
    }
    fn set_status(&mut self, status: String) {
        self.status = status;
    }
    fn set_mutations(&mut self, mutations: Vec<Mutation>) {
        self.mutations = Some(mutations);
    }
    fn set_store(&mut self, store: Vec<Credential>) {
        self.store = Some(store);
    }
    fn set_state_id(&mut self, state_id: String) {
        self.state_id = state_id;
    }
}

#[get("/sync", data = "<data>")]
pub fn sync_user(
    user: User,
    db: &State<Databases>,
    mut data: Json<SyncRequest>,
) -> Json<SyncResponse> {
    let User(alias) = user;
    info!("Syncing user {}", &alias);
    let mut response = SyncResponse::new();

    // Applying mutations
    data.mutations.retain_mut(
        |mutation| match db.store().apply_mutation(&alias, &mutation) {
            Ok(_) => true,
            Err(Error::DuplicateId { id, new_id }) => {
                if let Mutation::Add { credential } = mutation {
                    response.add_id_change(&id, &new_id);
                    credential.id = new_id;
                }
                true
            }
            Err(Error::MissingItem { id }) => {
                warn!("Item {} missing - skipped", &id);
                false
            }
            Err(_) => {
                error!("Item failed - skipped:\n{}", &mutation);
                false
            }
        },
    );

    // Check state
    if let Ok(state_exists) = db.cache().has_state(&alias, &data.state) {
        if !state_exists {
            if let Ok(state_id) = db.cache().add_mutations(&alias, &data.mutations) {
                response.set_state_id(state_id);
                if let Ok(store) = db.store().export_all(&alias) {
                    info!("Exported store for user {}", &alias);
                    response.set_store(store);
                } else {
                    error!("Failed to export store for user {}", &alias);
                    response.set_status("failed".into());
                }
            } else {
                error!("Failed to add mutations to cache for user {}", &alias);
                response.set_status("failed".into());
            }
        } else {
            if let Ok(mut remote_mutations) = db.cache().get_next_mutations(&alias, &data.state) {
                let mut overriden_ids: HashSet<&str> = HashSet::new();
                for id in data.mutations.iter().filter_map(|m| match m {
                    Mutation::Delete { id } => Some(id),
                    Mutation::Modify { credential } => Some(&credential.id),
                    _ => None,
                }) {
                    overriden_ids.insert(id);
                }

                remote_mutations.retain(|m| match m {
                    Mutation::Add { credential: _ } => {
                    warn!("Impossible state: credential modified/deleted locally without knowing about remote credential with same id");
                    false
                    },
                    Mutation::Modify { credential } => !overriden_ids.contains(&credential.id as&str),
                    Mutation::Delete { id } => !overriden_ids.contains(id as &str)
                });

                response.set_mutations(remote_mutations);
            } else {
                error!(
                    "Failed to retrieve cached remote mutations for user {}",
                    &alias
                );
                response.set_status("failed".into());
            }
        }
    } else {
        error!("Failed to read user state of {}", &alias);
        response.set_status("failed".into());
    }
    Json(response)
}
