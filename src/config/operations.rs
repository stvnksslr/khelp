use anyhow::{Context, Result};
use dirs::home_dir;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

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
    load_kube_config_from(&kube_config_path)
}

/// Loads the Kubernetes config from a custom path
///
/// # Arguments
///
/// * `path` - Path to the kubeconfig file
pub fn load_kube_config_from(path: &Path) -> Result<KubeConfig> {
    debug!("Loading Kubernetes config from: {}", path.display());

    let config_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

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
pub fn save_kube_config(config: &KubeConfig) -> Result<()> {
    let kube_config_path = get_kube_config_path()?;
    save_kube_config_to(config, &kube_config_path)
}

/// Saves the Kubernetes config to a custom path
///
/// # Arguments
///
/// * `config` - The Kubernetes configuration to save
/// * `path` - Path where the config should be saved
pub fn save_kube_config_to(config: &KubeConfig, path: &Path) -> Result<()> {
    debug!("Saving Kubernetes config to: {}", path.display());

    let config_yaml =
        serde_yaml::to_string(config).context("Failed to serialize Kubernetes config to YAML")?;

    fs::write(path, config_yaml)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    info!("Config updated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Helper function to create a sample valid kubeconfig YAML
    fn sample_kubeconfig_yaml() -> String {
        r#"apiVersion: v1
clusters:
- cluster:
    certificate-authority-data: LS0tLS1CRUdJTi
    server: https://127.0.0.1:6443
  name: test-cluster
contexts:
- context:
    cluster: test-cluster
    user: test-user
    namespace: default
  name: test-context
current-context: test-context
kind: Config
preferences: {}
users:
- name: test-user
  user:
    client-certificate-data: LS0tLS1CRUdJTi
    client-key-data: LS0tLS1CRUdJTi
"#
        .to_string()
    }

    #[test]
    fn test_load_kube_config_from_valid_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), sample_kubeconfig_yaml())
            .expect("Failed to write to temp file");

        let result = load_kube_config_from(temp_file.path());
        assert!(result.is_ok(), "Should load valid config successfully");

        let config = result.unwrap();
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Config");
        assert_eq!(config.current_context, "test-context");
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.contexts[0].name, "test-context");
        assert_eq!(config.clusters.len(), 1);
        assert_eq!(config.clusters[0].name, "test-cluster");
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].name, "test-user");
    }

    #[test]
    fn test_load_kube_config_from_nonexistent_file() {
        let result = load_kube_config_from(Path::new("/nonexistent/path/config"));
        assert!(result.is_err(), "Should fail for non-existent file");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to read config file"),
            "Error should mention failed to read: {}",
            error_msg
        );
    }

    #[test]
    fn test_load_kube_config_from_invalid_yaml() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), "invalid: yaml: [content")
            .expect("Failed to write to temp file");

        let result = load_kube_config_from(temp_file.path());
        assert!(result.is_err(), "Should fail for invalid YAML");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to parse Kubernetes config YAML"),
            "Error should mention YAML parse failure: {}",
            error_msg
        );
    }

    #[test]
    fn test_save_kube_config_to() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), sample_kubeconfig_yaml())
            .expect("Failed to write initial config");

        let config = load_kube_config_from(temp_file.path()).expect("Failed to load config");

        let save_result = save_kube_config_to(&config, temp_file.path());
        assert!(save_result.is_ok(), "Should save config successfully");

        // Verify file was written
        let content = std::fs::read_to_string(temp_file.path()).expect("Failed to read saved file");
        assert!(content.contains("test-context"));
        assert!(content.contains("test-cluster"));
        assert!(content.contains("test-user"));
    }

    #[test]
    fn test_save_kube_config_with_backup() {
        // Create a temporary directory for this test
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config");

        // Write initial config
        std::fs::write(&config_path, sample_kubeconfig_yaml())
            .expect("Failed to write initial config");

        // Load and save
        let mut config = load_kube_config_from(&config_path).expect("Failed to load config");
        config.current_context = "modified-context".to_string();

        save_kube_config_to(&config, &config_path).expect("Failed to save config");

        // Verify the modification was saved
        let saved_config = load_kube_config_from(&config_path).expect("Failed to reload config");
        assert_eq!(saved_config.current_context, "modified-context");
    }

    #[test]
    fn test_round_trip_save_and_load() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), sample_kubeconfig_yaml())
            .expect("Failed to write initial config");

        // Load original config
        let original_config =
            load_kube_config_from(temp_file.path()).expect("Failed to load original config");

        // Modify and save
        let mut modified_config = original_config.clone();
        modified_config.current_context = "updated-context".to_string();

        save_kube_config_to(&modified_config, temp_file.path())
            .expect("Failed to save modified config");

        // Load again and verify changes persisted
        let reloaded_config =
            load_kube_config_from(temp_file.path()).expect("Failed to reload config");

        assert_eq!(reloaded_config.current_context, "updated-context");
        assert_eq!(
            reloaded_config.contexts.len(),
            modified_config.contexts.len()
        );
        assert_eq!(
            reloaded_config.clusters.len(),
            modified_config.clusters.len()
        );
        assert_eq!(reloaded_config.users.len(), modified_config.users.len());
    }

    #[test]
    fn test_save_to_invalid_path() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), sample_kubeconfig_yaml())
            .expect("Failed to write initial config");

        let config = load_kube_config_from(temp_file.path()).expect("Failed to load config");

        // Try to save to an invalid path (directory that doesn't exist)
        let invalid_path = Path::new("/nonexistent/directory/config");
        let result = save_kube_config_to(&config, invalid_path);

        assert!(result.is_err(), "Should fail to save to invalid path");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to write config file"),
            "Error should mention write failure: {}",
            error_msg
        );
    }
}
