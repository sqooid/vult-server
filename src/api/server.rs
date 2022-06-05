use crate::{config::parse_config::Config, util::types::GenericResult};
use std::fs;

pub async fn launch_server(config: Config) -> GenericResult {
    let _rocket = rocket::build()
        .manage(config)
        .mount("/", routes![])
        .launch()
        .await?;
    Ok(())
}

pub fn prepare_server(config: &Config) -> GenericResult {
    fs::create_dir_all("data")?;
    for user in &config.users {
        sqlite::open(format!("data/{}", &user.key))?;
    }

    Ok(())
}
