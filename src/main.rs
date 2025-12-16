mod cli;
mod commands;
mod config;
mod utils;

use anyhow::Result;
use clap::Parser;
use log::{debug, info};

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    debug!("Starting khelp application");

    let cli = Cli::parse();
    debug!("Command line arguments parsed");

    match cli.command.unwrap_or(Commands::List) {
        Commands::List => {
            debug!("Executing List command");
            let config = config::operations::load_kube_config()?;
            commands::list::list_contexts(&config);
        }
        Commands::Current => {
            debug!("Executing Current command");
            let config = config::operations::load_kube_config()?;
            commands::current::show_current_context(&config);
        }
        Commands::Switch { context_name } => {
            debug!("Executing Switch command");
            commands::switch::switch_context(context_name)?;
        }
        Commands::Edit { context_name } => {
            debug!("Executing Edit command");
            commands::edit::edit_context(context_name)?;
        }
        Commands::Export { context_names } => {
            debug!("Executing Export command");
            commands::export::export_contexts(context_names)?;
        }
        Commands::Delete {
            context_name,
            force,
            cleanup,
        } => {
            debug!("Executing Delete command");
            commands::delete::delete_context(context_name, force, cleanup)?;
        }
        Commands::Rename { old_name, new_name } => {
            debug!("Executing Rename command");
            commands::rename::rename_context(old_name, new_name)?;
        }
        Commands::Add {
            file_path,
            rename,
            overwrite,
            switch,
        } => {
            debug!("Executing Add command with file: {:?}", file_path);
            commands::add::add_context(file_path, rename, overwrite, switch)?;
        }
        Commands::Completions { shell, install } => {
            debug!(
                "Executing Completions command with shell: {:?}, install: {}",
                shell, install
            );

            if let Some(s) = shell {
                debug!("Shell explicitly specified: {:?}", s);

                debug!("Generating completions");
                commands::completions::generate_completions(s, install)?;
            } else if install {
                debug!("No shell specified, detecting current shell...");

                match commands::completions::detect_shell() {
                    Ok(detected_shell) => {
                        debug!("Successfully detected shell: {:?}", detected_shell);

                        match commands::completions::generate_completions(detected_shell, true) {
                            Ok(_) => {
                                info!("Completions installed successfully");
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                debug!("No shell specified and not installing");
                anyhow::bail!("Shell must be specified when not using --install")
            }

            debug!("Completions command execution finished");
        }
        #[cfg(feature = "self_update")]
        Commands::Update { apply } => {
            debug!("Executing Update command with apply: {}", apply);
            commands::update::handle_update(apply)?;
        }
    }

    debug!("khelp execution completed successfully");
    Ok(())
}
