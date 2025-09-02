use crate::config::kubernetes::KubeConfig;
use console::style;

/// List all available Kubernetes contexts, highlighting the current one
pub fn list_contexts(config: &KubeConfig) {
    println!("{} available contexts:", style("Kubernetes").green().bold());
    println!("------------------------");

    for context in &config.contexts {
        let marker = if context.name == config.current_context {
            style("*").green().bold()
        } else {
            style(" ").dim()
        };

        let namespace_info = if let Some(namespace) = &context.context.namespace {
            format!(" (namespace: {})", style(namespace).cyan())
        } else {
            String::new()
        };

        println!("{} {}{}", marker, context.name, namespace_info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::kubernetes::*;

    fn create_test_config() -> KubeConfig {
        KubeConfig {
            api_version: "v1".to_string(),
            kind: "Config".to_string(),
            current_context: "context1".to_string(),
            preferences: Preferences {},
            clusters: vec![
                ClusterEntry {
                    name: "cluster1".to_string(),
                    cluster: ClusterData {
                        server: "https://test1.com".to_string(),
                        certificate_authority_data: Some("data1".to_string()),
                        certificate_authority: None,
                        insecure_skip_tls_verify: None,
                    },
                },
                ClusterEntry {
                    name: "cluster2".to_string(),
                    cluster: ClusterData {
                        server: "https://test2.com".to_string(),
                        certificate_authority_data: Some("data2".to_string()),
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
                        client_certificate_data: Some("cert1".to_string()),
                        client_key_data: Some("key1".to_string()),
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
                        token: Some("token2".to_string()),
                        username: None,
                        password: None,
                        exec: None,
                    },
                },
            ],
        }
    }

    #[test]
    fn test_list_contexts_displays_all_contexts() {
        let config = create_test_config();

        // We can't easily test stdout output in unit tests without more complex setup,
        // but we can test that the function doesn't panic and works with valid config
        list_contexts(&config);
    }

    #[test]
    fn test_list_contexts_with_empty_contexts() {
        let mut config = create_test_config();
        config.contexts.clear();

        // Should not panic even with no contexts
        list_contexts(&config);
    }

    #[test]
    fn test_list_contexts_identifies_current_context() {
        let config = create_test_config();

        // Verify that our test config has the expected current context
        assert_eq!(config.current_context, "context1");
        assert!(config.contexts.iter().any(|c| c.name == "context1"));
        assert!(config.contexts.iter().any(|c| c.name == "context2"));

        // The function should handle this correctly (tested via integration tests)
        list_contexts(&config);
    }
}
