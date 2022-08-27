use std::collections::HashSet;

use anyhow::{Context, Result};
use log::{error, info, warn};
use rocket::{http::Status, response::status, serde::json::Json, State};
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
    pub state_id: String,
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SyncResponse {
    pub status: String,
    pub state_id: Option<String>,
    pub mutations: Option<Vec<Mutation>>,
    pub store: Option<Vec<Credential>>,
    pub id_changes: Option<Vec<(String, String)>>,
}

impl SyncResponse {
    fn add_id_change(&mut self, id: &str, new_id: &str) {
        if self.id_changes.is_none() {
            self.id_changes = Some(Vec::new());
        }
        if let Some(changes) = &mut self.id_changes {
            changes.push((id.into(), new_id.into()));
        }
    }
}

#[post("/sync", data = "<data>")]
pub fn sync_user(
    user: User,
    db: &State<Databases>,
    data: Json<SyncRequest>,
) -> status::Custom<Json<SyncResponse>> {
    let User(alias) = user;
    info!("Syncing user {}", &alias);

    match sync_aux(&alias, db, data) {
        Ok(response) => status::Custom(Status::Ok, Json(response)),
        Err(e) => {
            error!("Failed to sync user\n{:?}", e);
            status::Custom(
                Status::InternalServerError,
                Json(SyncResponse {
                    status: "failed".into(),
                    ..Default::default()
                }),
            )
        }
    }
}

fn sync_aux(
    alias: &str,
    db: &State<Databases>,
    mut data: Json<SyncRequest>,
) -> Result<SyncResponse> {
    let mut response = SyncResponse::default();

    if data.mutations.is_empty() && !data.state_id.is_empty() {
        response.state_id = Some(data.state_id.to_string());
        response.status = "success".to_string();
        return Ok(response);
    }

    // Applying mutations
    data.mutations
        .retain_mut(|mutation| match db.store.apply_mutation(alias, mutation) {
            Ok(None) => true,
            Ok(Some(id)) => {
                if let Mutation::Add { credential } = mutation {
                    response.add_id_change(&credential.id, &id);
                    credential.id = id;
                }
                true
            }
            Err(e) => {
                if let Some(error) = e.downcast_ref::<Error>() {
                    match error {
                        Error::MissingId(id) => warn!(
                            "Credential with id {} missing: modification/deletion skipped",
                            id
                        ),
                        _ => {
                            error!("{:?} failed to apply: skipped", mutation);
                            error!("{:?}", &e)
                        }
                    };
                };
                false
            }
        });

    // Check state
    let state_exists = db
        .cache
        .has_state(alias, &data.state_id)
        .context(format!("Failed to check user state for user {}", alias))?;
    // Return whole store
    if !state_exists {
        info!("State id not found, exporting entire store");
        let store = db
            .store
            .export_all(alias)
            .with_context(|| format!("Failed to export store for user {}", alias))?;
        info!("Exported store for user {}", &alias);
        response.store = Some(store);
    } else {
        info!("State id found, getting remote mutations");
        let mut remote_mutations = db
            .cache
            .get_next_mutations(alias, &data.state_id)
            .with_context(|| format!("Failed to get next mutations for user {}", alias))?;
        // Just apply and return state if most recent
        if !remote_mutations.is_empty() {
            info!("Found new remote state, filtering new mutations");

            // Otherwise perform filters
            // Filter self-overriding local mutations
            let mut overriden_ids: HashSet<String> = HashSet::new();
            data.mutations.retain(|m| {
                overriden_ids.insert(match m {
                    Mutation::Add { credential } => credential.id.to_owned(),
                    Mutation::Delete { credential } => credential.id.to_owned(),
                    Mutation::Modify { credential } => credential.id.to_owned(),
                })
            });

            // Filter out remote mutations for return
            remote_mutations.retain(|m| match m {
                    Mutation::Add { credential } => {
                        if overriden_ids.insert(credential.id.to_owned()) {
                            warn!("Impossible state: credential modified/deleted locally without knowing about remote credential with same id");
                            true
                        } else {
                            false
                        }
                    },
                    Mutation::Modify { credential } => overriden_ids.insert(credential.id.to_owned()),
                    Mutation::Delete { credential } => overriden_ids.insert(credential.id.to_owned())
                });

            response.mutations = Some(remote_mutations);
        } else {
            info!("Already have most recent state");
        }
    }
    let state_id = db
        .cache
        .add_mutations(alias, &data.mutations)
        .with_context(|| format!("Failed to add mutations for user {}", alias))?;
    response.state_id = Some(state_id);
    response.status = "success".into();

    Ok(response)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rocket::{http::Header, local::blocking::Client};
    use serde_json::json;

    use crate::{
        api::{
            db_types::{Credential, Mutation},
            endpoints::init_upload::InitUploadResponse,
            server::build_server,
        },
        config::parse_config::{Config, User},
    };

    use super::SyncResponse;

    fn init_test_config(dir: &str) -> Config {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).expect("Remove test data directory");
        }
        std::fs::create_dir_all(dir).expect("Create test data directory");
        Config {
            users: vec![User {
                alias: "unit".into(),
                keys: vec!["unit".into()],
            }],
            cache_count: 50,
            db_directory: dir.into(),
            enable_test_routes: false,
        }
    }

    fn auth_header() -> Header<'static> {
        Header::new("Authentication", "unit")
    }

    #[test]
    fn simple_add() {
        let config = init_test_config("test/sync/simple_add");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(json!([]).to_string())
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":{
                                "id": "random",
                                "value": "nothing"}
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body_str = &response.into_string().unwrap();
        println!("{}", &body_str);
        let body: SyncResponse = serde_json::from_str(body_str).unwrap();
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert!(body.store.is_none());
        assert!(body.id_changes.is_none());
    }

    #[test]
    fn most_recent() {
        let config = init_test_config("test/sync/most_recent");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(json!([]).to_string())
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let first_response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":{
                                "id": "random",
                                "value": "nothing"}
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let first_body: SyncResponse =
            serde_json::from_str(&first_response.into_string().unwrap()).unwrap();
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": first_body.state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":{
                                "id": "something",
                                "value": "nothing"}
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert!(body.store.is_none());
        assert!(body.id_changes.is_none());
    }

    #[test]
    fn not_recent() {
        let config = init_test_config("test/sync/not_recent");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(json!([]).to_string())
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let _first_response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":{
                                "id": "random",
                                "value": "nothing"}
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":{
                                "id": "something",
                                "value": "nothing"}
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        println!("{:?}", &body);
        assert!(!body.state_id.unwrap().is_empty());
        assert_eq!(
            body.mutations,
            Some(vec![Mutation::Add {
                credential: Credential {
                    id: "random".to_string(),
                    value: "nothing".to_string()
                }
            }])
        );
        assert!(body.store.is_none());
        assert!(body.id_changes.is_none());
    }

    #[test]
    fn missing_state() {
        let config = init_test_config("test/sync/missing_state");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(
                json!([
                    {
                        "id": "random",
                        "value": "nothing"
                    }
                ])
                .to_string(),
            )
            .dispatch();
        let _init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": "asdfasdf",
                    "mutations": [
                        {
                            "type": "add",
                            "credential":
                            {
                                "id": "something",
                                "value": "nothing"
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        println!("{:?}", &body);
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert_eq!(
            body.store,
            Some(vec![
                Credential {
                    id: "random".to_string(),
                    value: "nothing".to_string()
                },
                Credential {
                    id: "something".to_string(),
                    value: "nothing".to_string()
                }
            ])
        );
        assert!(body.id_changes.is_none());
    }

    #[test]
    fn duplicate_id() {
        let config = init_test_config("test/sync/duplicate_id");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(
                json!([
                    {
                        "id": "random",
                        "value": "nothing"
                    }
                ])
                .to_string(),
            )
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":
                            {
                                "id": "random",
                                "value": "nothing"
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        println!("{:?}", &body);
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert!(body.store.is_none());
        assert!(body.id_changes.unwrap().len() == 1);
    }

    #[test]
    fn modify_missing() {
        let config = init_test_config("test/sync/modify_missing");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(
                json!([
                    {
                        "id": "random",
                        "value": "nothing"
                    }
                ])
                .to_string(),
            )
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "modify",
                            "credential":
                            {
                                "id": "missing",
                                "value": "nothing"
                            }
                        },
                        {
                            "type": "delete",
                            "credential": {
                                "id": "missing",
                                "value": ""
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        println!("{:?}", &body);
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert!(body.store.is_none());
        assert!(body.id_changes.is_none());
    }

    #[test]
    fn remote_overriden() {
        let config = init_test_config("test/sync/remote_overriden");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/init/upload")
            .header(auth_header())
            .body(
                json!([
                    {
                        "id": "random",
                        "value": "nothing"
                    }
                ])
                .to_string(),
            )
            .dispatch();
        let init_body: InitUploadResponse =
            serde_json::from_str(&_init.into_string().unwrap()).unwrap();
        let init_state_id = init_body.state_id.expect("Init body state id");
        let _first_response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "add",
                            "credential":
                            {
                                "id": "blah",
                                "value": "nothing"
                            }
                        }
                    ]
                })
                .to_string(),
            );
        let response = client
            .post(uri!(super::sync_user))
            .header(auth_header())
            .body(
                json!({
                    "state_id": &init_state_id,
                    "mutations": [
                        {
                            "type": "modify",
                            "credential":
                            {
                                "id": "blah",
                                "value": "stuff"
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .dispatch();
        let body: SyncResponse = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        println!("{:?}", &body);
        assert!(!body.state_id.unwrap().is_empty());
        assert!(body.mutations.is_none());
        assert!(body.store.is_none());
        assert!(body.id_changes.is_none());
    }
}
