mod api;
mod config;
pub mod util;

#[macro_use]
extern crate rocket;

use std::env;

use api::server::prepare_server;
use config::parse_config::read_config;

static HELP_TEXT: &str = r#"
usage: vult-server [COMMAND]

    run - Run the server
    help - Show help text
    version - Show the current version
"#;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{HELP_TEXT}");
        return Err("Invalid arguments".into());
    }

    let command = &args[1];
    let config = read_config("config.toml")?;

    // Info commands
    if command == "help" {
    } else if command == "version" {
        println!("version 1.0");
    } else if command == "run" {
        prepare_server(&config)?;
        let _rocket = rocket::build().mount("/", routes![]).launch().await?;
    } else {
        println!("{HELP_TEXT}");
        return Err("Invalid arguments".into());
    }

    Ok(())
}
