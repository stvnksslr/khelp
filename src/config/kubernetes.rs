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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_kube_config() -> KubeConfig {
        KubeConfig {
            api_version: "v1".to_string(),
            kind: "Config".to_string(),
            current_context: "test-context".to_string(),
            preferences: Preferences {},
            clusters: vec![ClusterEntry {
                name: "test-cluster".to_string(),
                cluster: ClusterData {
                    server: "https://test.example.com".to_string(),
                    certificate_authority_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    certificate_authority: None,
                    insecure_skip_tls_verify: Some(false),
                },
            }],
            contexts: vec![ContextEntry {
                name: "test-context".to_string(),
                context: ContextData {
                    cluster: "test-cluster".to_string(),
                    user: "test-user".to_string(),
                    namespace: Some("default".to_string()),
                },
            }],
            users: vec![UserEntry {
                name: "test-user".to_string(),
                user: UserData {
                    client_certificate_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    client_key_data: Some("LS0tLS1CRUdJTi...".to_string()),
                    token: None,
                    username: None,
                    password: None,
                    exec: None,
                },
            }],
        }
    }

    #[test]
    fn test_kube_config_serialization() {
        let config = create_test_kube_config();
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(yaml.contains("apiVersion: v1"));
        assert!(yaml.contains("kind: Config"));
        assert!(yaml.contains("current-context: test-context"));
        assert!(yaml.contains("test-cluster"));
        assert!(yaml.contains("test-user"));
    }

    #[test]
    fn test_kube_config_deserialization() {
        let yaml = r#"
apiVersion: v1
kind: Config
current-context: test-context
preferences: {}
clusters:
- name: test-cluster
  cluster:
    server: https://test.example.com
    certificate-authority-data: LS0tLS1CRUdJTi...
contexts:
- name: test-context
  context:
    cluster: test-cluster
    user: test-user
    namespace: default
users:
- name: test-user
  user:
    client-certificate-data: LS0tLS1CRUdJTi...
    client-key-data: LS0tLS1CRUdJTi...
"#;

        let config: KubeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Config");
        assert_eq!(config.current_context, "test-context");
        assert_eq!(config.clusters.len(), 1);
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.users.len(), 1);
    }

    #[test]
    fn test_cluster_data_with_certificate_authority_file() {
        let yaml = r#"
server: https://test.example.com
certificate-authority: /path/to/ca.crt
"#;
        let cluster_data: ClusterData = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cluster_data.server, "https://test.example.com");
        assert_eq!(
            cluster_data.certificate_authority,
            Some("/path/to/ca.crt".to_string())
        );
        assert_eq!(cluster_data.certificate_authority_data, None);
    }

    #[test]
    fn test_cluster_data_with_insecure_skip_tls_verify() {
        let yaml = r#"
server: https://test.example.com
insecure-skip-tls-verify: true
"#;
        let cluster_data: ClusterData = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cluster_data.server, "https://test.example.com");
        assert_eq!(cluster_data.insecure_skip_tls_verify, Some(true));
    }

    #[test]
    fn test_context_data_without_namespace() {
        let yaml = r#"
cluster: test-cluster
user: test-user
"#;
        let context_data: ContextData = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(context_data.cluster, "test-cluster");
        assert_eq!(context_data.user, "test-user");
        assert_eq!(context_data.namespace, None);
    }

    #[test]
    fn test_user_data_with_token() {
        let yaml = r#"
token: eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9...
"#;
        let user_data: UserData = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            user_data.token,
            Some("eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9...".to_string())
        );
        assert_eq!(user_data.client_certificate_data, None);
        assert_eq!(user_data.client_key_data, None);
    }

    #[test]
    fn test_user_data_with_basic_auth() {
        let yaml = r#"
username: admin
password: secret
"#;
        let user_data: UserData = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(user_data.username, Some("admin".to_string()));
        assert_eq!(user_data.password, Some("secret".to_string()));
    }

    #[test]
    fn test_user_data_with_exec_config() {
        let yaml = r#"
exec:
  apiVersion: client.authentication.k8s.io/v1beta1
  command: aws
  args:
    - eks
    - get-token
    - --cluster-name
    - my-cluster
  env:
    - name: AWS_PROFILE
      value: default
"#;
        let user_data: UserData = serde_yaml::from_str(yaml).unwrap();
        assert!(user_data.exec.is_some());

        let exec_config = user_data.exec.unwrap();
        assert_eq!(
            exec_config.api_version,
            "client.authentication.k8s.io/v1beta1"
        );
        assert_eq!(exec_config.command, "aws");
        assert!(exec_config.args.is_some());
        assert!(exec_config.env.is_some());

        let args = exec_config.args.unwrap();
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "eks");

        let env = exec_config.env.unwrap();
        assert_eq!(env.len(), 1);
        assert_eq!(env[0].name, "AWS_PROFILE");
        assert_eq!(env[0].value, "default");
    }

    #[test]
    fn test_kube_config_round_trip() {
        let original_config = create_test_kube_config();
        let yaml = serde_yaml::to_string(&original_config).unwrap();
        let deserialized_config: KubeConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(original_config.api_version, deserialized_config.api_version);
        assert_eq!(original_config.kind, deserialized_config.kind);
        assert_eq!(
            original_config.current_context,
            deserialized_config.current_context
        );
        assert_eq!(
            original_config.clusters.len(),
            deserialized_config.clusters.len()
        );
        assert_eq!(
            original_config.contexts.len(),
            deserialized_config.contexts.len()
        );
        assert_eq!(original_config.users.len(), deserialized_config.users.len());
    }
}
