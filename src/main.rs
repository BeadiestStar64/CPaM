use clap::Parser;
mod cli;
mod commands;
mod config;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New(args) => commands::new::execute(args),
        Commands::Add(args) => commands::add::execute(args),
        Commands::Remove(args) => commands::remove::execute(args),
        Commands::Build(args) => commands::build::execute(args),
        Commands::Run(args) => commands::run::execute(args),
    }
}
