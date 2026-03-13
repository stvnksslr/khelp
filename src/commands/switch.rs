use anyhow::{Context, Result};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use log::debug;

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

            // Build display items with current context annotation
            let display_items: Vec<String> = config
                .contexts
                .iter()
                .map(|c| {
                    if c.name == config.current_context {
                        format!("{} (current)", c.name)
                    } else {
                        c.name.clone()
                    }
                })
                .collect();

            // Pre-select the current context
            let default_idx = config
                .contexts
                .iter()
                .position(|c| c.name == config.current_context)
                .unwrap_or(0);

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to switch to")
                .default(default_idx)
                .items(&display_items)
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

    save_kube_config(&config)?;

    eprintln!(
        "Switched to context: {}",
        style(&selected_context).green().bold()
    );

    Ok(())
}
