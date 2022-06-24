use rocket::serde::json::Json;
use rocket::{http::Status, State};

use crate::{
    api::{db_types::Credential, guards::user::User},
    database::traits::Databases,
};

#[get("/init/upload", data = "<data>")]
pub fn user_initial_upload(
    key: User,
    db: &State<Databases>,
    data: Json<Vec<Credential>>,
) -> Status {
    let User(key) = key;
    if let Ok(store_empty) = db.store().is_empty(&key) {
        if let Ok(cache_empty) = db.cache().is_empty(&key) {
            if !store_empty || !cache_empty {
                Status::Conflict
            } else {
                match db.store().import_all(&key, &data) {
                    Ok(_) => Status::Ok,
                    Err(_) => Status::InternalServerError,
                }
            }
        } else {
            Status::InternalServerError
        }
    } else {
        Status::InternalServerError
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
            db_types::{Credential},
            server::build_server,
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
                alias: None,
                key: "unit".into(),
            }],
            cache_count: 50,
            db_directory: dir.into(),
        }
    }

    #[test]
    fn successful_import() {
        let config = init_test_config("test/init_upload/success");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .get(uri!(super::user_initial_upload))
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
    }

    #[test]
    fn conflict() {
        let config = init_test_config("test/init_upload/conflict");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _inital = client
            .get(uri!(super::user_initial_upload))
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
            .get(uri!(super::user_initial_upload))
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
    }
}
