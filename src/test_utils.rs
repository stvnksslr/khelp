use crate::config::kubernetes::*;

/// Create a test KubeConfig with multiple contexts for testing
pub fn create_test_kube_config() -> KubeConfig {
    KubeConfig {
        api_version: "v1".to_string(),
        kind: "Config".to_string(),
        current_context: "context1".to_string(),
        preferences: Preferences {},
        clusters: vec![
            ClusterEntry {
                name: "cluster1".to_string(),
                cluster: ClusterData {
                    server: "https://cluster1.example.com".to_string(),
                    certificate_authority_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    certificate_authority: None,
                    insecure_skip_tls_verify: Some(false),
                },
            },
            ClusterEntry {
                name: "cluster2".to_string(),
                cluster: ClusterData {
                    server: "https://cluster2.example.com".to_string(),
                    certificate_authority_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    certificate_authority: None,
                    insecure_skip_tls_verify: None,
                },
            },
        ],
        contexts: vec![
            ContextEntry {
                name: "context1".to_string(),
                context: ContextData {
                    cluster: "cluster1".to_string(),
                    user: "user1".to_string(),
                    namespace: Some("default".to_string()),
                },
            },
            ContextEntry {
                name: "context2".to_string(),
                context: ContextData {
                    cluster: "cluster2".to_string(),
                    user: "user2".to_string(),
                    namespace: None,
                },
            },
        ],
        users: vec![
            UserEntry {
                name: "user1".to_string(),
                user: UserData {
                    client_certificate_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    client_key_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    token: None,
                    username: None,
                    password: None,
                    exec: None,
                },
            },
            UserEntry {
                name: "user2".to_string(),
                user: UserData {
                    client_certificate_data: None,
                    client_key_data: None,
                    token: Some("eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9...".to_string()),
                    username: None,
                    password: None,
                    exec: None,
                },
            },
        ],
    }
}

/// Create a minimal test KubeConfig for basic testing
pub fn create_minimal_test_kube_config() -> KubeConfig {
    KubeConfig {
        api_version: "v1".to_string(),
        kind: "Config".to_string(),
        current_context: "test-context".to_string(),
        preferences: Preferences {},
        clusters: vec![ClusterEntry {
            name: "test-cluster".to_string(),
            cluster: ClusterData {
                server: "https://test.example.com".to_string(),
                certificate_authority_data: Some("test-cert-data".to_string()),
                certificate_authority: None,
                insecure_skip_tls_verify: None,
            },
        }],
        contexts: vec![ContextEntry {
            name: "test-context".to_string(),
            context: ContextData {
                cluster: "test-cluster".to_string(),
                user: "test-user".to_string(),
                namespace: Some("test-namespace".to_string()),
            },
        }],
        users: vec![UserEntry {
            name: "test-user".to_string(),
            user: UserData {
                client_certificate_data: Some("test-client-cert".to_string()),
                client_key_data: Some("test-client-key".to_string()),
                token: None,
                username: None,
                password: None,
                exec: None,
            },
        }],
    }
}

/// Convert a KubeConfig to YAML string for testing
pub fn kube_config_to_yaml(config: &KubeConfig) -> String {
    serde_yaml::to_string(config).expect("Failed to serialize test config to YAML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_kube_config() {
        let config = create_test_kube_config();

        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Config");
        assert_eq!(config.current_context, "context1");
        assert_eq!(config.contexts.len(), 2);
        assert_eq!(config.clusters.len(), 2);
        assert_eq!(config.users.len(), 2);
    }

    #[test]
    fn test_create_minimal_test_kube_config() {
        let config = create_minimal_test_kube_config();

        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Config");
        assert_eq!(config.current_context, "test-context");
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.clusters.len(), 1);
        assert_eq!(config.users.len(), 1);
    }

    #[test]
    fn test_kube_config_to_yaml() {
        let config = create_minimal_test_kube_config();
        let yaml = kube_config_to_yaml(&config);

        assert!(yaml.contains("apiVersion: v1"));
        assert!(yaml.contains("kind: Config"));
        assert!(yaml.contains("current-context: test-context"));
        assert!(yaml.contains("test-cluster"));
        assert!(yaml.contains("test-user"));
    }
}
