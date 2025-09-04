use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "khelp")]
#[command(about = "A tool to manage Kubernetes contexts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all available contexts
    List,
    /// Get the current context
    Current,
    /// Switch to a different context
    Switch {
        /// Name of the context to switch to (optional, interactive selection if not provided)
        context_name: Option<String>,
    },
    /// Edit a specific context
    Edit {
        /// Name of the context to edit (optional, interactive selection if not provided)
        context_name: Option<String>,
    },
    /// Export a specific context to stdout (can be redirected to a file)
    Export {
        /// Name of the context to export (optional, interactive selection if not provided)
        context_name: Option<String>,
    },
}
