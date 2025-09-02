use anyhow::{Context, Result};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use log::{debug, info};
use std::env;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;
use tempfile;

use crate::config::kubernetes::{ClusterEntry, ContextEntry, KubeConfig, UserEntry};
use crate::config::operations::{load_kube_config, save_kube_config};

/// Represents the content prepared for editing
struct EditContent {
    yaml_content: String,
    context_name: String,
    cluster_name: String,
    user_name: String,
}

/// Represents the result of parsing edited content
struct ParsedEdits {
    context: Option<ContextEntry>,
    cluster: Option<ClusterEntry>,
    user: Option<UserEntry>,
}

/// Prepares the content for editing by extracting the relevant context, cluster, and user entries
fn prepare_edit_content(config: &KubeConfig, context_name: &str) -> Result<EditContent> {
    let context = config
        .contexts
        .iter()
        .find(|c| c.name == context_name)
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", context_name))?;

    let cluster_name = &context.context.cluster;
    let _cluster = config
        .clusters
        .iter()
        .find(|c| &c.name == cluster_name)
        .ok_or_else(|| anyhow::anyhow!("Cluster '{}' not found", cluster_name))?;

    let user_name = &context.context.user;
    let _user = config
        .users
        .iter()
        .find(|u| &u.name == user_name)
        .ok_or_else(|| anyhow::anyhow!("User '{}' not found", user_name))?;

    debug!(
        "Found related cluster: {} and user: {}",
        cluster_name, user_name
    );

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

    let header_comment = format!(
        "# Editing Kubernetes context: {}\n\
         # Make your changes and save the file.\n\
         # The name fields must remain consistent across entries.\n\
         # Available clusters: {}\n\
         # Available users: {}\n\
         #\n\
         # This contains the full context, cluster, and user entries from your ~/.kube/config file.\n\
         # All changes here will be merged back into your config.\n\n",
        context_name, clusters_str, users_str
    );

    let yaml_config = serde_yaml::to_string(&config).context("Failed to convert config to YAML")?;
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(&yaml_config).context("Failed to parse config YAML")?;

    let mut combined_yaml = String::new();
    combined_yaml.push_str(&header_comment);

    if let serde_yaml::Value::Mapping(map) = &yaml_value {
        // Add context entry
        if let Some(serde_yaml::Value::Sequence(contexts)) =
            map.get(serde_yaml::Value::String("contexts".to_string()))
            && let Some(context_entry) = contexts.iter().find(|ctx| {
                if let serde_yaml::Value::Mapping(ctx_map) = ctx
                    && let Some(serde_yaml::Value::String(name)) =
                        ctx_map.get(serde_yaml::Value::String("name".to_string()))
                {
                    return name == context_name;
                }
                false
            })
        {
            combined_yaml.push_str("# Context entry\n");
            let context_yaml = serde_yaml::to_string(context_entry)
                .context("Failed to serialize context entry")?;
            combined_yaml.push_str(&context_yaml);
            combined_yaml.push_str("\n\n");
        }

        // Add cluster entry
        if let Some(serde_yaml::Value::Sequence(clusters)) =
            map.get(serde_yaml::Value::String("clusters".to_string()))
            && let Some(cluster_entry) = clusters.iter().find(|c| {
                if let serde_yaml::Value::Mapping(c_map) = c
                    && let Some(serde_yaml::Value::String(name)) =
                        c_map.get(serde_yaml::Value::String("name".to_string()))
                {
                    return name == cluster_name;
                }
                false
            })
        {
            combined_yaml.push_str("# Cluster entry\n");
            let cluster_yaml = serde_yaml::to_string(cluster_entry)
                .context("Failed to serialize cluster entry")?;
            combined_yaml.push_str(&cluster_yaml);
            combined_yaml.push_str("\n\n");
        }

        // Add user entry
        if let Some(serde_yaml::Value::Sequence(users)) =
            map.get(serde_yaml::Value::String("users".to_string()))
            && let Some(user_entry) = users.iter().find(|u| {
                if let serde_yaml::Value::Mapping(u_map) = u
                    && let Some(serde_yaml::Value::String(name)) =
                        u_map.get(serde_yaml::Value::String("name".to_string()))
                {
                    return name == user_name;
                }
                false
            })
        {
            combined_yaml.push_str("# User entry\n");
            let user_yaml =
                serde_yaml::to_string(user_entry).context("Failed to serialize user entry")?;
            combined_yaml.push_str(&user_yaml);
        }
    }

    debug!("Prepared YAML content for editing");

    Ok(EditContent {
        yaml_content: combined_yaml,
        context_name: context_name.to_string(),
        cluster_name: cluster_name.clone(),
        user_name: user_name.clone(),
    })
}

/// Launches the appropriate editor and handles the editing process
fn launch_editor(temp_file_path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    let is_gui_editor = editor.contains("code") || editor.contains("vscode");

    info!(
        "Opening context configuration in your editor... ({})",
        editor
    );

    let status = if is_gui_editor {
        let mut cmd = Command::new(&editor);
        cmd.arg(temp_file_path);
        let _ = cmd.spawn()?;

        println!("VS Code has been launched. Press Enter when you've finished editing.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        std::process::ExitStatus::from_raw(0)
    } else {
        Command::new(&editor)
            .arg(temp_file_path)
            .status()
            .with_context(|| format!("Failed to open editor for {}", temp_file_path.display()))?
    };

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status code");
    }

    debug!("Editor process completed successfully");
    Ok(())
}

/// Validates and parses the edited content
fn validate_edited_content(
    temp_file_path: &Path,
    edit_content: &EditContent,
) -> Result<ParsedEdits> {
    let edited_content = fs::read_to_string(temp_file_path)
        .with_context(|| format!("Failed to read edited file: {}", temp_file_path.display()))?;

    let content_without_comments = edited_content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

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

    debug!("Parsed {} entries from edited content", entries.len());

    let mut parsed_edits = ParsedEdits {
        context: None,
        cluster: None,
        user: None,
    };

    for entry in entries {
        let entry_yaml: serde_yaml::Value =
            serde_yaml::from_str(entry).context("Failed to parse edited YAML entry")?;

        if let serde_yaml::Value::Mapping(map) = &entry_yaml {
            if let Some(serde_yaml::Value::Mapping(_context_map)) =
                map.get(serde_yaml::Value::String("context".to_string()))
            {
                // Validate context name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                    && name != &edit_content.context_name
                {
                    anyhow::bail!(
                        "Context name cannot be changed (was: {}, now: {})",
                        edit_content.context_name,
                        name
                    );
                }

                parsed_edits.context = Some(
                    serde_yaml::from_value(entry_yaml.clone())
                        .context("Failed to deserialize edited context entry")?,
                );
            } else if let Some(serde_yaml::Value::Mapping(_cluster_map)) =
                map.get(serde_yaml::Value::String("cluster".to_string()))
            {
                // Validate cluster name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                    && name != &edit_content.cluster_name
                {
                    anyhow::bail!(
                        "Cluster name cannot be changed (was: {}, now: {})",
                        edit_content.cluster_name,
                        name
                    );
                }

                parsed_edits.cluster = Some(
                    serde_yaml::from_value(entry_yaml.clone())
                        .context("Failed to deserialize edited cluster entry")?,
                );
            } else if let Some(serde_yaml::Value::Mapping(_user_map)) =
                map.get(serde_yaml::Value::String("user".to_string()))
            {
                // Validate user name hasn't changed
                if let Some(serde_yaml::Value::String(name)) =
                    map.get(serde_yaml::Value::String("name".to_string()))
                    && name != &edit_content.user_name
                {
                    anyhow::bail!(
                        "User name cannot be changed (was: {}, now: {})",
                        edit_content.user_name,
                        name
                    );
                }

                parsed_edits.user = Some(
                    serde_yaml::from_value(entry_yaml.clone())
                        .context("Failed to deserialize edited user entry")?,
                );
            }
        }
    }

    debug!("Successfully validated and parsed edited entries");
    Ok(parsed_edits)
}

/// Merges the edited changes back into the main configuration
fn merge_changes_back(parsed_edits: ParsedEdits, edit_content: &EditContent) -> Result<()> {
    let mut modified_config = load_kube_config()?;

    if let Some(edited_context) = parsed_edits.context
        && let Some(index) = modified_config
            .contexts
            .iter()
            .position(|c| c.name == edit_content.context_name)
    {
        modified_config.contexts[index] = edited_context;
        debug!("Updated context entry in config");
    }

    if let Some(edited_cluster) = parsed_edits.cluster
        && let Some(index) = modified_config
            .clusters
            .iter()
            .position(|c| c.name == edit_content.cluster_name)
    {
        modified_config.clusters[index] = edited_cluster;
        debug!("Updated cluster entry in config");
    }

    if let Some(edited_user) = parsed_edits.user
        && let Some(index) = modified_config
            .users
            .iter()
            .position(|u| u.name == edit_content.user_name)
    {
        modified_config.users[index] = edited_user;
        debug!("Updated user entry in config");
    }

    save_kube_config(&modified_config, true)?;
    info!(
        "Context '{}' configuration updated successfully",
        style(&edit_content.context_name).green().bold()
    );

    Ok(())
}

/// Edit a specific Kubernetes context
///
/// Opens the selected context in the user's preferred editor.
/// If context_name is provided, edits that context directly.
/// Otherwise, presents an interactive menu to select a context.
pub fn edit_context(context_name: Option<String>) -> Result<()> {
    let config = load_kube_config()?;

    let selected_context_name = match context_name {
        Some(name) => {
            if !config.contexts.iter().any(|c| c.name == name) {
                anyhow::bail!("Context '{}' not found", name);
            }
            name
        }
        None => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a context to edit")
                .default(0)
                .items(&config.contexts.iter().map(|c| &c.name).collect::<Vec<_>>())
                .interact()
                .context("Failed to display interactive selection")?;

            config.contexts[selection].name.clone()
        }
    };

    debug!("Selected context to edit: {}", selected_context_name);

    // Step 1: Prepare content for editing
    let edit_content = prepare_edit_content(&config, &selected_context_name)?;

    // Step 2: Write content to temporary file and launch editor
    let temp_dir = tempfile::tempdir()?;
    let temp_file_path = temp_dir.path().join("kube_context_edit.yaml");
    fs::write(&temp_file_path, &edit_content.yaml_content)?;

    launch_editor(&temp_file_path)?;

    // Step 3: Validate and parse the edited content
    let parsed_edits = validate_edited_content(&temp_file_path, &edit_content)?;

    // Step 4: Merge changes back into the main configuration
    merge_changes_back(parsed_edits, &edit_content)?;

    Ok(())
}
