use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        db_types::{Credential, Mutation},
        guards::user::User,
    },
    database::traits::Databases,
};

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    state: String,
    mutations: Vec<Mutation>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    state: String,
    mutations: Option<Vec<Mutation>>,
    store: Option<Vec<Credential>>,
    id_changes: Option<Vec<(String, String)>>,
}

#[get("/sync", data = "<data>")]
pub fn sync_user(user: User, db: &State<Databases>, data: Json<SyncRequest>) -> Json<SyncResponse> {
    todo!()
}
