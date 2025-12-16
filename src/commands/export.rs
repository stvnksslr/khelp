use anyhow::{Context, Result};
use dialoguer::{MultiSelect, theme::ColorfulTheme};

use crate::config::kubernetes::KubeConfig;
use crate::config::operations::load_kube_config;

/// Export one or more Kubernetes contexts to stdout
///
/// If context_names is provided, exports those contexts directly.
/// Otherwise, presents an interactive menu to select contexts.
/// The output can be redirected to a file.
pub fn export_contexts(context_names: Vec<String>) -> Result<()> {
    let full_config = load_kube_config()?;

    let selected_context_names = if context_names.is_empty() {
        // Interactive selection
        let context_list: Vec<&str> = full_config
            .contexts
            .iter()
            .map(|c| c.name.as_str())
            .collect();

        if context_list.is_empty() {
            anyhow::bail!("No contexts available to export");
        }

        if context_list.len() == 1 {
            // Only one context, just select it
            vec![context_list[0].to_string()]
        } else {
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select contexts to export (Space to select, Enter to confirm)")
                .items(&context_list)
                .interact()
                .context("Failed to display interactive selection")?;

            if selections.is_empty() {
                anyhow::bail!("No contexts selected");
            }

            selections
                .iter()
                .map(|&i| context_list[i].to_string())
                .collect()
        }
    } else {
        // Validate all provided context names exist
        for name in &context_names {
            if !full_config.contexts.iter().any(|c| c.name == *name) {
                anyhow::bail!("Context '{}' not found", name);
            }
        }
        context_names
    };

    // Collect contexts, clusters, and users
    let mut contexts = Vec::new();
    let mut clusters = Vec::new();
    let mut users = Vec::new();

    for context_name in &selected_context_names {
        let context = full_config
            .contexts
            .iter()
            .find(|c| c.name == *context_name)
            .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", context_name))?;

        let cluster_name = &context.context.cluster;
        let cluster = full_config
            .clusters
            .iter()
            .find(|c| c.name == *cluster_name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Cluster '{}' not found for context '{}'",
                    cluster_name,
                    context_name
                )
            })?;

        let user_name = &context.context.user;
        let user = full_config
            .users
            .iter()
            .find(|u| u.name == *user_name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "User '{}' not found for context '{}'",
                    user_name,
                    context_name
                )
            })?;

        // Add if not already present (contexts might share clusters/users)
        if !contexts
            .iter()
            .any(|c: &crate::config::kubernetes::ContextEntry| c.name == context.name)
        {
            contexts.push(context.clone());
        }
        if !clusters
            .iter()
            .any(|c: &crate::config::kubernetes::ClusterEntry| c.name == cluster.name)
        {
            clusters.push(cluster.clone());
        }
        if !users
            .iter()
            .any(|u: &crate::config::kubernetes::UserEntry| u.name == user.name)
        {
            users.push(user.clone());
        }
    }

    // Use the first selected context as the current-context
    let current_context = selected_context_names.first().cloned().unwrap_or_default();

    let config = KubeConfig {
        api_version: full_config.api_version.clone(),
        clusters,
        contexts,
        current_context,
        kind: full_config.kind.clone(),
        preferences: full_config.preferences.clone(),
        users,
    };

    let yaml = serde_yaml::to_string(&config).context("Failed to serialize config to YAML")?;

    println!("{}", yaml);

    Ok(())
}
