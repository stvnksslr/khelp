use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;

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
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,
    },

    /// Edit a specific context
    Edit {
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,
    },

    /// Export a specific context to stdout (can be redirected to a file)
    Export {
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,
    },

    /// Generate or install shell completions
    Completions {
        #[arg(value_enum)]
        shell: Option<Shell>,
        #[arg(long)]
        install: bool,
    },

    /// Check for updates to khelp
    #[cfg(feature = "self_update")]
    Update {
        /// Apply the update if one is available
        #[arg(long)]
        apply: bool,
    },
}
