use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture that provides an isolated kubeconfig environment
pub struct TestKubeConfig {
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl TestKubeConfig {
    /// Creates a new test fixture with a sample kubeconfig
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("config");

        // Write a sample kubeconfig
        let sample_config = r#"apiVersion: v1
clusters:
- cluster:
    certificate-authority-data: LS0tLS1CRUdJTi
    server: https://127.0.0.1:6443
  name: test-cluster
- cluster:
    certificate-authority-data: LS0tLS1CRUdJTi
    server: https://192.168.1.100:6443
  name: second-cluster
contexts:
- context:
    cluster: test-cluster
    user: test-user
    namespace: default
  name: test-context
- context:
    cluster: second-cluster
    user: second-user
    namespace: production
  name: second-context
current-context: test-context
kind: Config
preferences: {}
users:
- name: test-user
  user:
    client-certificate-data: LS0tLS1CRUdJTi
    client-key-data: LS0tLS1CRUdJTi
- name: second-user
  user:
    token: sample-token-here
"#;

        fs::write(&config_path, sample_config).expect("Failed to write test config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Creates a test fixture with a custom kubeconfig content
    pub fn with_content(content: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("config");

        fs::write(&config_path, content).expect("Failed to write test config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Creates a test fixture with a single context
    pub fn with_single_context(context_name: &str) -> Self {
        let content = format!(
            r#"apiVersion: v1
clusters:
- cluster:
    server: https://127.0.0.1:6443
  name: {}-cluster
contexts:
- context:
    cluster: {}-cluster
    user: {}-user
  name: {}
current-context: {}
kind: Config
preferences: {{}}
users:
- name: {}-user
  user:
    token: test-token
"#,
            context_name, context_name, context_name, context_name, context_name, context_name
        );

        Self::with_content(&content)
    }

    /// Creates a test fixture with multiple contexts
    pub fn with_contexts(context_names: &[&str]) -> Self {
        if context_names.is_empty() {
            panic!("At least one context name must be provided");
        }

        let mut clusters = String::new();
        let mut contexts = String::new();
        let mut users = String::new();

        for name in context_names {
            clusters.push_str(&format!(
                r#"- cluster:
    server: https://{}.example.com:6443
  name: {}-cluster
"#,
                name, name
            ));

            contexts.push_str(&format!(
                r#"- context:
    cluster: {}-cluster
    user: {}-user
  name: {}
"#,
                name, name, name
            ));

            users.push_str(&format!(
                r#"- name: {}-user
  user:
    token: {}-token
"#,
                name, name
            ));
        }

        let content = format!(
            r#"apiVersion: v1
clusters:
{}contexts:
{}current-context: {}
kind: Config
preferences: {{}}
users:
{}"#,
            clusters, contexts, context_names[0], users
        );

        Self::with_content(&content)
    }

    /// Returns the path to the kubeconfig file
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// Creates an external kubeconfig file for testing imports
    pub fn create_external_config(&self, name: &str) -> PathBuf {
        let external_path = self.temp_dir.path().join(format!("{}.yaml", name));
        let content = format!(
            r#"apiVersion: v1
clusters:
- cluster:
    server: https://{}.external.com:6443
  name: {}-cluster
contexts:
- context:
    cluster: {}-cluster
    user: {}-user
  name: {}
current-context: {}
kind: Config
preferences: {{}}
users:
- name: {}-user
  user:
    token: external-token
"#,
            name, name, name, name, name, name, name
        );

        fs::write(&external_path, content).expect("Failed to write external config");
        external_path
    }
}

impl Default for TestKubeConfig {
    fn default() -> Self {
        Self::new()
    }
}
