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
use database::{sqlite::SqliteDatabase, traits::CacheDatabase};

use crate::{
    api::db_types::{Credential, Mutation},
    database::traits::StoreDatabase,
};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = Cli::parse();
    // println!("{:?}", &cli_config);

    let config = Config::read_config(&cli_config.config)?;
    // println!("{:?}", &config);

    match cli_config.command {
        Commands::Run => {
            launch_server(config).await?;
        }
        Commands::Test => {
            println!("Testing stuff");
            let db = SqliteDatabase::new("data");
            db.add_mutation(
                "test",
                &Mutation::Delete {
                    id: "blahblah".into(),
                },
            )?;
            let res = db.get_next_mutations("test", "0")?;
            println!("{res:?}");
            // db.import_all(
            //     "test",
            //     &vec![Credential {
            //         id: "blah".into(),
            //         value: "chicken".into(),
            //     }],
            // )?;

            let exports = db.export_all("test")?;
            println!("{exports:?}");
        }
    }

    Ok(())
}
