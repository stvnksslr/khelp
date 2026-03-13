use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "khelp")]
#[command(about = "A tool to manage Kubernetes contexts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the kubeconfig file
    #[arg(long, short = 'k', global = true, env = "KUBECONFIG", value_hint = ValueHint::FilePath)]
    pub kubeconfig: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table output (default)
    Table,
    /// Bare names, one per line
    Name,
    /// JSON output
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all available contexts
    #[command(visible_alias = "ls")]
    List {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Table)]
        output: OutputFormat,
    },

    /// Get the current context
    Current {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Table)]
        output: OutputFormat,
    },

    /// Switch to a different context
    #[command(visible_aliases = ["use", "s"])]
    Switch {
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,
    },

    /// Edit a specific context
    Edit {
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,
    },

    /// Export one or more contexts to stdout (can be redirected to a file)
    Export {
        /// Names of contexts to export (if none provided, interactive selection)
        #[arg(value_hint = ValueHint::Other, num_args = 0..)]
        context_names: Vec<String>,
    },

    /// Delete a specific context (also removes orphaned cluster and user)
    #[command(visible_alias = "rm")]
    Delete {
        /// Name of the context to delete
        #[arg(value_hint = ValueHint::Other)]
        context_name: Option<String>,

        /// Skip confirmation prompt
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Clean up orphaned clusters and users not referenced by any context
    Cleanup {
        /// Skip confirmation prompt
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Rename a context
    #[command(visible_alias = "mv")]
    Rename {
        /// Current name of the context
        #[arg(value_hint = ValueHint::Other)]
        old_name: String,

        /// New name for the context
        #[arg(value_hint = ValueHint::Other)]
        new_name: String,
    },

    /// Add contexts from an external kubeconfig file
    Add {
        /// Path to the kubeconfig file to import
        #[arg(value_hint = ValueHint::FilePath)]
        file_path: PathBuf,

        /// Rename conflicting entries by appending a suffix
        #[arg(long, short = 'r')]
        rename: bool,

        /// Overwrite existing entries with the same name
        #[arg(long, short = 'o')]
        overwrite: bool,

        /// Switch to the first newly added context after import
        #[arg(long, short = 's')]
        switch: bool,
    },

    /// Generate or install shell completions
    Completions {
        #[arg(value_enum)]
        shell: Option<Shell>,
        #[arg(long, short = 'i')]
        install: bool,
    },

    /// Check for updates to khelp
    #[cfg(feature = "self_update")]
    Update {
        /// Apply the update if one is available
        #[arg(long, short = 'a')]
        apply: bool,
    },
}
