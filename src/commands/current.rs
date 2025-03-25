use console::style;
use crate::config::kubernetes::KubeConfig;

/// Display details about the currently active context
pub fn show_current_context(config: &KubeConfig) {
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