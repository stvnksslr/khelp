use anyhow::Result;
use console::style;
use log::{debug, info};

use crate::config::operations::{load_kube_config, save_kube_config};

/// Rename a Kubernetes context
///
/// Renames the specified context from old_name to new_name.
/// If the current context matches old_name, it will be updated to new_name.
pub fn rename_context(old_name: String, new_name: String) -> Result<()> {
    debug!(
        "Attempting to rename context from '{}' to '{}'",
        old_name, new_name
    );

    let mut config = load_kube_config()?;
    debug!("Loaded kube config with {} contexts", config.contexts.len());

    // Validate old context exists
    let old_context_exists = config.contexts.iter().any(|c| c.name == old_name);
    if !old_context_exists {
        anyhow::bail!("Context '{}' not found", old_name);
    }

    // Validate new context name doesn't already exist
    let new_context_exists = config.contexts.iter().any(|c| c.name == new_name);
    if new_context_exists {
        anyhow::bail!("Context '{}' already exists", new_name);
    }

    // Prevent renaming to the same name
    if old_name == new_name {
        anyhow::bail!("New name must be different from the current name");
    }

    // Rename the context
    for context in &mut config.contexts {
        if context.name == old_name {
            debug!("Renaming context from '{}' to '{}'", old_name, new_name);
            context.name = new_name.clone();
            break;
        }
    }

    // Update current-context if it matches the old name
    if config.current_context == old_name {
        debug!(
            "Updating current-context from '{}' to '{}'",
            old_name, new_name
        );
        config.current_context = new_name.clone();
    }

    // Save the updated configuration with backup
    save_kube_config(&config)?;

    info!(
        "Renamed context from {} to {}",
        style(&old_name).yellow(),
        style(&new_name).green().bold()
    );

    Ok(())
}
