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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Global environment lock for all tests that manipulate HOME
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn create_test_kube_config() -> String {
        r#"apiVersion: v1
clusters:
- cluster:
    certificate-authority-data: LS0tLS1CRUdJTi...
    server: https://cluster1.example.com
  name: cluster1
- cluster:
    certificate-authority-data: LS0tLS1CRUdJTi...
    server: https://cluster2.example.com
  name: cluster2
contexts:
- context:
    cluster: cluster1
    user: user1
    namespace: default
  name: context1
- context:
    cluster: cluster2
    user: user2
  name: context2
current-context: context1
kind: Config
preferences: {}
users:
- name: user1
  user:
    client-certificate-data: LS0tLS1CRUdJTi...
    client-key-data: LS0tLS1CRUdJTi...
- name: user2
  user:
    token: eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9..."#
            .to_string()
    }

    #[test]
    fn test_load_kube_config_success() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        let kube_dir = temp_dir.path().join(".kube");
        fs::create_dir_all(&kube_dir).unwrap();

        let config_path = kube_dir.join("config");
        fs::write(&config_path, create_test_kube_config()).unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let result = load_kube_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.contexts.len(), 2);
        assert_eq!(config.current_context, "context1");
        assert_eq!(config.clusters.len(), 2);
        assert_eq!(config.users.len(), 2);

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_load_kube_config_file_not_found() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let result = load_kube_config();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Kubernetes config file not found")
        );

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_load_kube_config_invalid_yaml() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        let kube_dir = temp_dir.path().join(".kube");
        fs::create_dir_all(&kube_dir).unwrap();

        let config_path = kube_dir.join("config");
        fs::write(&config_path, "invalid: yaml: content: [").unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let result = load_kube_config();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse Kubernetes config YAML")
        );

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_save_kube_config_with_backup() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        let kube_dir = temp_dir.path().join(".kube");
        fs::create_dir_all(&kube_dir).unwrap();

        let config_path = kube_dir.join("config");
        let original_content = create_test_kube_config();
        fs::write(&config_path, &original_content).unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let mut config = load_kube_config().unwrap();
        config.current_context = "context2".to_string();

        let result = save_kube_config(&config, true);
        assert!(result.is_ok());

        let backup_path = config_path.with_extension("bak");
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        let updated_content = fs::read_to_string(&config_path).unwrap();

        assert_eq!(backup_content, original_content);
        assert!(updated_content.contains("current-context: context2"));

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_save_kube_config_without_backup() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        let kube_dir = temp_dir.path().join(".kube");
        fs::create_dir_all(&kube_dir).unwrap();

        let config_path = kube_dir.join("config");
        fs::write(&config_path, create_test_kube_config()).unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let mut config = load_kube_config().unwrap();
        config.current_context = "context2".to_string();

        let result = save_kube_config(&config, false);
        assert!(result.is_ok());

        let backup_path = config_path.with_extension("bak");
        assert!(!backup_path.exists());

        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("current-context: context2"));

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_get_kube_config_path_success() {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original HOME
        let original_home = std::env::var("HOME").ok();

        let temp_dir = tempdir().unwrap();
        let kube_dir = temp_dir.path().join(".kube");
        fs::create_dir_all(&kube_dir).unwrap();

        let config_path = kube_dir.join("config");
        fs::write(&config_path, create_test_kube_config()).unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let result = get_kube_config_path();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), config_path);

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    // Note: This test is challenging to implement due to platform-specific behavior
    // of home directory detection. The dirs::home_dir() function may use system calls
    // rather than just environment variables, making mocking difficult.
    // The functionality is tested via integration tests instead.
}
