use rocket::{http::Status, State};

use crate::{api::guards::user::User, database::traits::Databases};

#[get("/init")]
pub fn check_user_state(user: User, db: &State<Databases>) -> Status {
    let User(alias) = user;
    if let Ok(empty) = db.cache().is_empty(&alias) {
        match empty {
            true => Status::Ok,
            _ => Status::Conflict,
        }
    } else {
        Status::BadRequest
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
            .get(uri!(super::check_user_state))
            .header(Header::new("Authentication", "random"))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn missing_header() {
        let config = init_test_config("test/init/missing_header");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client.get(uri!(super::check_user_state)).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn ready() {
        let config = init_test_config("test/init/ready");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .get(uri!(super::check_user_state))
            .header(Header::new("Authentication", "unit"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn taken() {
        let config = init_test_config("test/init/taken");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let db = client
            .rocket()
            .state::<Databases>()
            .expect("State databases");
        db.cache()
            .add_mutations("unit", &[Mutation::Delete { id: "blah".into() }])
            .unwrap();
        let response = client
            .get(uri!(super::check_user_state))
            .header(Header::new("Authentication", "unit"))
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
    }
}
