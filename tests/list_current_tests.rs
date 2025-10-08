mod common;

use khelp::config::operations::load_kube_config_from;

#[test]
fn test_load_config_with_multiple_contexts() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    assert_eq!(config.contexts.len(), 2);
    assert_eq!(config.current_context, "test-context");

    // Verify first context
    let first_context = config
        .contexts
        .iter()
        .find(|c| c.name == "test-context")
        .expect("test-context not found");
    assert_eq!(first_context.context.cluster, "test-cluster");
    assert_eq!(first_context.context.user, "test-user");

    // Verify second context
    let second_context = config
        .contexts
        .iter()
        .find(|c| c.name == "second-context")
        .expect("second-context not found");
    assert_eq!(second_context.context.cluster, "second-cluster");
    assert_eq!(second_context.context.user, "second-user");
}

#[test]
fn test_load_config_with_single_context() {
    let test_config = common::TestKubeConfig::with_single_context("my-context");
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    assert_eq!(config.contexts.len(), 1);
    assert_eq!(config.current_context, "my-context");
    assert_eq!(config.contexts[0].name, "my-context");
    assert_eq!(config.contexts[0].context.cluster, "my-context-cluster");
    assert_eq!(config.contexts[0].context.user, "my-context-user");
}

#[test]
fn test_current_context_points_to_valid_context() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Verify current context exists in the list of contexts
    let current_exists = config
        .contexts
        .iter()
        .any(|c| c.name == config.current_context);
    assert!(
        current_exists,
        "Current context '{}' should exist in contexts list",
        config.current_context
    );
}

#[test]
fn test_clusters_match_context_references() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Verify all context cluster references exist in clusters list
    for context in &config.contexts {
        let cluster_exists = config
            .clusters
            .iter()
            .any(|c| c.name == context.context.cluster);
        assert!(
            cluster_exists,
            "Cluster '{}' referenced by context '{}' should exist",
            context.context.cluster, context.name
        );
    }
}

#[test]
fn test_users_match_context_references() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Verify all context user references exist in users list
    for context in &config.contexts {
        let user_exists = config.users.iter().any(|u| u.name == context.context.user);
        assert!(
            user_exists,
            "User '{}' referenced by context '{}' should exist",
            context.context.user, context.name
        );
    }
}

#[test]
fn test_list_contexts_with_many() {
    let context_names = vec!["dev", "staging", "production", "test"];
    let test_config = common::TestKubeConfig::with_contexts(&context_names);
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    assert_eq!(config.contexts.len(), 4);

    for name in context_names {
        let context_exists = config.contexts.iter().any(|c| c.name == name);
        assert!(context_exists, "Context '{}' should exist", name);
    }
}

#[test]
fn test_current_context_details() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let current_context = config
        .contexts
        .iter()
        .find(|c| c.name == config.current_context)
        .expect("Current context should exist");

    // Verify current context has valid details
    assert!(!current_context.context.cluster.is_empty());
    assert!(!current_context.context.user.is_empty());

    // Verify namespace is set
    assert_eq!(
        current_context.context.namespace,
        Some("default".to_string())
    );
}

#[test]
fn test_isolation_multiple_test_instances() {
    // Create two separate test configs to verify they don't interfere
    let test_config1 = common::TestKubeConfig::with_single_context("context1");
    let test_config2 = common::TestKubeConfig::with_single_context("context2");

    let config1 = load_kube_config_from(test_config1.path()).expect("Failed to load config1");
    let config2 = load_kube_config_from(test_config2.path()).expect("Failed to load config2");

    assert_eq!(config1.current_context, "context1");
    assert_eq!(config2.current_context, "context2");

    // Verify they're truly isolated
    assert_ne!(
        test_config1.path(),
        test_config2.path(),
        "Test configs should use different paths"
    );
}
