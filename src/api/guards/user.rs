use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::{config::parse_config::Config, util::error::Error};

pub struct User(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Error;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(key) = req.headers().get_one("Authentication") {
            let config = req
                .rocket()
                .state::<Config>()
                .expect("Missing server config in rocket instance?");
            if config.users.iter().any(|i| i.key == key) {
                request::Outcome::Success(Self(key.into()))
            } else {
                request::Outcome::Failure((
                    Status::NotFound,
                    Error::Unknown {
                        message: "Key does not belong to any user".into(),
                    },
                ))
            }
        } else {
            request::Outcome::Failure((
                Status::NotFound,
                Error::Unknown {
                    message: "Missing key in authorization header".into(),
                },
            ))
        }
    }
}
