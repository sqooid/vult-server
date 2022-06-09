use crate::{
    config::parse_config::Config,
    database::{sqlite::SqliteDatabase, traits::Databases},
};

pub async fn launch_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
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
