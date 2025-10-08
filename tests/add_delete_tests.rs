mod common;

use khelp::config::operations::{load_kube_config_from, save_kube_config_to};

#[test]
fn test_add_external_context() {
    let test_config = common::TestKubeConfig::new();
    let external_path = test_config.create_external_config("new-context");

    // Load both configs
    let mut main_config =
        load_kube_config_from(test_config.path()).expect("Failed to load main config");
    let external_config =
        load_kube_config_from(&external_path).expect("Failed to load external config");

    let original_context_count = main_config.contexts.len();

    // Simulate add operation: merge external config into main config
    for context in external_config.contexts {
        if !main_config.contexts.iter().any(|c| c.name == context.name) {
            main_config.contexts.push(context);
        }
    }
    for cluster in external_config.clusters {
        if !main_config.clusters.iter().any(|c| c.name == cluster.name) {
            main_config.clusters.push(cluster);
        }
    }
    for user in external_config.users {
        if !main_config.users.iter().any(|u| u.name == user.name) {
            main_config.users.push(user);
        }
    }

    save_kube_config_to(&main_config, test_config.path()).expect("Failed to save merged config");

    // Verify the context was added
    let merged_config = load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(merged_config.contexts.len(), original_context_count + 1);
    assert!(
        merged_config
            .contexts
            .iter()
            .any(|c| c.name == "new-context")
    );
}

#[test]
fn test_add_duplicate_context_skipped() {
    let test_config = common::TestKubeConfig::new();
    let external_path = test_config.create_external_config("test-context"); // Same name as existing

    let mut main_config =
        load_kube_config_from(test_config.path()).expect("Failed to load main config");
    let external_config =
        load_kube_config_from(&external_path).expect("Failed to load external config");

    let original_context_count = main_config.contexts.len();

    // Simulate add with skip logic: don't add if name exists
    for context in external_config.contexts {
        if !main_config.contexts.iter().any(|c| c.name == context.name) {
            main_config.contexts.push(context);
        }
    }

    // Verify no context was added (duplicate skipped)
    assert_eq!(main_config.contexts.len(), original_context_count);
}

#[test]
fn test_add_multiple_external_contexts() {
    let test_config = common::TestKubeConfig::with_single_context("original");
    let external1 = test_config.create_external_config("external1");
    let external2 = test_config.create_external_config("external2");

    let mut main_config =
        load_kube_config_from(test_config.path()).expect("Failed to load main config");

    // Add first external config
    let external_config1 =
        load_kube_config_from(&external1).expect("Failed to load external config 1");
    for context in external_config1.contexts {
        if !main_config.contexts.iter().any(|c| c.name == context.name) {
            main_config.contexts.push(context);
        }
    }
    for cluster in external_config1.clusters {
        if !main_config.clusters.iter().any(|c| c.name == cluster.name) {
            main_config.clusters.push(cluster);
        }
    }
    for user in external_config1.users {
        if !main_config.users.iter().any(|u| u.name == user.name) {
            main_config.users.push(user);
        }
    }

    // Add second external config
    let external_config2 =
        load_kube_config_from(&external2).expect("Failed to load external config 2");
    for context in external_config2.contexts {
        if !main_config.contexts.iter().any(|c| c.name == context.name) {
            main_config.contexts.push(context);
        }
    }
    for cluster in external_config2.clusters {
        if !main_config.clusters.iter().any(|c| c.name == cluster.name) {
            main_config.clusters.push(cluster);
        }
    }
    for user in external_config2.users {
        if !main_config.users.iter().any(|u| u.name == user.name) {
            main_config.users.push(user);
        }
    }

    save_kube_config_to(&main_config, test_config.path()).expect("Failed to save merged config");

    // Verify all contexts were added
    let final_config = load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(final_config.contexts.len(), 3); // original + external1 + external2
    assert!(final_config.contexts.iter().any(|c| c.name == "original"));
    assert!(final_config.contexts.iter().any(|c| c.name == "external1"));
    assert!(final_config.contexts.iter().any(|c| c.name == "external2"));
}

#[test]
fn test_delete_context() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let original_count = config.contexts.len();
    assert!(original_count > 1, "Need at least 2 contexts for this test");

    // Delete the second context
    config.contexts.retain(|c| c.name != "second-context");

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify deletion
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.contexts.len(), original_count - 1);
    assert!(
        !reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "second-context")
    );
}

#[test]
fn test_delete_context_with_orphaned_cleanup() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Delete a context
    let deleted_context = config
        .contexts
        .iter()
        .find(|c| c.name == "second-context")
        .expect("Context not found")
        .clone();

    config.contexts.retain(|c| c.name != "second-context");

    // Find and remove orphaned cluster and user
    let cluster_to_remove = deleted_context.context.cluster;
    let user_to_remove = deleted_context.context.user;

    // Check if cluster is referenced by any remaining context
    let cluster_referenced = config
        .contexts
        .iter()
        .any(|c| c.context.cluster == cluster_to_remove);
    if !cluster_referenced {
        config.clusters.retain(|c| c.name != cluster_to_remove);
    }

    // Check if user is referenced by any remaining context
    let user_referenced = config
        .contexts
        .iter()
        .any(|c| c.context.user == user_to_remove);
    if !user_referenced {
        config.users.retain(|u| u.name != user_to_remove);
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify cleanup
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert!(
        !reloaded_config
            .contexts
            .iter()
            .any(|c| c.name == "second-context")
    );
    assert!(
        !reloaded_config
            .clusters
            .iter()
            .any(|c| c.name == cluster_to_remove)
    );
    assert!(
        !reloaded_config
            .users
            .iter()
            .any(|u| u.name == user_to_remove)
    );
}

#[test]
fn test_delete_current_context_switches_to_another() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let original_current = config.current_context.clone();
    assert_eq!(original_current, "test-context");

    // Delete the current context
    config.contexts.retain(|c| c.name != original_current);

    // Switch to another available context
    if !config.contexts.is_empty() {
        config.current_context = config.contexts[0].name.clone();
    } else {
        config.current_context = String::new();
    }

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify switch happened
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_ne!(reloaded_config.current_context, original_current);
    assert!(!reloaded_config.current_context.is_empty());
}

#[test]
fn test_delete_all_contexts_except_one() {
    let test_config = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2", "ctx3"]);
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Delete all except ctx2
    config.contexts.retain(|c| c.name == "ctx2");
    config.current_context = "ctx2".to_string();

    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Verify only one context remains
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.contexts.len(), 1);
    assert_eq!(reloaded_config.contexts[0].name, "ctx2");
    assert_eq!(reloaded_config.current_context, "ctx2");
}

#[test]
fn test_isolation_add_doesnt_affect_other_instance() {
    let test_config1 = common::TestKubeConfig::with_single_context("ctx1");
    let test_config2 = common::TestKubeConfig::with_single_context("ctx1");

    // Add a context to first instance
    let external = test_config1.create_external_config("new-ctx");
    let mut config1 = load_kube_config_from(test_config1.path()).expect("Failed to load config1");
    let external_config = load_kube_config_from(&external).expect("Failed to load external");

    for context in external_config.contexts {
        config1.contexts.push(context);
    }
    for cluster in external_config.clusters {
        config1.clusters.push(cluster);
    }
    for user in external_config.users {
        config1.users.push(user);
    }

    save_kube_config_to(&config1, test_config1.path()).expect("Failed to save config1");

    // Verify second instance is unaffected
    let config2 = load_kube_config_from(test_config2.path()).expect("Failed to load config2");
    assert_eq!(config2.contexts.len(), 1);
    assert_eq!(config2.contexts[0].name, "ctx1");
}

#[test]
fn test_isolation_delete_doesnt_affect_other_instance() {
    let test_config1 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);
    let test_config2 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);

    // Delete from first instance
    let mut config1 = load_kube_config_from(test_config1.path()).expect("Failed to load config1");
    config1.contexts.retain(|c| c.name != "ctx2");
    save_kube_config_to(&config1, test_config1.path()).expect("Failed to save config1");

    // Verify second instance is unaffected
    let config2 = load_kube_config_from(test_config2.path()).expect("Failed to load config2");
    assert_eq!(config2.contexts.len(), 2);
    assert!(config2.contexts.iter().any(|c| c.name == "ctx2"));
}
