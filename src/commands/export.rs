use anyhow::{Context, Result};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::config::kubernetes::KubeConfig;
use crate::config::operations::load_kube_config;

/// Export a specific Kubernetes context to stdout
///
/// If context_name is provided, exports that context directly.
/// Otherwise, presents an interactive menu to select a context.
/// The output can be redirected to a file.
pub fn export_context(context_name: Option<String>) -> Result<()> {
    // Load the config
    let full_config = load_kube_config()?;

    // Get the context to export
    let selected_context_name = match context_name {
        Some(name) => {
            // Find the context by name
            if !full_config.contexts.iter().any(|c| c.name == name) {
                anyhow::bail!("Context '{}' not found", name);
            }
            name
        }
        None => {
            // Interactive selection if no name provided
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to export")
                .default(0)
                .items(
                    &full_config
                        .contexts
                        .iter()
                        .map(|c| &c.name)
                        .collect::<Vec<_>>(),
                )
                .interact()
                .context("Failed to display interactive selection")?;

            full_config.contexts[selection].name.clone()
        }
    };

    // Find the context
    let context = full_config
        .contexts
        .iter()
        .find(|c| c.name == selected_context_name)
        .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

    // Find the cluster
    let cluster_name = &context.context.cluster;
    let cluster = full_config
        .clusters
        .iter()
        .find(|c| c.name == *cluster_name)
        .ok_or_else(|| anyhow::anyhow!("Cluster not found"))?;

    // Find the user
    let user_name = &context.context.user;
    let user = full_config
        .users
        .iter()
        .find(|u| u.name == *user_name)
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    // Create a new config with only the selected components
    let config = KubeConfig {
        api_version: full_config.api_version.clone(),
        clusters: vec![cluster.clone()],
        contexts: vec![context.clone()],
        current_context: selected_context_name.clone(),
        kind: full_config.kind.clone(),
        preferences: full_config.preferences.clone(),
        users: vec![user.clone()],
    };

    // Output the YAML to stdout
    let yaml = serde_yaml::to_string(&config).context("Failed to serialize config to YAML")?;

    // Print to stdout (can be redirected to file)
    println!("{}", yaml);

    Ok(())
}
