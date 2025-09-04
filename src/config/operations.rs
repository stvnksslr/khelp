use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

use super::kubernetes::KubeConfig;

/// Gets the path to the Kubernetes config file
pub fn get_kube_config_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not find home directory")?;
    let kube_config_path = home.join(".kube").join("config");

    if !kube_config_path.exists() {
        anyhow::bail!(
            "Kubernetes config file not found at: {}",
            kube_config_path.display()
        );
    }

    debug!(
        "Using Kubernetes config file: {}",
        kube_config_path.display()
    );
    Ok(kube_config_path)
}

/// Loads the Kubernetes config from the default location
pub fn load_kube_config() -> Result<KubeConfig> {
    let kube_config_path = get_kube_config_path()?;
    debug!(
        "Loading Kubernetes config from: {}",
        kube_config_path.display()
    );

    let config_content = fs::read_to_string(&kube_config_path)
        .with_context(|| format!("Failed to read config file: {}", kube_config_path.display()))?;

    let config: KubeConfig =
        serde_yaml::from_str(&config_content).context("Failed to parse Kubernetes config YAML")?;

    debug!(
        "Kubernetes config loaded successfully with {} contexts",
        config.contexts.len()
    );
    Ok(config)
}

/// Saves the Kubernetes config to the default location
///
/// # Arguments
///
/// * `config` - The Kubernetes configuration to save
/// * `create_backup` - Whether to create a backup of the existing config (defaults to true)
pub fn save_kube_config(config: &KubeConfig, create_backup: bool) -> Result<()> {
    let kube_config_path = get_kube_config_path()?;
    debug!(
        "Saving Kubernetes config to: {}",
        kube_config_path.display()
    );

    if create_backup {
        let backup_path = kube_config_path.with_extension("bak");
        fs::copy(&kube_config_path, &backup_path)
            .with_context(|| format!("Failed to create backup at: {}", backup_path.display()))?;
        debug!("Created backup at: {}", backup_path.display());
    }

    let config_yaml =
        serde_yaml::to_string(config).context("Failed to serialize Kubernetes config to YAML")?;

    fs::write(&kube_config_path, config_yaml).with_context(|| {
        format!(
            "Failed to write config file: {}",
            kube_config_path.display()
        )
    })?;

    if create_backup {
        let backup_path = kube_config_path.with_extension("bak");
        info!(
            "Config updated successfully (backup saved at: {})",
            backup_path.display()
        );
    } else {
        info!("Config updated successfully (no backup created)");
    }
    Ok(())
}
