mod common;

use khelp::config::operations::{load_kube_config_from, save_kube_config_to};

#[test]
fn test_switch_context() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    assert_eq!(config.current_context, "test-context");

    // Switch to second context
    config.current_context = "second-context".to_string();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify the switch persisted
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.current_context, "second-context");
}

#[test]
fn test_switch_preserves_all_contexts() {
    let test_config = common::TestKubeConfig::new();
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    let original_contexts_count = config.contexts.len();
    let original_clusters_count = config.clusters.len();
    let original_users_count = config.users.len();

    // Switch context
    config.current_context = "second-context".to_string();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

    // Reload and verify nothing was lost
    let reloaded_config =
        load_kube_config_from(test_config.path()).expect("Failed to reload config");
    assert_eq!(reloaded_config.contexts.len(), original_contexts_count);
    assert_eq!(reloaded_config.clusters.len(), original_clusters_count);
    assert_eq!(reloaded_config.users.len(), original_users_count);
}

#[test]
fn test_switch_between_multiple_contexts() {
    let context_names = vec!["dev", "staging", "production"];
    let test_config = common::TestKubeConfig::with_contexts(&context_names);
    let mut config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Switch through each context
    for name in &context_names {
        config.current_context = name.to_string();
        save_kube_config_to(&config, test_config.path()).expect("Failed to save config");

        let reloaded_config =
            load_kube_config_from(test_config.path()).expect("Failed to reload config");
        assert_eq!(&reloaded_config.current_context, name);
    }
}

#[test]
fn test_export_single_context() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Find the test-context
    let context_to_export = config
        .contexts
        .iter()
        .find(|c| c.name == "test-context")
        .expect("Context not found");

    // Create an exported config with just this context
    let exported_config = khelp::config::kubernetes::KubeConfig {
        api_version: config.api_version.clone(),
        clusters: config
            .clusters
            .iter()
            .filter(|c| c.name == context_to_export.context.cluster)
            .cloned()
            .collect(),
        contexts: vec![context_to_export.clone()],
        current_context: context_to_export.name.clone(),
        kind: config.kind.clone(),
        preferences: config.preferences.clone(),
        users: config
            .users
            .iter()
            .filter(|u| u.name == context_to_export.context.user)
            .cloned()
            .collect(),
    };

    // Verify exported config only has the relevant parts
    assert_eq!(exported_config.contexts.len(), 1);
    assert_eq!(exported_config.clusters.len(), 1);
    assert_eq!(exported_config.users.len(), 1);
    assert_eq!(exported_config.current_context, "test-context");
}

#[test]
fn test_export_config_can_be_saved() {
    let test_config = common::TestKubeConfig::new();
    let config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Create export path
    let export_path = test_config.temp_dir.path().join("exported.yaml");

    // Save the entire config to the export path
    save_kube_config_to(&config, &export_path).expect("Failed to export config");

    // Verify the exported file can be loaded
    let exported_config = load_kube_config_from(&export_path).expect("Failed to load export");

    assert_eq!(exported_config.contexts.len(), config.contexts.len());
    assert_eq!(exported_config.current_context, config.current_context);
}

#[test]
fn test_export_partial_config() {
    let test_config = common::TestKubeConfig::with_contexts(&["dev", "staging", "production"]);
    let original_config = load_kube_config_from(test_config.path()).expect("Failed to load config");

    // Export just the "staging" context
    let staging_context = original_config
        .contexts
        .iter()
        .find(|c| c.name == "staging")
        .expect("staging context not found")
        .clone();

    let partial_config = khelp::config::kubernetes::KubeConfig {
        api_version: original_config.api_version.clone(),
        clusters: original_config
            .clusters
            .iter()
            .filter(|c| c.name == staging_context.context.cluster)
            .cloned()
            .collect(),
        contexts: vec![staging_context.clone()],
        current_context: staging_context.name.clone(),
        kind: original_config.kind.clone(),
        preferences: original_config.preferences.clone(),
        users: original_config
            .users
            .iter()
            .filter(|u| u.name == staging_context.context.user)
            .cloned()
            .collect(),
    };

    let export_path = test_config.temp_dir.path().join("staging-export.yaml");
    save_kube_config_to(&partial_config, &export_path).expect("Failed to save partial config");

    // Verify the partial config
    let reloaded = load_kube_config_from(&export_path).expect("Failed to reload partial config");
    assert_eq!(reloaded.contexts.len(), 1);
    assert_eq!(reloaded.contexts[0].name, "staging");
    assert_eq!(reloaded.current_context, "staging");
}

#[test]
fn test_isolation_switch_doesnt_affect_other_instance() {
    let test_config1 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);
    let test_config2 = common::TestKubeConfig::with_contexts(&["ctx1", "ctx2"]);

    // Switch context in first instance
    let mut config1 = load_kube_config_from(test_config1.path()).expect("Failed to load config1");
    config1.current_context = "ctx2".to_string();
    save_kube_config_to(&config1, test_config1.path()).expect("Failed to save config1");

    // Verify second instance is unaffected
    let config2 = load_kube_config_from(test_config2.path()).expect("Failed to load config2");
    assert_eq!(config2.current_context, "ctx1"); // Should still be the first one
}

#[test]
fn test_round_trip_maintains_context_details() {
    let test_config = common::TestKubeConfig::new();
    let original_config =
        load_kube_config_from(test_config.path()).expect("Failed to load original config");

    // Get details of the current context before switch
    let original_context = original_config
        .contexts
        .iter()
        .find(|c| c.name == "test-context")
        .expect("test-context not found")
        .clone();

    // Switch and switch back
    let mut config = original_config.clone();
    config.current_context = "second-context".to_string();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save");

    config.current_context = "test-context".to_string();
    save_kube_config_to(&config, test_config.path()).expect("Failed to save");

    // Reload and verify context details are unchanged
    let final_config = load_kube_config_from(test_config.path()).expect("Failed to reload config");
    let final_context = final_config
        .contexts
        .iter()
        .find(|c| c.name == "test-context")
        .expect("test-context not found");

    assert_eq!(
        final_context.context.cluster,
        original_context.context.cluster
    );
    assert_eq!(final_context.context.user, original_context.context.user);
    assert_eq!(
        final_context.context.namespace,
        original_context.context.namespace
    );
}
