mod common;

use khelp::config::operations::{load_kube_config_from, save_kube_config_to};

#[test]
fn test_rename_context() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Verify initial state
    assert!(config.contexts.iter().any(|c| c.name == "test-context"));
    assert!(!config.contexts.iter().any(|c| c.name == "renamed-context"));

    // Rename the context
    for context in &mut config.contexts {
        if context.name == "test-context" {
            context.name = "renamed-context".to_string();
            break;
        }
    }

    // Update current-context if needed
    if config.current_context == "test-context" {
        config.current_context = "renamed-context".to_string();
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify the rename persisted
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert!(
        !reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "test-context")
    );
    assert!(
        reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "renamed-context")
    );
    assert_eq!(reloaded_config.current_context, "renamed-context");
}

#[test]
fn test_rename_preserves_context_details() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Get original context details
    let original_context = config
        .contexts
        .iter()
        .find(|c| c.name == "test-context")
        .expect("test-context not found")
        .clone();

    // Rename the context
    for context in &mut config.contexts {
        if context.name == "test-context" {
            context.name = "new-name".to_string();
            break;
        }
    }

    if config.current_context == "test-context" {
        config.current_context = "new-name".to_string();
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify all context details are preserved
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    let renamed_context = reloaded_config
        .contexts
        .iter()
        .find(|c| c.name == "new-name")
        .expect("new-name context not found");

    assert_eq!(
        renamed_context.context.cluster,
        original_context.context.cluster
    );
    assert_eq!(renamed_context.context.user, original_context.context.user);
    assert_eq!(
        renamed_context.context.namespace,
        original_context.context.namespace
    );
}

#[test]
fn test_rename_non_current_context() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Ensure we're starting with test-context as current
    assert_eq!(config.current_context, "test-context");

    // Rename the second-context (not the current one)
    for context in &mut config.contexts {
        if context.name == "second-context" {
            context.name = "renamed-second".to_string();
            break;
        }
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify current-context is unchanged
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.current_context, "test-context");
    assert!(
        reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "renamed-second")
    );
    assert!(
        !reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "second-context")
    );
}

#[test]
fn test_rename_preserves_all_contexts() {
    let test_config = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2", "ctx3"]);
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let original_count = config.contexts.len();

    // Rename ctx2
    for context in &mut config.contexts {
        if context.name == "ctx2" {
            context.name = "ctx2-renamed".to_string();
            break;
        }
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify all contexts are still there
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.contexts.len(), original_count);
    assert!(reloaded_config.contexts.iter().any(|c| c.name == "ctx1"));
    assert!(
        reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "ctx2-renamed")
    );
    assert!(reloaded_config.contexts.iter().any(|c| c.name == "ctx3"));
}

#[test]
fn test_rename_updates_current_context() {
    let test_config = common::TestKubeConfig::with_contexts(&["dev", "staging", "prod"]);
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Set staging as current
    config.current_context = "staging".to_string();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload to ensure it's persisted
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");
    assert_eq!(config.current_context, "staging");

    // Rename staging to stage
    for context in &mut config.contexts {
        if context.name == "staging" {
            context.name = "stage".to_string();
            break;
        }
    }

    if config.current_context == "staging" {
        config.current_context = "stage".to_string();
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify current-context was updated
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.current_context, "stage");
    assert!(reloaded_config.contexts.iter().any(|c| c.name == "stage"));
    assert!(!reloaded_config.contexts.iter().any(|c| c.name == "staging"));
}

#[test]
fn test_rename_preserves_clusters_and_users() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let original_clusters = config.clusters.clone();
    let original_users = config.users.clone();

    // Rename a context
    for context in &mut config.contexts {
        if context.name == "test-context" {
            context.name = "renamed".to_string();
            break;
        }
    }

    if config.current_context == "test-context" {
        config.current_context = "renamed".to_string();
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify clusters and users are unchanged
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.clusters.len(), original_clusters.len());
    assert_eq!(reloaded_config.users.len(), original_users.len());

    for (original, reloaded) in original_clusters
        .iter()
        .zip(reloaded_config.clusters.iter())
    {
        assert_eq!(original.name, reloaded.name);
    }

    for (original, reloaded) in original_users.iter().zip(reloaded_config.users.iter()) {
        assert_eq!(original.name, reloaded.name);
    }
}

#[test]
fn test_isolation_rename_doesnt_affect_other_instance() {
    let test_config1 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);
    let test_config2 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);

    // Rename context in first instance
    let mut config1 = load_kube_config_from(test_config1.path()).expect("Failed to load config1");
    for context in &mut config1.contexts {
        if context.name == "ctx1" {
            context.name = "ctx1-renamed".to_string();
            break;
        }
    }
    save_kube_config_to(&config1, test_config1.path()).expect("Failed to save config1");

    // Verify second instance is unaffected
    let config2 = load_kube_config_from(test_config2.path()).expect("Failed to load config2");
    assert!(config2.contexts.iter().any(|c| c.name == "ctx1"));
    assert!(!config2.contexts.iter().any(|c| c.name == "ctx1-renamed"));
}
