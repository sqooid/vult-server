use rocket::{http::Status, State};

use crate::{api::guards::user::User, database::traits::Databases};

#[get("/init")]
pub fn check_user_state(key: User, db: &State<Databases>) -> Status {
    let User(key) = key;
    if let Ok(empty) = db.cache().is_empty(&key) {
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
    use rocket::{
        http::{Header, Status},
        local::blocking::Client,
    };

    use crate::{
        api::{db_types::Mutation, server::build_server},
        config::test_config::init_test_config,
        database::traits::Databases,
    };

    #[test]
    fn invalid_key() {
        let config = init_test_config("test/init/invalid_key");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .get(uri!(init::check_user_state))
            .header(Header::new("Authentication", "random"))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn missing_header() {
        let config = init_test_config("test/init/missing_header");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client.get(uri!(init::check_user_state)).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn ready() {
        let config = init_test_config("test/init/ready");
        let client = Client::tracked(build_server(config)).expect("Valid rocket instance");
        let response = client
            .get(uri!(init::check_user_state))
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
            .add_mutation("unit", &Mutation::Delete { id: "blah".into() })
            .unwrap();
        let response = client
            .get(uri!(init::check_user_state))
            .header(Header::new("Authentication", "unit"))
            .dispatch();
        assert_eq!(response.status(), Status::Conflict);
    }
}
