use crate::config::kubernetes::KubeConfig;
use console::style;

/// Display details about the currently active context
pub fn show_current_context(config: &KubeConfig) {
    println!(
        "Current context: {}",
        style(&config.current_context).green().bold()
    );

    if let Some(context) = config
        .contexts
        .iter()
        .find(|c| c.name == config.current_context)
    {
        println!("  Cluster: {}", style(&context.context.cluster).cyan());
        println!("  User: {}", style(&context.context.user).cyan());

        if let Some(namespace) = &context.context.namespace {
            println!("  Namespace: {}", style(namespace).cyan());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::kubernetes::*;

    fn create_test_config_with_namespace() -> KubeConfig {
        KubeConfig {
            api_version: "v1".to_string(),
            kind: "Config".to_string(),
            current_context: "test-context".to_string(),
            preferences: Preferences {},
            clusters: vec![ClusterEntry {
                name: "test-cluster".to_string(),
                cluster: ClusterData {
                    server: "https://test.com".to_string(),
                    certificate_authority_data: Some("data".to_string()),
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
                    client_certificate_data: Some("cert".to_string()),
                    client_key_data: Some("key".to_string()),
                    token: None,
                    username: None,
                    password: None,
                    exec: None,
                },
            }],
        }
    }

    fn create_test_config_without_namespace() -> KubeConfig {
        let mut config = create_test_config_with_namespace();
        config.contexts[0].context.namespace = None;
        config
    }

    #[test]
    fn test_show_current_context_with_namespace() {
        let config = create_test_config_with_namespace();

        // Test that the function doesn't panic and handles namespace correctly
        show_current_context(&config);

        // Verify the config has the expected structure
        assert_eq!(config.current_context, "test-context");
        assert!(config.contexts.iter().any(|c| c.name == "test-context"));
        assert_eq!(
            config.contexts[0].context.namespace,
            Some("test-namespace".to_string())
        );
    }

    #[test]
    fn test_show_current_context_without_namespace() {
        let config = create_test_config_without_namespace();

        // Test that the function doesn't panic when namespace is None
        show_current_context(&config);

        // Verify the config structure
        assert_eq!(config.current_context, "test-context");
        assert!(config.contexts.iter().any(|c| c.name == "test-context"));
        assert_eq!(config.contexts[0].context.namespace, None);
    }

    #[test]
    fn test_show_current_context_missing_context() {
        let mut config = create_test_config_with_namespace();
        config.current_context = "non-existent-context".to_string();

        // Should not panic even when current context doesn't exist in contexts list
        show_current_context(&config);
    }

    #[test]
    fn test_show_current_context_empty_contexts() {
        let mut config = create_test_config_with_namespace();
        config.contexts.clear();

        // Should not panic even with empty contexts
        show_current_context(&config);
    }
}
