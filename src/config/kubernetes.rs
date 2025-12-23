use serde::{Deserialize, Serialize};

fn default_api_version() -> String {
    "v1".to_string()
}

fn default_kind() -> String {
    "Config".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KubeConfig {
    #[serde(rename = "apiVersion", default = "default_api_version")]
    pub api_version: String,
    #[serde(default)]
    pub clusters: Vec<ClusterEntry>,
    #[serde(default)]
    pub contexts: Vec<ContextEntry>,
    #[serde(rename = "current-context", default)]
    pub current_context: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Preferences>,
    #[serde(default)]
    pub users: Vec<UserEntry>,
}

impl Default for KubeConfig {
    fn default() -> Self {
        Self {
            api_version: "v1".to_string(),
            clusters: Vec::new(),
            contexts: Vec::new(),
            current_context: String::new(),
            kind: "Config".to_string(),
            preferences: Some(Preferences {}),
            users: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterEntry {
    pub cluster: ClusterData,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    #[serde(rename = "tls-server-name", skip_serializing_if = "Option::is_none")]
    pub tls_server_name: Option<String>,
    #[serde(rename = "proxy-url", skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(
        rename = "disable-compression",
        skip_serializing_if = "Option::is_none"
    )]
    pub disable_compression: Option<bool>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserData {
    #[serde(
        rename = "client-certificate-data",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_certificate_data: Option<String>,
    #[serde(rename = "client-certificate", skip_serializing_if = "Option::is_none")]
    pub client_certificate: Option<String>,
    #[serde(rename = "client-key-data", skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(rename = "client-key", skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "tokenFile", skip_serializing_if = "Option::is_none")]
    pub token_file: Option<String>,
    #[serde(rename = "as", skip_serializing_if = "Option::is_none")]
    pub impersonate: Option<String>,
    #[serde(rename = "as-uid", skip_serializing_if = "Option::is_none")]
    pub impersonate_uid: Option<String>,
    #[serde(rename = "as-groups", skip_serializing_if = "Option::is_none")]
    pub impersonate_groups: Option<Vec<String>>,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "auth-provider", skip_serializing_if = "Option::is_none")]
    pub auth_provider: Option<AuthProviderConfig>,
    #[serde(rename = "exec", skip_serializing_if = "Option::is_none")]
    pub exec: Option<ExecConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthProviderConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExecConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVar>>,
    #[serde(rename = "installHint", skip_serializing_if = "Option::is_none")]
    pub install_hint: Option<String>,
    #[serde(rename = "provideClusterInfo", skip_serializing_if = "Option::is_none")]
    pub provide_cluster_info: Option<bool>,
    #[serde(rename = "interactiveMode", skip_serializing_if = "Option::is_none")]
    pub interactive_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Preferences {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubeconfig_serialization() {
        let config = KubeConfig {
            api_version: "v1".to_string(),
            clusters: vec![ClusterEntry {
                cluster: ClusterData {
                    certificate_authority_data: Some("test-cert".to_string()),
                    server: "https://127.0.0.1:6443".to_string(),
                    ..Default::default()
                },
                name: "test-cluster".to_string(),
            }],
            contexts: vec![ContextEntry {
                context: ContextData {
                    cluster: "test-cluster".to_string(),
                    user: "test-user".to_string(),
                    namespace: Some("default".to_string()),
                },
                name: "test-context".to_string(),
            }],
            current_context: "test-context".to_string(),
            kind: "Config".to_string(),
            preferences: None,
            users: vec![UserEntry {
                name: "test-user".to_string(),
                user: UserData {
                    client_certificate_data: Some("cert-data".to_string()),
                    client_key_data: Some("key-data".to_string()),
                    ..Default::default()
                },
            }],
        };

        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize");
        assert!(yaml.contains("apiVersion: v1"));
        assert!(yaml.contains("kind: Config"));
        assert!(yaml.contains("test-cluster"));
        assert!(yaml.contains("test-context"));
        assert!(yaml.contains("test-user"));
    }

    #[test]
    fn test_kubeconfig_deserialization() {
        let yaml = r#"
apiVersion: v1
clusters:
- cluster:
    server: https://127.0.0.1:6443
    certificate-authority-data: test-cert
  name: test-cluster
contexts:
- context:
    cluster: test-cluster
    user: test-user
    namespace: default
  name: test-context
current-context: test-context
kind: Config
preferences: {}
users:
- name: test-user
  user:
    client-certificate-data: cert-data
    client-key-data: key-data
"#;

        let config: KubeConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Config");
        assert_eq!(config.current_context, "test-context");
        assert_eq!(config.clusters.len(), 1);
        assert_eq!(config.clusters[0].name, "test-cluster");
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.contexts[0].name, "test-context");
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].name, "test-user");
    }

    #[test]
    fn test_cluster_entry_with_insecure_skip_tls() {
        let cluster = ClusterEntry {
            cluster: ClusterData {
                server: "https://insecure.example.com:6443".to_string(),
                insecure_skip_tls_verify: Some(true),
                ..Default::default()
            },
            name: "insecure-cluster".to_string(),
        };

        let yaml = serde_yaml::to_string(&cluster).expect("Failed to serialize");
        assert!(yaml.contains("insecure-skip-tls-verify: true"));
    }

    #[test]
    fn test_user_entry_with_exec_config() {
        let user = UserEntry {
            name: "exec-user".to_string(),
            user: UserData {
                exec: Some(ExecConfig {
                    api_version: "client.authentication.k8s.io/v1beta1".to_string(),
                    command: "aws".to_string(),
                    args: Some(vec!["eks".to_string(), "get-token".to_string()]),
                    env: Some(vec![EnvVar {
                        name: "AWS_PROFILE".to_string(),
                        value: "default".to_string(),
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };

        let yaml = serde_yaml::to_string(&user).expect("Failed to serialize");
        assert!(yaml.contains("exec:"));
        assert!(yaml.contains("command: aws"));
        assert!(yaml.contains("AWS_PROFILE"));
    }

    #[test]
    fn test_context_without_namespace() {
        let context = ContextEntry {
            context: ContextData {
                cluster: "test-cluster".to_string(),
                user: "test-user".to_string(),
                namespace: None,
            },
            name: "test-context".to_string(),
        };

        let yaml = serde_yaml::to_string(&context).expect("Failed to serialize");
        // namespace field should be omitted when None
        assert!(!yaml.contains("namespace:"));
    }

    #[test]
    fn test_user_with_token_auth() {
        let user = UserEntry {
            name: "token-user".to_string(),
            user: UserData {
                token: Some("bearer-token-here".to_string()),
                ..Default::default()
            },
        };

        let yaml = serde_yaml::to_string(&user).expect("Failed to serialize");
        assert!(yaml.contains("token: bearer-token-here"));
    }

    #[test]
    fn test_user_with_basic_auth() {
        let user = UserEntry {
            name: "basic-user".to_string(),
            user: UserData {
                username: Some("admin".to_string()),
                password: Some("secret".to_string()),
                ..Default::default()
            },
        };

        let yaml = serde_yaml::to_string(&user).expect("Failed to serialize");
        assert!(yaml.contains("username: admin"));
        assert!(yaml.contains("password: secret"));
    }

    #[test]
    fn test_round_trip_serialization() {
        let original_yaml = r#"
apiVersion: v1
clusters:
- cluster:
    server: https://127.0.0.1:6443
  name: test-cluster
contexts:
- context:
    cluster: test-cluster
    user: test-user
  name: test-context
current-context: test-context
kind: Config
preferences: {}
users:
- name: test-user
  user:
    token: test-token
"#;

        let config: KubeConfig =
            serde_yaml::from_str(original_yaml).expect("Failed to deserialize");
        let serialized = serde_yaml::to_string(&config).expect("Failed to serialize");
        let config2: KubeConfig =
            serde_yaml::from_str(&serialized).expect("Failed to deserialize again");

        assert_eq!(config.api_version, config2.api_version);
        assert_eq!(config.kind, config2.kind);
        assert_eq!(config.current_context, config2.current_context);
        assert_eq!(config.clusters.len(), config2.clusters.len());
        assert_eq!(config.contexts.len(), config2.contexts.len());
        assert_eq!(config.users.len(), config2.users.len());
    }
}
