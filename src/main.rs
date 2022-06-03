mod api;
mod config;
pub mod util;

#[macro_use]
extern crate rocket;

use std::{env, fmt::Display};

use api::server::prepare_server;

static help_text: &str = r#"
usage: vult-server [COMMAND]

    run - Run the server
    help - Show help text
"#;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::parse_config::read_config("config.toml")?;
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{}", help_text);
        return Err("Invalid arguments".into());
    }

    let command = &args[1];
    if command == "run" {
        prepare_server(&config)?;
        // let _rocket = rocket::build().mount("/", routes![]).launch().await?;
    }

    Ok(())
}
