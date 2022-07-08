use std::fmt::Display;

use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::config::parse_config::Config;

pub struct User(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserError;
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
                None => request::Outcome::Failure((Status::NotFound, UserError::MissingUser)),
            }
        } else {
            request::Outcome::Failure((Status::BadRequest, UserError::MissingHeader))
        }
    }
}

#[derive(Debug)]
pub enum UserError {
    MissingHeader,
    MissingUser,
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::MissingHeader => write!(f, "Missing user key in authentication header"),
            UserError::MissingUser => write!(f, "User key does not exist"),
        }
    }
}
