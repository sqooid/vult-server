use crate::{
    config::parse_config::Config,
    database::{sqlite::SqliteDatabase, traits::Databases},
    util::types::GenericResult,
};
use std::fs;

pub async fn launch_server(config: Config) -> GenericResult<()> {
    let sqlite_store = SqliteDatabase::new(&config.db_directory);
    let sqlite_cache = SqliteDatabase::new(&config.db_directory);
    let _rocket = rocket::build()
        .manage(Databases::new(
            Box::new(sqlite_store),
            Box::new(sqlite_cache),
        ))
        .manage(config)
        .mount("/", routes![])
        .launch()
        .await?;
    Ok(())
}

pub fn prepare_server(config: &Config) -> GenericResult<()> {
    fs::create_dir_all("data")?;
    for user in &config.users {
        sqlite::open(format!("data/{}", &user.key))?;
    }

    Ok(())
}
