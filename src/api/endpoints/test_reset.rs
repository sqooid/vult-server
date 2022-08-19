use std::path::Path;

use anyhow::Result;
use rocket::{http::Status, State};

use crate::{api::guards::user::User, config::parse_config::Config, database::traits::Databases};

#[post("/test/reset")]
pub fn reset_databases(user: User, config: &State<Config>, db: &State<Databases>) -> Status {
    let User(alias) = user;
    match clear_database(&alias, &config.db_directory, db) {
        Ok(_) => Status::Ok,
        Err(_) => {
            error!("Failed to reset for user {}", &alias);
            Status::InternalServerError
        }
    }
}

fn clear_database(alias: &str, db_directory: &str, db: &State<Databases>) -> Result<()> {
    db.salt.remove_salt(alias)?;
    std::fs::remove_file(Path::new(db_directory).join(format!("{alias}.sqlite")))?;
    Ok(())
}
