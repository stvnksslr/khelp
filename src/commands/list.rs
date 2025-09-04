use crate::config::kubernetes::KubeConfig;
use console::style;

/// List all available Kubernetes contexts, highlighting the current one
pub fn list_contexts(config: &KubeConfig) {
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
