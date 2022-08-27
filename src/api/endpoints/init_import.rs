use crate::{api::guards::user::User, database::traits::Databases, util::error::Error};
use log::{info, warn};
use rocket::{http::Status, response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserImportResponse {
    pub status: String,
    pub salt: Option<String>,
    pub hash: Option<String>,
}

#[get("/user/import")]
pub fn get_user(user: User, db: &State<Databases>) -> status::Custom<Json<UserImportResponse>> {
    let User(alias) = user;
    let result = db.user.get_user(&alias);
    match result {
        Ok(user) => {
            info!("Provided salt for user {}", &alias);
            status::Custom(
                Status::Ok,
                Json(UserImportResponse {
                    status: "success".into(),
                    salt: Some(user.0),
                    hash: Some(user.1),
                }),
            )
        }
        Err(e) => {
            let error_response = status::Custom(
                Status::InternalServerError,
                Json(UserImportResponse {
                    status: "failed".into(),
                    salt: None,
                    hash: None,
                }),
            );
            if let Some(e) = e.downcast_ref::<Error>() {
                match e {
                    Error::UninitializedUser(_) => {
                        warn!("Failed to provided salt for uninitialized user {}", &alias);
                        status::Custom(
                            Status::Conflict,
                            Json(UserImportResponse {
                                status: "uninitialized".into(),
                                salt: None,
                                hash: None,
                            }),
                        )
                    }
                    _ => {
                        error!("Failed to import user: {:?}", e);
                        error_response
                    }
                }
            } else {
                error!("Failed to import user: {:?}", e);
                error_response
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rocket::{
        http::{Header, Status},
        local::blocking::Client,
    };
    use serde_json::{json, Value};

    use crate::{
        api::server::build_server,
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

    fn auth_header() -> Header<'static> {
        Header::new("Authentication", "unit")
    }

    #[test]
    fn get_user_success() {
        let config = init_test_config("test/init_import/success");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _init = client
            .post("/user/init")
            .header(auth_header())
            .body(json!({"salt":"somesalt", "hash": "somehash"}).to_string())
            .dispatch();
        let response = client
            .get(uri!(super::get_user))
            .header(auth_header())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body: Value = response.into_json().unwrap();
        assert_eq!(
            body,
            json!({"status":"success","salt":"somesalt", "hash": "somehash"})
        );
    }

    #[test]
    fn get_user_uninitialized() {
        let config = init_test_config("test/init_import/uninit");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .get(uri!(super::get_user))
            .header(auth_header())
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
        let body: Value = response.into_json().unwrap();
        assert_eq!(
            body,
            json!({"status":"uninitialized","salt":null,"hash":null})
        );
    }
}
