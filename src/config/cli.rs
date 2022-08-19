use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version)]
/// Personal sync server for Vult
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Path to configuration TOML file
    #[clap(global = true, short, long, default_value_t = String::from("./config.toml"))]
    pub config: String,

    /// Verbose logging
    #[clap(global = true, short, long)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the server
    Run {
        /// Enable test routes
        #[clap(short, long)]
        test: bool,
    },

    /// Run test functions
    Test,
}
