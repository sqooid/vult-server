mod api;
mod config;
pub mod database;
pub mod util;

#[macro_use]
extern crate rocket;

use api::server::launch_server;
use clap::Parser;
use config::{
    cli::{Cli, Commands},
    parse_config::Config,
};
use log::info;

use crate::database::traits::CacheDatabase;
use crate::{api::db_types::Mutation, database::sqlite::SqliteDatabase};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = Cli::parse();

    pretty_env_logger::formatted_timed_builder()
        .filter(
            None,
            match &cli_config.verbose {
                &true => log::LevelFilter::Info,
                _ => log::LevelFilter::Warn,
            },
        )
        .init();

    let config = Config::read_config(&cli_config.config)?;
    info!("Parsed config:\n{}", &config);

    match cli_config.command {
        Commands::Run => {
            launch_server(config).await?;
        }
        Commands::Test => {
            info!("Testing stuff");
        }
    }

    Ok(())
}
