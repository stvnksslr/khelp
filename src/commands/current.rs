use crate::cli::OutputFormat;
use crate::config::kubernetes::KubeConfig;
use console::style;
use serde::Serialize;

#[derive(Serialize)]
struct CurrentContextInfo {
    name: String,
    cluster: String,
    user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

/// Display details about the currently active context
pub fn show_current_context(config: &KubeConfig, output: &OutputFormat) {
    match output {
        OutputFormat::Table => {
            println!(
                "Current context: {}",
                style(&config.current_context).green().bold()
            );

            if let Some(context) = config
                .contexts
                .iter()
                .find(|c| c.name == config.current_context)
            {
                println!("  Cluster: {}", style(&context.context.cluster).cyan());
                println!("  User: {}", style(&context.context.user).cyan());

                if let Some(namespace) = &context.context.namespace {
                    println!("  Namespace: {}", style(namespace).cyan());
                }
            }
        }
        OutputFormat::Name => {
            println!("{}", config.current_context);
        }
        OutputFormat::Json => {
            if let Some(context) = config
                .contexts
                .iter()
                .find(|c| c.name == config.current_context)
            {
                let info = CurrentContextInfo {
                    name: config.current_context.clone(),
                    cluster: context.context.cluster.clone(),
                    user: context.context.user.clone(),
                    namespace: context.context.namespace.clone(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&info) {
                    println!("{}", json);
                }
            } else {
                println!("\"{}\"", config.current_context);
            }
        }
    }
}
