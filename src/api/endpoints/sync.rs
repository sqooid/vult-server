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
    state: String,
    mutations: Option<Vec<Mutation>>,
    store: Option<Vec<Credential>>,
    id_changes: Option<Vec<(String, String)>>,
}

impl SyncResponse {
    fn new() -> Self {
        Self {
            state: String::new(),
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
}

#[get("/sync", data = "<data>")]
pub fn sync_user(user: User, db: &State<Databases>, data: Json<SyncRequest>) -> Json<SyncResponse> {
    // let User(alias) = user;
    // let mut response = SyncResponse::new();
    // data.mutations.retain_mut(
    //     |mutation| match db.store().apply_mutation(&alias, &mutation) {
    //         Ok(_) => true,
    //         Err(Error::DuplicateId { id, new_id }) => {
    //             if let Mutation::Add { credential } = mutation {
    //                 credential.id = new_id;
    //             }
    //             response.add_id_change(&id, &new_id);
    //             true
    //         }
    //         Err(Error::MissingItem { id: () })
    //     },
    // );
    todo!()
}
