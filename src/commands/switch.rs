use anyhow::{Context, Result};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use log::{debug, info};

use crate::config::operations::{load_kube_config, save_kube_config};

/// Switch to a different Kubernetes context
///
/// If context_name is provided, switches directly to that context.
/// Otherwise, presents an interactive menu to select a context.
pub fn switch_context(context_name: Option<String>) -> Result<()> {
    let mut config = load_kube_config()?;
    debug!("Loaded kube config with {} contexts", config.contexts.len());

    let selected_context = match context_name {
        Some(name) => {
            debug!("Context name provided: {}", name);
            if let Some(context) = config.contexts.iter().find(|c| c.name == name) {
                context.name.clone()
            } else {
                anyhow::bail!("Context '{}' not found", name);
            }
        }
        None => {
            debug!("No context name provided, showing selection menu");
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to switch to")
                .default(0)
                .items(&config.contexts.iter().map(|c| &c.name).collect::<Vec<_>>())
                .interact()
                .context("Failed to display interactive selection")?;

            config.contexts[selection].name.clone()
        }
    };

    debug!("Selected context: {}", selected_context);

    let old_context = config.current_context.clone();
    config.current_context = selected_context.clone();
    debug!(
        "Changing current context from '{}' to '{}'",
        old_context, selected_context
    );

    save_kube_config(&config, false)?;

    info!(
        "Switched to context: {}",
        style(&selected_context).green().bold()
    );

    Ok(())
}
