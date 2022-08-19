use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::{db_types::Credential, guards::user::User},
    database::traits::Databases,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct InitUploadResponse {
    pub state_id: Option<String>,
    pub status: String,
}

#[post("/init/upload", data = "<data>")]
pub fn user_initial_upload(
    user: User,
    db: &State<Databases>,
    data: Json<Vec<Credential>>,
) -> status::Custom<Json<InitUploadResponse>> {
    let User(alias) = user;
    if let Ok(store_empty) = db.store.is_empty(&alias) {
        if let Ok(cache_empty) = db.cache.is_empty(&alias) {
            if !store_empty || !cache_empty {
                status::Custom(
                    Status::Conflict,
                    Json(InitUploadResponse {
                        state_id: None,
                        status: "existing".to_string(),
                    }),
                )
            } else {
                match db.store.import_all(&alias, &data) {
                    Ok(_) => {
                        if let Ok(state_id) = db.cache.add_mutations(&alias, &[]) {
                            status::Custom(
                                Status::Ok,
                                Json(InitUploadResponse {
                                    state_id: Some(state_id),
                                    status: "success".to_string(),
                                }),
                            )
                        } else {
                            status::Custom(
                                Status::InternalServerError,
                                Json(InitUploadResponse {
                                    state_id: None,
                                    status: "failed".to_string(),
                                }),
                            )
                        }
                    }
                    Err(_) => status::Custom(
                        Status::InternalServerError,
                        Json(InitUploadResponse {
                            state_id: None,
                            status: "failed".to_string(),
                        }),
                    ),
                }
            }
        } else {
            status::Custom(
                Status::InternalServerError,
                Json(InitUploadResponse {
                    state_id: None,
                    status: "failed".to_string(),
                }),
            )
        }
    } else {
        status::Custom(
            Status::InternalServerError,
            Json(InitUploadResponse {
                state_id: None,
                status: "failed".to_string(),
            }),
        )
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rocket::{
        http::{Header, Status},
        local::blocking::Client,
    };

    use crate::{
        api::{
            db_types::Credential, endpoints::init_upload::InitUploadResponse, server::build_server,
        },
        config::parse_config::{Config, User},
    };

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

    #[test]
    fn successful_import() {
        let config = init_test_config("test/init_upload/success");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::user_initial_upload))
            .header(Header::new("Authentication", "unit"))
            .body(
                serde_json::to_string(&vec![Credential {
                    id: "nothing".into(),
                    value: "nothing".into(),
                }])
                .unwrap(),
            )
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body: InitUploadResponse =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert_eq!(body.status, "success".to_string());
        assert!(body.state_id.is_some());
    }

    #[test]
    fn conflict() {
        let config = init_test_config("test/init_upload/conflict");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _inital = client
            .post(uri!(super::user_initial_upload))
            .header(Header::new("Authentication", "unit"))
            .body(
                serde_json::to_string(&vec![Credential {
                    id: "nothing".into(),
                    value: "nothing".into(),
                }])
                .unwrap(),
            )
            .dispatch();
        let response = client
            .post(uri!(super::user_initial_upload))
            .header(Header::new("Authentication", "unit"))
            .body(
                serde_json::to_string(&vec![Credential {
                    id: "nothing".into(),
                    value: "nothing".into(),
                }])
                .unwrap(),
            )
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
        let body: InitUploadResponse =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert_eq!(body.status, "existing".to_string());
        assert!(body.state_id.is_none());
    }
}
