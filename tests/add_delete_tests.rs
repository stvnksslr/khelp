mod common;

use khelp::config::kubernetes::KubeConfig;
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

#[test]
fn test_add_to_empty_config() {
    // Create an empty config
    let test_config = common::TestKubeConfig::empty();

    // Create an external config to add (keep temp in scope)
    let external_temp = common::TestKubeConfig::with_single_context("external");
    let external_path = external_temp.create_external_config("external");

    // Loading empty config should fail
    let load_result = load_kube_config_from(test_config.path());
    assert!(load_result.is_err(), "Should fail to load empty config");
    let error_msg = load_result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Config file is empty"),
        "Error should mention empty config: {}",
        error_msg
    );

    // But we can use a default config and save external content to it
    let external_config =
        load_kube_config_from(&external_path).expect("Failed to load external config");

    // Start with default config and merge external
    let mut main_config = KubeConfig::default();
    for context in external_config.contexts {
        main_config.contexts.push(context);
    }
    for cluster in external_config.clusters {
        main_config.clusters.push(cluster);
    }
    for user in external_config.users {
        main_config.users.push(user);
    }
    if !main_config.contexts.is_empty() {
        main_config.current_context = main_config.contexts[0].name.clone();
    }

    save_kube_config_to(&main_config, test_config.path()).expect("Failed to save config");

    // Verify the config can now be loaded
    let loaded = load_kube_config_from(test_config.path()).expect("Failed to load saved config");
    assert_eq!(loaded.contexts.len(), 1);
    assert_eq!(loaded.contexts[0].name, "external");
    assert_eq!(loaded.current_context, "external");
}

#[test]
fn test_empty_config_error_message() {
    let test_config = common::TestKubeConfig::empty();

    let result = load_kube_config_from(test_config.path());
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Config file is empty"));
    assert!(error_msg.contains("apiVersion"));
    assert!(error_msg.contains("kind"));
}

#[test]
fn test_whitespace_only_config_error_message() {
    let test_config = common::TestKubeConfig::with_content("   \n\n  \t  \n");

    let result = load_kube_config_from(test_config.path());
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Config file is empty"),
        "Whitespace-only should be treated as empty: {}",
        error_msg
    );
}

#[test]
fn test_missing_api_version_uses_default() {
    let config_without_api_version = r#"kind: Config
clusters: []
contexts: []
users: []
current-context: ""
preferences: {}
"#;
    let test_config = common::TestKubeConfig::with_content(config_without_api_version);

    let result = load_kube_config_from(test_config.path());
    assert!(result.is_ok(), "Should succeed with default apiVersion");

    let config = result.unwrap();
    assert_eq!(config.api_version, "v1", "Should default to v1");
}

#[test]
fn test_default_kubeconfig_is_valid() {
    let config = KubeConfig::default();

    assert_eq!(config.api_version, "v1");
    assert_eq!(config.kind, "Config");
    assert!(config.clusters.is_empty());
    assert!(config.contexts.is_empty());
    assert!(config.users.is_empty());
    assert!(config.current_context.is_empty());

    // Verify it can be serialized and deserialized
    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize");
    let parsed: KubeConfig = serde_yaml::from_str(&yaml).expect("Failed to deserialize");
    assert_eq!(parsed.api_version, config.api_version);
    assert_eq!(parsed.kind, config.kind);
}

#[test]
fn test_save_and_load_default_config() {
    let test_config = common::TestKubeConfig::empty();

    // Save a default config to the empty file
    let config = KubeConfig::default();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save default config");

    // Verify it can be loaded
    let loaded = load_kube_config_from(test_config.path()).expect("Failed to load config");
    assert_eq!(loaded.api_version, "v1");
    assert_eq!(loaded.kind, "Config");
    assert!(loaded.clusters.is_empty());
    assert!(loaded.contexts.is_empty());
    assert!(loaded.users.is_empty());
}
