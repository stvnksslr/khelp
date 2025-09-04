mod cli;
mod commands;
mod config;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::List) {
        Commands::List => {
            let config = config::operations::load_kube_config()?;
            commands::list::list_contexts(&config);
        }
        Commands::Current => {
            let config = config::operations::load_kube_config()?;
            commands::current::show_current_context(&config);
        }
        Commands::Switch { context_name } => {
            commands::switch::switch_context(context_name)?;
        }
        Commands::Edit { context_name } => {
            commands::edit::edit_context(context_name)?;
        }
        Commands::Export { context_name } => {
            commands::export::export_context(context_name)?;
        }
    }

    Ok(())
}
