use rocket::{Build, Rocket};

use crate::{
    config::parse_config::Config,
    database::{sqlite::SqliteDatabase, traits::Databases},
};

use super::endpoints::{init::check_user_state, init_upload::user_initial_upload, sync::sync_user};

pub fn build_server(config: Config) -> Rocket<Build> {
    let sqlite_store = SqliteDatabase::new(&config.db_directory);
    let sqlite_cache = SqliteDatabase::new(&config.db_directory);
    rocket::build()
        .manage(Databases::new(
            Box::new(sqlite_store),
            Box::new(sqlite_cache),
        ))
        .manage(config)
        .mount(
            "/",
            routes![check_user_state, user_initial_upload, sync_user],
        )
}

pub async fn launch_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let _rocket = build_server(config).launch().await?;
    Ok(())
}
