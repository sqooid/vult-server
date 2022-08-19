use rocket::{Build, Rocket};

use crate::{
    config::parse_config::Config,
    database::{sqlite::SqliteDatabase, traits::Databases},
};

use super::endpoints::{
    init::initialize_user, init_import::get_user, init_upload::user_initial_upload,
    sync::sync_user, test_reset::reset_databases,
};

pub fn build_server(config: Config) -> Rocket<Build> {
    let enable_test_routes = config.enable_test_routes.to_owned();
    let sqlite_store = SqliteDatabase::new(&config.db_directory);
    let sqlite_cache = SqliteDatabase::new(&config.db_directory);
    let sqlite_salt = SqliteDatabase::new(&config.db_directory);
    rocket::build()
        .manage(Databases::new(
            Box::new(sqlite_store),
            Box::new(sqlite_cache),
            Box::new(sqlite_salt),
        ))
        .manage(config)
        .mount(
            "/",
            if enable_test_routes {
                routes![
                    initialize_user,
                    user_initial_upload,
                    sync_user,
                    get_user,
                    reset_databases
                ]
            } else {
                routes![initialize_user, user_initial_upload, sync_user, get_user]
            },
        )
}

pub async fn launch_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let _rocket = build_server(config).launch().await?;
    Ok(())
}
