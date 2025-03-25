use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KubeConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub clusters: Vec<ClusterEntry>,
    pub contexts: Vec<ContextEntry>,
    #[serde(rename = "current-context")]
    pub current_context: String,
    pub kind: String,
    pub preferences: Preferences,
    pub users: Vec<UserEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterEntry {
    pub cluster: ClusterData,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterData {
    #[serde(
        rename = "certificate-authority-data",
        skip_serializing_if = "Option::is_none"
    )]
    pub certificate_authority_data: Option<String>,
    #[serde(
        rename = "certificate-authority",
        skip_serializing_if = "Option::is_none"
    )]
    pub certificate_authority: Option<String>,
    pub server: String,
    #[serde(
        rename = "insecure-skip-tls-verify",
        skip_serializing_if = "Option::is_none"
    )]
    pub insecure_skip_tls_verify: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextEntry {
    pub context: ContextData,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextData {
    pub cluster: String,
    pub user: String,
    #[serde(rename = "namespace", skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserEntry {
    pub name: String,
    pub user: UserData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    #[serde(
        rename = "client-certificate-data",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_certificate_data: Option<String>,
    #[serde(rename = "client-key-data", skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "exec", skip_serializing_if = "Option::is_none")]
    pub exec: Option<ExecConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVar>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences {}
