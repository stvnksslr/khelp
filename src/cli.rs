use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

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

    /// Delete a specific context
    Delete {
        /// Name of the context to delete
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,

        /// Also delete orphaned clusters and users
        #[arg(long)]
        cleanup: bool,
    },

    /// Add contexts from an external kubeconfig file
    Add {
        /// Path to the kubeconfig file to import
        #[arg(value_hint = ValueHint::FilePath)]
        file_path: PathBuf,

        /// Rename conflicting entries by appending a suffix
        #[arg(long)]
        rename: bool,

        /// Overwrite existing entries with the same name
        #[arg(long)]
        overwrite: bool,

        /// Switch to the first newly added context after import
        #[arg(long)]
        switch: bool,
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
