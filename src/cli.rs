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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_list_command() {
        let cli = Cli::parse_from(&["khelp", "list"]);
        matches!(cli.command, Some(Commands::List));
    }

    #[test]
    fn test_cli_parsing_current_command() {
        let cli = Cli::parse_from(&["khelp", "current"]);
        matches!(cli.command, Some(Commands::Current));
    }

    #[test]
    fn test_cli_parsing_switch_command_with_context() {
        let cli = Cli::parse_from(&["khelp", "switch", "my-context"]);
        match cli.command {
            Some(Commands::Switch { context_name }) => {
                assert_eq!(context_name, Some("my-context".to_string()));
            }
            _ => panic!("Expected Switch command"),
        }
    }

    #[test]
    fn test_cli_parsing_switch_command_without_context() {
        let cli = Cli::parse_from(&["khelp", "switch"]);
        match cli.command {
            Some(Commands::Switch { context_name }) => {
                assert_eq!(context_name, None);
            }
            _ => panic!("Expected Switch command"),
        }
    }

    #[test]
    fn test_cli_parsing_edit_command_with_context() {
        let cli = Cli::parse_from(&["khelp", "edit", "my-context"]);
        match cli.command {
            Some(Commands::Edit { context_name }) => {
                assert_eq!(context_name, Some("my-context".to_string()));
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_cli_parsing_edit_command_without_context() {
        let cli = Cli::parse_from(&["khelp", "edit"]);
        match cli.command {
            Some(Commands::Edit { context_name }) => {
                assert_eq!(context_name, None);
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_cli_parsing_export_command_with_context() {
        let cli = Cli::parse_from(&["khelp", "export", "my-context"]);
        match cli.command {
            Some(Commands::Export { context_name }) => {
                assert_eq!(context_name, Some("my-context".to_string()));
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_cli_parsing_export_command_without_context() {
        let cli = Cli::parse_from(&["khelp", "export"]);
        match cli.command {
            Some(Commands::Export { context_name }) => {
                assert_eq!(context_name, None);
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_cli_parsing_completions_command_with_shell() {
        let cli = Cli::parse_from(&["khelp", "completions", "bash"]);
        match cli.command {
            Some(Commands::Completions { shell, install }) => {
                assert_eq!(shell, Some(Shell::Bash));
                assert!(!install);
            }
            _ => panic!("Expected Completions command"),
        }
    }

    #[test]
    fn test_cli_parsing_completions_command_with_install() {
        let cli = Cli::parse_from(&["khelp", "completions", "--install"]);
        match cli.command {
            Some(Commands::Completions { shell, install }) => {
                assert_eq!(shell, None);
                assert!(install);
            }
            _ => panic!("Expected Completions command"),
        }
    }

    #[test]
    fn test_cli_parsing_completions_command_with_shell_and_install() {
        let cli = Cli::parse_from(&["khelp", "completions", "zsh", "--install"]);
        match cli.command {
            Some(Commands::Completions { shell, install }) => {
                assert_eq!(shell, Some(Shell::Zsh));
                assert!(install);
            }
            _ => panic!("Expected Completions command"),
        }
    }

    #[cfg(feature = "self_update")]
    #[test]
    fn test_cli_parsing_update_command_without_apply() {
        let cli = Cli::parse_from(&["khelp", "update"]);
        match cli.command {
            Some(Commands::Update { apply }) => {
                assert!(!apply);
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[cfg(feature = "self_update")]
    #[test]
    fn test_cli_parsing_update_command_with_apply() {
        let cli = Cli::parse_from(&["khelp", "update", "--apply"]);
        match cli.command {
            Some(Commands::Update { apply }) => {
                assert!(apply);
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_cli_parsing_no_command_defaults_to_none() {
        let cli = Cli::parse_from(&["khelp"]);
        assert!(cli.command.is_none());
    }
}
