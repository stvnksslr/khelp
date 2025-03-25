use anyhow::{Context, Result};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use std::env;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use tempfile;

use crate::config::operations::{load_kube_config, save_kube_config};

/// Edit a specific Kubernetes context
///
/// Opens the selected context in the user's preferred editor.
/// If context_name is provided, edits that context directly.
/// Otherwise, presents an interactive menu to select a context.
pub fn edit_context(context_name: Option<String>) -> Result<()> {
    // Load the config to get the list of contexts and identify which one to edit
    let config = load_kube_config()?;

    // Get the context to edit
    let selected_context_name = match context_name {
        Some(name) => {
            // Find the context by name
            if !config.contexts.iter().any(|c| c.name == name) {
                anyhow::bail!("Context '{}' not found", name);
            }
            name
        }
        None => {
            // Interactive selection if no name provided
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to edit")
                .default(0)
                .items(&config.contexts.iter().map(|c| &c.name).collect::<Vec<_>>())
                .interact()
                .context("Failed to display interactive selection")?;

            config.contexts[selection].name.clone()
        }
    };

    // Find the selected context
    let context = config
        .contexts
        .iter()
        .find(|c| c.name == selected_context_name)
        .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

    // Find the associated cluster
    let cluster_name = &context.context.cluster;
    let _cluster = config
        .clusters
        .iter()
        .find(|c| &c.name == cluster_name)
        .ok_or_else(|| anyhow::anyhow!("Cluster '{}' not found", cluster_name))?;

    // Find the associated user
    let user_name = &context.context.user;
    let _user = config
        .users
        .iter()
        .find(|u| &u.name == user_name)
        .ok_or_else(|| anyhow::anyhow!("User '{}' not found", user_name))?;

    // Convert clusters and users to string lists for the header
    let clusters_str = config
        .clusters
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let users_str = config
        .users
        .iter()
        .map(|u| u.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // Add helpful header comments
    let header_comment = format!(
        "# Editing Kubernetes context: {}\n\
         # Make your changes and save the file.\n\
         # The name fields must remain consistent across entries.\n\
         # Available clusters: {}\n\
         # Available users: {}\n\
         #\n\
         # This contains the full context, cluster, and user entries from your ~/.kube/config file.\n\
         # All changes here will be merged back into your config.\n\n",
        selected_context_name, clusters_str, users_str
    );

    // Now convert the relevant portions to YAML
    let yaml_config = serde_yaml::to_string(&config).context("Failed to convert config to YAML")?;
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(&yaml_config).context("Failed to parse config YAML")?;

    let mut combined_yaml = String::new();
    combined_yaml.push_str(&header_comment);

    // Extract and add the context
    if let serde_yaml::Value::Mapping(map) = &yaml_value {
        if let Some(serde_yaml::Value::Sequence(contexts)) =
            map.get(serde_yaml::Value::String("contexts".to_string()))
        {
            // Find the specific context
            if let Some(context) = contexts.iter().find(|ctx| {
                if let serde_yaml::Value::Mapping(ctx_map) = ctx {
                    if let Some(serde_yaml::Value::String(name)) =
                        ctx_map.get(serde_yaml::Value::String("name".to_string()))
                    {
                        return name == &selected_context_name;
                    }
                }
                false
            }) {
                combined_yaml.push_str("# Context entry\n");
                let context_yaml = serde_yaml::to_string(context).unwrap_or_default();
                combined_yaml.push_str(&context_yaml);
                combined_yaml.push_str("\n\n");
            }
        }

        // Extract and add the cluster
        if let Some(serde_yaml::Value::Sequence(clusters)) =
            map.get(serde_yaml::Value::String("clusters".to_string()))
        {
            // Find the specific cluster
            if let Some(cluster) = clusters.iter().find(|c| {
                if let serde_yaml::Value::Mapping(c_map) = c {
                    if let Some(serde_yaml::Value::String(name)) =
                        c_map.get(serde_yaml::Value::String("name".to_string()))
                    {
                        return name == cluster_name;
                    }
                }
                false
            }) {
                combined_yaml.push_str("# Cluster entry\n");
                let cluster_yaml = serde_yaml::to_string(cluster).unwrap_or_default();
                combined_yaml.push_str(&cluster_yaml);
                combined_yaml.push_str("\n\n");
            }
        }

        // Extract and add the user
        if let Some(serde_yaml::Value::Sequence(users)) =
            map.get(serde_yaml::Value::String("users".to_string()))
        {
            // Find the specific user
            if let Some(user) = users.iter().find(|u| {
                if let serde_yaml::Value::Mapping(u_map) = u {
                    if let Some(serde_yaml::Value::String(name)) =
                        u_map.get(serde_yaml::Value::String("name".to_string()))
                    {
                        return name == user_name;
                    }
                }
                false
            }) {
                combined_yaml.push_str("# User entry\n");
                let user_yaml = serde_yaml::to_string(user).unwrap_or_default();
                combined_yaml.push_str(&user_yaml);
            }
        }
    }

    // Create a more persistent temporary file that won't be deleted immediately
    let temp_dir = tempfile::tempdir()?;
    let temp_file_path = temp_dir.path().join("kube_context_edit.yaml");

    // Write content to this file
    fs::write(&temp_file_path, combined_yaml)?;

    // Determine which editor to use
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    // Check if editor is VS Code or similar GUI editor that doesn't block
    let is_gui_editor = editor.contains("code") || editor.contains("vscode");

    println!(
        "Opening context configuration in your editor... ({})",
        editor
    );

    let status = if is_gui_editor {
        // For VS Code, don't wait for process to finish
        let mut cmd = Command::new(&editor);
        cmd.arg(&temp_file_path);
        let _ = cmd.spawn()?;

        // Add a pause for user to edit and signal when done
        println!("VS Code has been launched. Press Enter when you've finished editing.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Simulate success
        std::process::ExitStatus::from_raw(0)
    } else {
        // For terminal editors, wait as usual
        Command::new(&editor)
            .arg(&temp_file_path)
            .status()
            .with_context(|| format!("Failed to open editor for {}", temp_file_path.display()))?
    };

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status code");
    }

    // Read the modified file
    let edited_content = fs::read_to_string(&temp_file_path)
        .with_context(|| format!("Failed to read edited file: {}", temp_file_path.display()))?;

    // Skip comment lines when parsing
    let content_without_comments = edited_content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // Split the content based on double newlines to get separate entries
    let entries: Vec<&str> = content_without_comments
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if entries.is_empty() || entries.len() > 3 {
        anyhow::bail!(
            "Expected 1-3 configuration entries (context, cluster, user), found {}",
            entries.len()
        );
    }

    // Parse each entry
    let mut edited_context_value: Option<serde_yaml::Value> = None;
    let mut edited_cluster_value: Option<serde_yaml::Value> = None;
    let mut edited_user_value: Option<serde_yaml::Value> = None;

    for entry in entries {
        let entry_yaml: serde_yaml::Value =
            serde_yaml::from_str(entry).context("Failed to parse edited YAML entry")?;

        if let serde_yaml::Value::Mapping(map) = &entry_yaml {
            // Check if this is a context, cluster, or user entry
            if let Some(serde_yaml::Value::Mapping(_context_map)) =
                map.get(serde_yaml::Value::String("context".to_string()))
            {
                // This is a context entry
                edited_context_value = Some(entry_yaml.clone());

                // Validate context name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                {
                    if name != &selected_context_name {
                        anyhow::bail!(
                            "Context name cannot be changed (was: {}, now: {})",
                            selected_context_name,
                            name
                        );
                    }
                }
            } else if let Some(serde_yaml::Value::Mapping(_cluster_map)) =
                map.get(serde_yaml::Value::String("cluster".to_string()))
            {
                // This is a cluster entry
                edited_cluster_value = Some(entry_yaml.clone());

                // Validate cluster name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                {
                    if name != cluster_name {
                        anyhow::bail!(
                            "Cluster name cannot be changed (was: {}, now: {})",
                            cluster_name,
                            name
                        );
                    }
                }
            } else if let Some(serde_yaml::Value::Mapping(_user_map)) =
                map.get(serde_yaml::Value::String("user".to_string()))
            {
                // This is a user entry
                edited_user_value = Some(entry_yaml.clone());

                // Validate user name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                {
                    if name != user_name {
                        anyhow::bail!(
                            "User name cannot be changed (was: {}, now: {})",
                            user_name,
                            name
                        );
                    }
                }
            }
        }
    }

    // Now update the config with the edited values
    let mut modified_config = load_kube_config()?;

    // Update context if edited
    if let Some(edited_context) = edited_context_value {
        if let Ok(edited_context_entry) =
            serde_yaml::from_value::<crate::config::kubernetes::ContextEntry>(edited_context)
        {
            // Find the context to update
            if let Some(index) = modified_config
                .contexts
                .iter()
                .position(|c| c.name == selected_context_name)
            {
                modified_config.contexts[index] = edited_context_entry;
            }
        }
    }

    // Update cluster if edited
    if let Some(edited_cluster) = edited_cluster_value {
        if let Ok(edited_cluster_entry) =
            serde_yaml::from_value::<crate::config::kubernetes::ClusterEntry>(edited_cluster)
        {
            // Find the cluster to update
            if let Some(index) = modified_config
                .clusters
                .iter()
                .position(|c| &c.name == cluster_name)
            {
                modified_config.clusters[index] = edited_cluster_entry;
            }
        }
    }

    // Update user if edited
    if let Some(edited_user) = edited_user_value {
        if let Ok(edited_user_entry) =
            serde_yaml::from_value::<crate::config::kubernetes::UserEntry>(edited_user)
        {
            // Find the user to update
            if let Some(index) = modified_config
                .users
                .iter()
                .position(|u| &u.name == user_name)
            {
                modified_config.users[index] = edited_user_entry;
            }
        }
    }

    // Save the updated config
    save_kube_config(&modified_config)?;
    println!(
        "Context '{}' configuration updated successfully",
        style(&selected_context_name).green().bold()
    );

    Ok(())
}
