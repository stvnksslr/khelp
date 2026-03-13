use crate::cli::OutputFormat;
use crate::config::kubernetes::KubeConfig;
use console::style;
use serde::Serialize;

#[derive(Serialize)]
struct ContextInfo {
    name: String,
    cluster: String,
    user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    current: bool,
}

/// List all available Kubernetes contexts, highlighting the current one
pub fn list_contexts(config: &KubeConfig, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
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
        OutputFormat::Name => {
            for context in &config.contexts {
                println!("{}", context.name);
            }
        }
        OutputFormat::Json => {
            let contexts: Vec<ContextInfo> = config
                .contexts
                .iter()
                .map(|c| ContextInfo {
                    name: c.name.clone(),
                    cluster: c.context.cluster.clone(),
                    user: c.context.user.clone(),
                    namespace: c.context.namespace.clone(),
                    current: c.name == config.current_context,
                })
                .collect();
            if let Ok(json) = serde_json::to_string_pretty(&contexts) {
                println!("{}", json);
            }
        }
    }
}
