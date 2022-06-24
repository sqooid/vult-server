use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::{config::parse_config::Config, util::error::Error};

pub struct User(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Error;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(key) = req.headers().get_one("Authentication") {
            let config = req
                .rocket()
                .state::<Config>()
                .expect("Rocket instance contains managed state for server config");
            match config
                .users
                .iter()
                .find(|i| i.keys.contains(&key.to_owned()))
            {
                Some(user) => request::Outcome::Success(Self(user.alias.to_owned())),
                None => request::Outcome::Failure((
                    Status::NotFound,
                    Error::Unknown {
                        message: "Key does not belong to any user".into(),
                    },
                )),
            }
        } else {
            request::Outcome::Failure((
                Status::BadRequest,
                Error::Unknown {
                    message: "Missing key in authorization header".into(),
                },
            ))
        }
    }
}
