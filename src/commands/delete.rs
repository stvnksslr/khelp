use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use log::{debug, info, warn};
use std::collections::HashSet;

use crate::config::operations::{load_kube_config, save_kube_config};

/// Delete a Kubernetes context
///
/// If context_name is provided, deletes that context directly.
/// Otherwise, presents an interactive menu to select a context.
/// Optionally cleans up orphaned clusters and users.
pub fn delete_context(context_name: Option<String>, force: bool, cleanup: bool) -> Result<()> {
    let mut config = load_kube_config()?;
    debug!("Loaded kube config with {} contexts", config.contexts.len());

    if config.contexts.is_empty() {
        anyhow::bail!("No contexts available to delete");
    }

    // Select context to delete
    let selected_context_name = match context_name {
        Some(name) => {
            debug!("Context name provided: {}", name);
            if !config.contexts.iter().any(|c| c.name == name) {
                anyhow::bail!("Context '{}' not found", name);
            }
            name
        }
        None => {
            debug!("No context name provided, showing selection menu");
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to delete")
                .default(0)
                .items(&config.contexts.iter().map(|c| &c.name).collect::<Vec<_>>())
                .interact()
                .context("Failed to display interactive selection")?;

            config.contexts[selection].name.clone()
        }
    };

    debug!("Selected context to delete: {}", selected_context_name);

    // Check if it's the current context
    let is_current_context = config.current_context == selected_context_name;

    if is_current_context {
        warn!(
            "Context '{}' is currently active",
            style(&selected_context_name).yellow()
        );

        // If there are other contexts, offer to switch
        if config.contexts.len() > 1 {
            let should_switch = if force {
                true
            } else {
                Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Switch to another context first?")
                    .default(true)
                    .interact()
                    .context("Failed to get confirmation")?
            };

            if should_switch {
                let other_contexts: Vec<_> = config
                    .contexts
                    .iter()
                    .filter(|c| c.name != selected_context_name)
                    .map(|c| &c.name)
                    .collect();

                let selection = if force {
                    0
                } else {
                    Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select a context to switch to")
                        .default(0)
                        .items(&other_contexts)
                        .interact()
                        .context("Failed to display interactive selection")?
                };

                let new_context = other_contexts[selection].clone();
                config.current_context = new_context.clone();
                info!(
                    "Switched to context: {}",
                    style(&new_context).green().bold()
                );
            } else {
                anyhow::bail!("Cannot delete the current context without switching first");
            }
        } else {
            // This is the last context, clear current_context
            debug!("Deleting the last remaining context");
            config.current_context = String::new();
        }
    }

    // Confirmation prompt
    if !force {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Are you sure you want to delete context '{}'?",
                selected_context_name
            ))
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            info!("Deletion cancelled");
            return Ok(());
        }
    }

    // Get cluster and user names before deletion for potential cleanup
    let context_to_delete = config
        .contexts
        .iter()
        .find(|c| c.name == selected_context_name)
        .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

    let cluster_name = context_to_delete.context.cluster.clone();
    let user_name = context_to_delete.context.user.clone();

    // Delete the context
    config.contexts.retain(|c| c.name != selected_context_name);
    debug!("Removed context: {}", selected_context_name);

    info!(
        "{} Deleted context: {}",
        style("✓").green(),
        style(&selected_context_name).green().bold()
    );

    // Optional cleanup of orphaned clusters and users
    if cleanup {
        let mut deleted_clusters = Vec::new();
        let mut deleted_users = Vec::new();

        // Find referenced clusters and users
        let referenced_clusters: HashSet<String> = config
            .contexts
            .iter()
            .map(|c| c.context.cluster.clone())
            .collect();

        let referenced_users: HashSet<String> = config
            .contexts
            .iter()
            .map(|c| c.context.user.clone())
            .collect();

        // Delete orphaned cluster
        if !referenced_clusters.contains(&cluster_name) {
            config.clusters.retain(|c| c.name != cluster_name);
            deleted_clusters.push(cluster_name);
            debug!("Removed orphaned cluster");
        }

        // Delete orphaned user
        if !referenced_users.contains(&user_name) {
            config.users.retain(|u| u.name != user_name);
            deleted_users.push(user_name);
            debug!("Removed orphaned user");
        }

        // Report cleanup results
        for cluster in deleted_clusters {
            info!(
                "{} Deleted orphaned cluster: {}",
                style("✓").green(),
                style(&cluster).cyan()
            );
        }

        for user in deleted_users {
            info!(
                "{} Deleted orphaned user: {}",
                style("✓").green(),
                style(&user).cyan()
            );
        }
    }

    // Save the config
    save_kube_config(&config)?;

    Ok(())
}
