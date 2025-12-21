use anyhow::{Context, Result};
use console::style;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::config::kubernetes::{ContextEntry, KubeConfig};
use crate::config::operations::{load_kube_config_or_default, save_kube_config};

#[derive(Debug)]
pub struct ImportSummary {
    pub contexts_added: Vec<String>,
    pub clusters_added: Vec<String>,
    pub users_added: Vec<String>,
    pub contexts_skipped: Vec<String>,
    pub clusters_skipped: Vec<String>,
    pub users_skipped: Vec<String>,
    pub contexts_overwritten: Vec<String>,
    pub clusters_overwritten: Vec<String>,
    pub users_overwritten: Vec<String>,
}

impl ImportSummary {
    fn new() -> Self {
        Self {
            contexts_added: Vec::new(),
            clusters_added: Vec::new(),
            users_added: Vec::new(),
            contexts_skipped: Vec::new(),
            clusters_skipped: Vec::new(),
            users_skipped: Vec::new(),
            contexts_overwritten: Vec::new(),
            clusters_overwritten: Vec::new(),
            users_overwritten: Vec::new(),
        }
    }

    fn has_changes(&self) -> bool {
        !self.contexts_added.is_empty()
            || !self.clusters_added.is_empty()
            || !self.users_added.is_empty()
            || !self.contexts_overwritten.is_empty()
            || !self.clusters_overwritten.is_empty()
            || !self.users_overwritten.is_empty()
    }

    fn print_summary(&self) {
        println!("\n{}", style("Import Summary:").green().bold());
        println!("{}", style("───────────────").green());

        if !self.contexts_added.is_empty() {
            println!(
                "{} {} context(s): {}",
                style("✓").green(),
                style("Added").green().bold(),
                self.contexts_added.join(", ")
            );
        }
        if !self.clusters_added.is_empty() {
            println!(
                "{} {} cluster(s): {}",
                style("✓").green(),
                style("Added").green().bold(),
                self.clusters_added.join(", ")
            );
        }
        if !self.users_added.is_empty() {
            println!(
                "{} {} user(s): {}",
                style("✓").green(),
                style("Added").green().bold(),
                self.users_added.join(", ")
            );
        }

        if !self.contexts_overwritten.is_empty() {
            println!(
                "{} {} context(s): {}",
                style("↻").yellow(),
                style("Overwritten").yellow().bold(),
                self.contexts_overwritten.join(", ")
            );
        }
        if !self.clusters_overwritten.is_empty() {
            println!(
                "{} {} cluster(s): {}",
                style("↻").yellow(),
                style("Overwritten").yellow().bold(),
                self.clusters_overwritten.join(", ")
            );
        }
        if !self.users_overwritten.is_empty() {
            println!(
                "{} {} user(s): {}",
                style("↻").yellow(),
                style("Overwritten").yellow().bold(),
                self.users_overwritten.join(", ")
            );
        }

        if !self.contexts_skipped.is_empty() {
            println!(
                "{} {} context(s): {}",
                style("−").dim(),
                style("Skipped").dim(),
                self.contexts_skipped.join(", ")
            );
        }
        if !self.clusters_skipped.is_empty() {
            println!(
                "{} {} cluster(s): {}",
                style("−").dim(),
                style("Skipped").dim(),
                self.clusters_skipped.join(", ")
            );
        }
        if !self.users_skipped.is_empty() {
            println!(
                "{} {} user(s): {}",
                style("−").dim(),
                style("Skipped").dim(),
                self.users_skipped.join(", ")
            );
        }
    }
}

/// Add contexts from an external kubeconfig file into the main config
///
/// # Arguments
///
/// * `file_path` - Path to the external kubeconfig file
/// * `rename` - Whether to rename conflicting entries
/// * `overwrite` - Whether to overwrite existing entries
/// * `switch` - Whether to switch to the first imported context
pub fn add_context(file_path: PathBuf, rename: bool, overwrite: bool, switch: bool) -> Result<()> {
    // Validate file path
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    debug!("Loading external kubeconfig from: {}", file_path.display());

    // Load external config
    let external_config_content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Check for empty file
    let trimmed = external_config_content.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Config file is empty: {}\n\nThe kubeconfig file you're trying to add contains no data.",
            file_path.display()
        );
    }

    let mut external_config: KubeConfig = serde_yaml::from_str(&external_config_content)
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("missing field `apiVersion`") || error_msg.contains("missing field `kind`") {
                anyhow::anyhow!(
                    "Invalid kubeconfig file: {}\n\nThe file appears to be missing required fields (apiVersion, kind).\n\nOriginal error: {}",
                    file_path.display(),
                    error_msg
                )
            } else if error_msg.contains("missing field") {
                anyhow::anyhow!(
                    "Invalid kubeconfig file: {}\n\n{}\n\nPlease check that your kubeconfig file has all required fields.",
                    file_path.display(),
                    error_msg
                )
            } else {
                anyhow::anyhow!(
                    "Failed to parse kubeconfig file: {}\n\n{}",
                    file_path.display(),
                    error_msg
                )
            }
        })?;

    debug!(
        "External config loaded: {} contexts, {} clusters, {} users",
        external_config.contexts.len(),
        external_config.clusters.len(),
        external_config.users.len()
    );

    // Handle missing contexts array (like sandbox-kubeconfig.yaml)
    if external_config.contexts.is_empty() && !external_config.clusters.is_empty() {
        warn!("No contexts found in external config, attempting to create from current-context");

        if !external_config.current_context.is_empty() {
            let cluster_name = external_config
                .clusters
                .first()
                .map(|c| c.name.clone())
                .unwrap_or_default();

            let user_name = external_config
                .users
                .first()
                .map(|u| u.name.clone())
                .unwrap_or_default();

            if !cluster_name.is_empty() && !user_name.is_empty() {
                let context_entry = ContextEntry {
                    name: external_config.current_context.clone(),
                    context: crate::config::kubernetes::ContextData {
                        cluster: cluster_name,
                        user: user_name,
                        namespace: Some("default".to_string()),
                    },
                };
                external_config.contexts.push(context_entry);
                info!("Created context entry: {}", external_config.current_context);
            }
        }
    }

    // Validate we have something to import
    if external_config.contexts.is_empty()
        && external_config.clusters.is_empty()
        && external_config.users.is_empty()
    {
        anyhow::bail!("External kubeconfig contains no contexts, clusters, or users to import");
    }

    // Load main config (or create empty one if it doesn't exist or is empty)
    let mut main_config = load_kube_config_or_default()?;
    debug!(
        "Main config loaded: {} contexts, {} clusters, {} users",
        main_config.contexts.len(),
        main_config.clusters.len(),
        main_config.users.len()
    );

    let mut summary = ImportSummary::new();

    // Track name mappings for renamed entities
    let mut cluster_name_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut user_name_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Import clusters
    for cluster in external_config.clusters {
        let cluster_name = cluster.name.clone();

        if let Some(existing_idx) = main_config
            .clusters
            .iter()
            .position(|c| c.name == cluster_name)
        {
            if overwrite {
                main_config.clusters[existing_idx] = cluster;
                summary.clusters_overwritten.push(cluster_name.clone());
                debug!("Overwritten cluster: {}", cluster_name);
            } else if rename {
                let new_name = find_available_name(&cluster_name, &get_cluster_names(&main_config));
                cluster_name_map.insert(cluster_name.clone(), new_name.clone());
                let mut renamed_cluster = cluster;
                renamed_cluster.name = new_name.clone();
                main_config.clusters.push(renamed_cluster);
                summary.clusters_added.push(new_name.clone());
                debug!("Added renamed cluster: {} -> {}", cluster_name, new_name);
            } else {
                summary.clusters_skipped.push(cluster_name.clone());
                debug!("Skipped existing cluster: {}", cluster_name);
            }
        } else {
            main_config.clusters.push(cluster);
            summary.clusters_added.push(cluster_name.clone());
            debug!("Added cluster: {}", cluster_name);
        }
    }

    // Import users
    for user in external_config.users {
        let user_name = user.name.clone();

        if let Some(existing_idx) = main_config.users.iter().position(|u| u.name == user_name) {
            if overwrite {
                main_config.users[existing_idx] = user;
                summary.users_overwritten.push(user_name.clone());
                debug!("Overwritten user: {}", user_name);
            } else if rename {
                let new_name = find_available_name(&user_name, &get_user_names(&main_config));
                user_name_map.insert(user_name.clone(), new_name.clone());
                let mut renamed_user = user;
                renamed_user.name = new_name.clone();
                main_config.users.push(renamed_user);
                summary.users_added.push(new_name.clone());
                debug!("Added renamed user: {} -> {}", user_name, new_name);
            } else {
                summary.users_skipped.push(user_name.clone());
                debug!("Skipped existing user: {}", user_name);
            }
        } else {
            main_config.users.push(user);
            summary.users_added.push(user_name.clone());
            debug!("Added user: {}", user_name);
        }
    }

    // Import contexts
    let mut first_added_context: Option<String> = None;

    for mut context in external_config.contexts {
        let context_name = context.name.clone();

        // Update cluster and user references if they were renamed
        if let Some(new_cluster_name) = cluster_name_map.get(&context.context.cluster) {
            context.context.cluster = new_cluster_name.clone();
        }
        if let Some(new_user_name) = user_name_map.get(&context.context.user) {
            context.context.user = new_user_name.clone();
        }

        if let Some(existing_idx) = main_config
            .contexts
            .iter()
            .position(|c| c.name == context_name)
        {
            if overwrite {
                main_config.contexts[existing_idx] = context;
                summary.contexts_overwritten.push(context_name.clone());
                if first_added_context.is_none() {
                    first_added_context = Some(context_name.clone());
                }
                debug!("Overwritten context: {}", context_name);
            } else if rename {
                let new_name = find_available_name(&context_name, &get_context_names(&main_config));
                let mut renamed_context = context;
                renamed_context.name = new_name.clone();
                main_config.contexts.push(renamed_context);
                summary.contexts_added.push(new_name.clone());
                if first_added_context.is_none() {
                    first_added_context = Some(new_name.clone());
                }
                debug!("Added renamed context: {} -> {}", context_name, new_name);
            } else {
                summary.contexts_skipped.push(context_name.clone());
                debug!("Skipped existing context: {}", context_name);
            }
        } else {
            main_config.contexts.push(context);
            summary.contexts_added.push(context_name.clone());
            if first_added_context.is_none() {
                first_added_context = Some(context_name.clone());
            }
            debug!("Added context: {}", context_name);
        }
    }

    // Check if any changes were made
    if !summary.has_changes() {
        warn!("No changes made - all entries already exist in the main config");
        summary.print_summary();
        println!(
            "\n{} Use {} to rename conflicting entries or {} to overwrite them.",
            style("Tip:").cyan().bold(),
            style("--rename").yellow(),
            style("--overwrite").yellow()
        );
        return Ok(());
    }

    // Save the config
    save_kube_config(&main_config)?;

    // Print summary
    summary.print_summary();

    // Switch to first added context if requested
    if switch {
        if let Some(context_name) = first_added_context {
            main_config.current_context = context_name.clone();
            save_kube_config(&main_config)?;
            info!(
                "\nSwitched to context: {}",
                style(&context_name).green().bold()
            );
        } else {
            warn!("No new contexts were added to switch to");
        }
    }

    Ok(())
}

/// Find an available name by appending a suffix
fn find_available_name(base_name: &str, existing_names: &HashSet<String>) -> String {
    let mut counter = 1;
    let mut new_name = format!("{}-imported", base_name);

    while existing_names.contains(&new_name) {
        counter += 1;
        new_name = format!("{}-imported-{}", base_name, counter);
    }

    new_name
}

/// Get all cluster names from config
fn get_cluster_names(config: &KubeConfig) -> HashSet<String> {
    config.clusters.iter().map(|c| c.name.clone()).collect()
}

/// Get all user names from config
fn get_user_names(config: &KubeConfig) -> HashSet<String> {
    config.users.iter().map(|u| u.name.clone()).collect()
}

/// Get all context names from config
fn get_context_names(config: &KubeConfig) -> HashSet<String> {
    config.contexts.iter().map(|c| c.name.clone()).collect()
}
