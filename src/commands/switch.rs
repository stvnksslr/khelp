use anyhow::{Context, Result};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::config::operations::{load_kube_config, save_kube_config};

/// Switch to a different Kubernetes context
///
/// If context_name is provided, switches directly to that context.
/// Otherwise, presents an interactive menu to select a context.
pub fn switch_context(context_name: Option<String>) -> Result<()> {
    let mut config = load_kube_config()?;

    let selected_context = match context_name {
        Some(name) => {
            // Find the context by name
            if let Some(context) = config.contexts.iter().find(|c| c.name == name) {
                context.name.clone()
            } else {
                anyhow::bail!("Context '{}' not found", name);
            }
        }
        None => {
            // Interactive selection if no name provided
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to switch to")
                .default(0)
                .items(&config.contexts.iter().map(|c| &c.name).collect::<Vec<_>>())
                .interact()
                .context("Failed to display interactive selection")?;

            config.contexts[selection].name.clone()
        }
    };

    config.current_context = selected_context.clone();

    save_kube_config(&config)?;
    println!(
        "Switched to context: {}",
        style(&selected_context).green().bold()
    );

    Ok(())
}
