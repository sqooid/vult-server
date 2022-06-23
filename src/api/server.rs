use rocket::{Build, Rocket};

use crate::{
    config::parse_config::Config,
    database::{sqlite::SqliteDatabase, traits::Databases},
};

use super::endpoints::init_user::check_user_state;

pub fn build_server(config: Config) -> Rocket<Build> {
    let sqlite_store = SqliteDatabase::new(&config.db_directory);
    let sqlite_cache = SqliteDatabase::new(&config.db_directory);
    rocket::build()
        .manage(Databases::new(
            Box::new(sqlite_store),
            Box::new(sqlite_cache),
        ))
        .manage(config)
        .mount("/", routes![check_user_state])
}

pub async fn launch_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let _rocket = build_server(config).launch().await?;
    Ok(())
}
