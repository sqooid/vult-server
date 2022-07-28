use crate::util::error::Error;
use crate::{api::guards::user::User, database::traits::Databases};
use anyhow::Result;
use log::error;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct InitRequest {
    pub salt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InitResponse {
    pub status: String,
}

#[post("/user/init", data = "<data>")]
pub fn initialize_user(
    user: User,
    db: &State<Databases>,
    data: Json<InitRequest>,
) -> status::Custom<Json<InitResponse>> {
    let User(alias) = user;
    let result = add_salt_aux(db, &alias, &data.salt);
    match result {
        Ok(true) => status::Custom(
            Status::Ok,
            Json(InitResponse {
                status: "success".to_string(),
            }),
        ),
        Ok(false) => status::Custom(
            Status::Conflict,
            Json(InitResponse {
                status: "existing".to_string(),
            }),
        ),
        Err(e) => {
            error!("Failed to initialize user: {:?}", e);
            status::Custom(
                Status::InternalServerError,
                Json(InitResponse {
                    status: "failed".to_string(),
                }),
            )
        }
    }
}

fn add_salt_aux(db: &State<Databases>, alias: &str, salt: &str) -> Result<bool> {
    let result = db.salt.get_salt(&alias);
    match result {
        Ok(_) => Ok(false),
        Err(e) => {
            let e = e.downcast::<Error>()?;
            match e {
                Error::UninitializedUser(_) => {
                    db.salt.set_salt(alias, salt)?;
                    Ok(true)
                }
                _ => Err(e.into()),
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
    use serde_json::json;

    use crate::{
        api::{db_types::Mutation, server::build_server},
        config::parse_config::{Config, User},
        database::traits::Databases,
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
        }
    }

    #[test]
    fn invalid_key() {
        let config = init_test_config("test/init/invalid_key");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::initialize_user))
            .header(Header::new("Authentication", "random"))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn missing_header() {
        let config = init_test_config("test/init/missing_header");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client.post(uri!(super::initialize_user)).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn ready() {
        let config = init_test_config("test/init/ready");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::initialize_user))
            .header(Header::new("Authentication", "unit"))
            .body(json!({"salt": "somesalt"}).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert_eq!(body, json!({"status": "success"}).to_string())
    }

    #[test]
    fn taken() {
        let config = init_test_config("test/init/taken");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let _initial_response = client
            .post(uri!(super::initialize_user))
            .header(Header::new("Authentication", "unit"))
            .body(json!({"salt": "somesalt"}).to_string())
            .dispatch();
        let response = client
            .post(uri!(super::initialize_user))
            .header(Header::new("Authentication", "unit"))
            .body(json!({"salt": "somesalt"}).to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
        let body = response.into_string().unwrap();
        assert_eq!(body, json!({"status": "existing"}).to_string());
    }
}
