use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn setup_test_kube_config() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let kube_dir = temp_dir.path().join(".kube");
    fs::create_dir_all(&kube_dir).unwrap();

    let config_content = r#"apiVersion: v1
kind: Config
current-context: context1
preferences: {}
clusters:
- name: cluster1
  cluster:
    server: https://cluster1.example.com
    certificate-authority-data: LS0tLS1CRUdJTi...
- name: cluster2
  cluster:
    server: https://cluster2.example.com
    certificate-authority-data: LS0tLS1CRUdJTi...
contexts:
- name: context1
  context:
    cluster: cluster1
    user: user1
    namespace: default
- name: context2
  context:
    cluster: cluster2
    user: user2
users:
- name: user1
  user:
    client-certificate-data: LS0tLS1CRUdJTi...
    client-key-data: LS0tLS1CRUdJTi...
- name: user2
  user:
    token: eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9..."#;

    let config_path = kube_dir.join("config");
    fs::write(&config_path, config_content).unwrap();

    temp_dir
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "A tool to manage Kubernetes contexts",
    ));
}

#[test]
fn test_list_command_with_config() {
    let temp_dir = setup_test_kube_config();

    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("list").env("HOME", temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Kubernetes"))
        .stdout(predicate::str::contains("context1"))
        .stdout(predicate::str::contains("context2"));
}

#[test]
fn test_current_command_with_config() {
    let temp_dir = setup_test_kube_config();

    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("current").env("HOME", temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Current context: context1"))
        .stdout(predicate::str::contains("Cluster: cluster1"))
        .stdout(predicate::str::contains("User: user1"));
}

#[test]
fn test_list_command_no_config() {
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("list").env("HOME", temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Kubernetes config file not found"));
}

#[test]
fn test_current_command_no_config() {
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("current").env("HOME", temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Kubernetes config file not found"));
}

#[test]
fn test_completions_command() {
    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("completions").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("_khelp"));
}

#[test]
fn test_export_command_with_config() {
    let temp_dir = setup_test_kube_config();

    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("export")
        .arg("context1")
        .env("HOME", temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("apiVersion: v1"))
        .stdout(predicate::str::contains("current-context: context1"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[cfg(feature = "self_update")]
#[test]
fn test_update_command() {
    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("update");

    cmd.assert().success();
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("khelp").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("khelp"));
}
