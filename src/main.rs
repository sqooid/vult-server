mod api;
mod config;
pub mod database;
pub mod util;

#[macro_use]
extern crate rocket;

use api::server::{launch_server, prepare_server};
use clap::Parser;
use config::cli::{Cli, Commands};
use config::parse_config::read_config;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = Cli::parse();
    println!("{:?}", &cli_config);

    let config = read_config(&cli_config.config)?;
    println!("{:?}", &config);

    match cli_config.command {
        Commands::Run => {
            prepare_server(&config)?;
            launch_server(config).await?;
        }
        Commands::Test => {
            println!("Testing stuff");
        }
    }

    Ok(())
}
