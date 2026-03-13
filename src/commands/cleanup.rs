use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, theme::ColorfulTheme};
use log::debug;
use std::collections::HashSet;

use crate::config::operations::{load_kube_config, save_kube_config};

/// Clean up orphaned clusters and users not referenced by any context
pub fn cleanup_orphans(force: bool) -> Result<()> {
    let mut config = load_kube_config()?;
    debug!(
        "Loaded kube config with {} clusters, {} users, {} contexts",
        config.clusters.len(),
        config.users.len(),
        config.contexts.len()
    );

    // Find all referenced clusters and users
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

    // Find orphaned clusters
    let orphaned_clusters: Vec<String> = config
        .clusters
        .iter()
        .filter(|c| !referenced_clusters.contains(&c.name))
        .map(|c| c.name.clone())
        .collect();

    // Find orphaned users
    let orphaned_users: Vec<String> = config
        .users
        .iter()
        .filter(|u| !referenced_users.contains(&u.name))
        .map(|u| u.name.clone())
        .collect();

    if orphaned_clusters.is_empty() && orphaned_users.is_empty() {
        eprintln!("No orphaned clusters or users found");
        return Ok(());
    }

    // Display what will be cleaned up
    eprintln!("Found orphaned resources:");
    if !orphaned_clusters.is_empty() {
        eprintln!("\nClusters:");
        for cluster in &orphaned_clusters {
            eprintln!("  - {}", style(cluster).cyan());
        }
    }
    if !orphaned_users.is_empty() {
        eprintln!("\nUsers:");
        for user in &orphaned_users {
            eprintln!("  - {}", style(user).cyan());
        }
    }
    eprintln!();

    // Confirmation prompt
    if !force {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Delete these orphaned resources?")
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            eprintln!("Cleanup cancelled");
            return Ok(());
        }
    }

    // Remove orphaned clusters
    for cluster in &orphaned_clusters {
        config.clusters.retain(|c| &c.name != cluster);
        eprintln!(
            "{} Deleted orphaned cluster: {}",
            style("✓").green(),
            style(cluster).cyan()
        );
    }

    // Remove orphaned users
    for user in &orphaned_users {
        config.users.retain(|u| &u.name != user);
        eprintln!(
            "{} Deleted orphaned user: {}",
            style("✓").green(),
            style(user).cyan()
        );
    }

    // Save the config
    save_kube_config(&config)?;

    eprintln!(
        "Cleaned up {} cluster(s) and {} user(s)",
        orphaned_clusters.len(),
        orphaned_users.len()
    );

    Ok(())
}
