use anyhow::{Context, Result};
use clap_complete::Shell;
use console::style;
use log::{debug, info};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Generate shell completions
///
/// This function uses the clap_complete crate to generate shell completions for
/// the specified shell.
pub fn generate_completions(shell: Shell, install: bool) -> Result<()> {
    debug!(
        "Running completions command with shell: {:?}, install: {}",
        shell, install
    );

    if install {
        install_completions(shell)?;
    } else {
        // Generate a custom completion script based on the shell type
        // This completely avoids using clap_complete for stdout output
        match shell {
            Shell::Bash => {
                // Simple bash completions
                println!("# Bash completions for khelp");
                println!("_khelp_completions() {{");
                println!("  local cur prev");
                println!("  COMPREPLY=()");
                println!("  cur=\"${{COMP_WORDS[COMP_CWORD]}}\"");
                println!("  prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"");
                println!();
                println!("  if [ \"$COMP_CWORD\" -eq 1 ]; then");
                println!(
                    "    COMPREPLY=( $(compgen -W \"list current switch edit export completions\" -- \"$cur\") )"
                );
                println!("    return 0");
                println!("  fi");
                println!();
                println!("  if [ \"$COMP_CWORD\" -ge 2 ]; then");
                println!("    case \"$prev\" in");
                println!("      switch|edit|export)");
                println!(
                    "        COMPREPLY=( $(compgen -W \"$(kubectl config get-contexts -o name 2>/dev/null)\" -- \"$cur\") )"
                );
                println!("        ;;");
                println!("      completions)");
                println!(
                    "        COMPREPLY=( $(compgen -W \"bash zsh fish powershell elvish\" -- \"$cur\") )"
                );
                println!("        ;;");
                println!("    esac");
                println!("  fi");
                println!("}}");
                println!();
                println!("complete -F _khelp_completions khelp");
            }
            Shell::Zsh => {
                // Simple zsh completions
                println!("#compdef khelp");
                println!();
                println!("_khelp() {{");
                println!("  local -a commands");
                println!("  commands=(");
                println!("    'list:List all available contexts'");
                println!("    'current:Get the current context'");
                println!("    'switch:Switch to a different context'");
                println!("    'edit:Edit a specific context'");
                println!("    'export:Export a specific context to stdout'");
                println!("    'completions:Generate shell completions'");
                println!("  )");
                println!();
                println!("  _arguments -C \\");
                println!("    '1: :->command' \\");
                println!("    '2: :->argument' \\");
                println!("    '*::arg:->args'");
                println!();
                println!("  case $state in");
                println!("    (command)");
                println!("      _describe -t commands 'khelp commands' commands");
                println!("      ;;");
                println!("    (argument)");
                println!("      case $line[1] in");
                println!("        (switch|edit|export)");
                println!("          local -a contexts");
                println!(
                    "          contexts=(${{{{(f)\"$(kubectl config get-contexts -o name 2>/dev/null)\"}}}}"
                );
                println!("          _describe 'contexts' contexts");
                println!("          ;;");
                println!("        (completions)");
                println!("          local -a shells");
                println!("          shells=('bash' 'zsh' 'fish' 'powershell' 'elvish')");
                println!("          _describe 'shells' shells");
                println!("          ;;");
                println!("      esac");
                println!("      ;;");
                println!("  esac");
                println!("}}");
                println!();
                println!("_khelp");
            }
            Shell::Fish => {
                // Simple fish completions
                println!("# Fish completions for khelp");
                println!();
                println!("function __khelp_get_contexts");
                println!("    kubectl config get-contexts -o name 2>/dev/null");
                println!("end");
                println!();
                println!("# Main commands");
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a list -d \"List all available contexts\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a current -d \"Get the current context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a switch -d \"Switch to a different context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a edit -d \"Edit a specific context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a export -d \"Export a specific context to stdout\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export completions\" -a completions -d \"Generate shell completions\""
                );
                println!();
                println!("# Context name completions");
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from switch\" -a \"(__khelp_get_contexts)\" -d \"Kubernetes context\""
                );
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from edit\" -a \"(__khelp_get_contexts)\" -d \"Kubernetes context\""
                );
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from export\" -a \"(__khelp_get_contexts)\" -d \"Kubernetes context\""
                );
                println!();
                println!("# Shell completions");
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from completions\" -a \"bash zsh fish powershell elvish\" -d \"Shell\""
                );
            }
            _ => {
                println!("# Completions not supported for this shell");
                println!("# Supported shells: bash, zsh, fish");
            }
        }
    }

    // Ensure output is flushed
    io::stdout().flush().context("Failed to flush stdout")?;

    Ok(())
}

/// Detect the current shell
pub fn detect_shell() -> Result<Shell> {
    let shell_path = env::var("SHELL").context("$SHELL environment variable not set")?;
    let path = PathBuf::from(shell_path);
    let shell_name = path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .context("Invalid shell path")?;

    debug!("Detected shell: {}", shell_name);

    match shell_name {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        _ => anyhow::bail!(
            "Unsupported shell: {}. Please specify a supported shell (bash, zsh, fish)",
            shell_name
        ),
    }
}

/// Install completions for the specified shell
fn install_completions(shell: Shell) -> Result<()> {
    debug!(
        "Starting installation process for {:?} shell completions",
        shell
    );

    let shell = if shell == Shell::Bash || shell == Shell::Zsh || shell == Shell::Fish {
        debug!("Shell {:?} is directly supported", shell);
        shell
    } else {
        // Auto-detect shell if not one of the supported ones
        debug!(
            "Shell {:?} support is limited. Attempting to detect current shell...",
            shell
        );
        let detected = detect_shell()?;
        debug!("Detected shell: {:?}", detected);
        detected
    };

    debug!("Installing completions for shell: {:?}", shell);

    let result = match shell {
        Shell::Bash => {
            debug!("Installing Bash completions");
            install_bash_completions()
        }
        Shell::Zsh => {
            debug!("Installing Zsh completions");
            install_zsh_completions()
        }
        Shell::Fish => {
            debug!("Installing Fish completions");
            install_fish_completions()
        }
        _ => {
            debug!("Unsupported shell: {:?}", shell);
            anyhow::bail!("Completions installation not implemented for {:?}", shell)
        }
    };

    debug!("Installation process result: {:?}", result.is_ok());
    result
}

/// Install Bash completions
fn install_bash_completions() -> Result<()> {
    info!("Installing Bash completions for khelp...");

    let home = dirs::home_dir().context("Could not find home directory")?;
    let completions_dir = home.join(".bash_completion.d");

    debug!(
        "Creating completions directory: {}",
        completions_dir.display()
    );

    // Create completions directory if it doesn't exist
    fs::create_dir_all(&completions_dir).context("Failed to create completions directory")?;

    // Generate the completion script content
    let content = r#"#!/usr/bin/env bash

# Dynamic Kubernetes context completion for khelp in Bash

# Get the Kubernetes contexts from kubectl
_khelp_get_contexts() {
    kubectl config get-contexts -o name 2>/dev/null
}

# Complete khelp commands and options
_khelp_complete() {
    local cur prev words cword
    _init_completion || return

    # Complete first argument (command)
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "list current switch edit export completions" -- "$cur"))
        return 0
    fi

    # Complete second argument based on first argument
    if [[ $cword -eq 2 ]]; then
        case "$prev" in
            switch|edit|export)
                # Complete with context names
                COMPREPLY=($(compgen -W "$(_khelp_get_contexts)" -- "$cur"))
                return 0
                ;;
            completions)
                # Complete with shell names
                COMPREPLY=($(compgen -W "bash zsh fish powershell elvish" -- "$cur"))
                return 0
                ;;
            *)
                return 0
                ;;
        esac
    fi

    return 0
}

# Register the completion function
complete -F _khelp_complete khelp
"#;

    // Write the completion script
    let completions_file = completions_dir.join("khelp");
    fs::write(&completions_file, content).context("Failed to write completion script")?;

    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&completions_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&completions_file, perms)?;
    }

    // Update .bashrc if needed
    let bashrc_path = home.join(".bashrc");
    if let Ok(bashrc_content) = fs::read_to_string(&bashrc_path) {
        let source_line = format!("source {}", completions_file.display());
        if !bashrc_content.contains(&source_line) {
            let mut bashrc_file = fs::OpenOptions::new()
                .append(true)
                .open(bashrc_path)
                .context("Failed to open .bashrc")?;

            writeln!(bashrc_file, "\n# Source khelp completions")?;
            writeln!(bashrc_file, "{}", source_line)?;
            debug!("Added source line to ~/.bashrc");
        }
    }

    println!(
        "{}",
        style("Bash completions installed successfully!")
            .green()
            .bold()
    );
    println!(
        "Please run 'source ~/.bash_completion.d/khelp' to enable completions in your current session."
    );

    Ok(())
}

/// Install Zsh completions
fn install_zsh_completions() -> Result<()> {
    info!("Installing Zsh completions for khelp...");

    let home = dirs::home_dir().context("Could not find home directory")?;
    let completions_dir = home.join(".zfunc");

    // Create completions directory if it doesn't exist
    fs::create_dir_all(&completions_dir).context("Failed to create completions directory")?;

    // Generate the completion script content
    let content = r#"#compdef khelp

# Dynamic Kubernetes context completion for khelp in Zsh

# Function to get Kubernetes contexts
_khelp_get_contexts() {
    local -a contexts
    contexts=(${(f)"$(kubectl config get-contexts -o name 2>/dev/null)"})
    _describe 'contexts' contexts
}

# Define the completion function
_khelp() {
    local line state

    _arguments -C \
        '1: :->command' \
        '2: :->argument' \
        '*: :->args'

    case $state in
        command)
            _values "command" \
                "list[List all available contexts]" \
                "current[Get the current context]" \
                "switch[Switch to a different context]" \
                "edit[Edit a specific context]" \
                "export[Export a specific context to stdout]" \
                "completions[Generate shell completions]"
            ;;
        argument)
            case $line[1] in
                switch|edit|export)
                    _khelp_get_contexts
                    ;;
                completions)
                    _values "shell" "bash" "zsh" "fish" "powershell" "elvish"
                    ;;
            esac
            ;;
    esac
}

# Register the completion function
compdef _khelp khelp
"#;

    // Write the completion script
    let completions_file = completions_dir.join("_khelp");
    fs::write(&completions_file, content).context("Failed to write completion script")?;

    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&completions_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&completions_file, perms)?;
    }

    // Update .zshrc if needed
    let zshrc_path = home.join(".zshrc");
    if let Ok(zshrc_content) = fs::read_to_string(&zshrc_path) {
        if !zshrc_content.contains("fpath=(~/.zfunc") {
            let mut zshrc_file = fs::OpenOptions::new()
                .append(true)
                .open(zshrc_path)
                .context("Failed to open .zshrc")?;

            writeln!(zshrc_file, "\n# Add khelp completions to fpath")?;
            writeln!(zshrc_file, "fpath=(~/.zfunc $fpath)")?;
            writeln!(zshrc_file, "autoload -Uz compinit && compinit")?;
            debug!("Added fpath configuration to ~/.zshrc");
        }
    }

    println!(
        "{}",
        style("Zsh completions installed successfully!")
            .green()
            .bold()
    );
    println!("Please run 'source ~/.zshrc' to enable completions in your current session.");

    Ok(())
}

/// Install Fish completions
fn install_fish_completions() -> Result<()> {
    info!("Installing Fish completions for khelp...");

    let home = dirs::home_dir().context("Could not find home directory")?;
    debug!("Home directory: {}", home.display());

    let completions_dir = home.join(".config/fish/completions");
    debug!("Fish completions directory: {}", completions_dir.display());

    // Create completions directory if it doesn't exist
    debug!("Creating fish completions directory...");
    fs::create_dir_all(&completions_dir).context("Failed to create fish completions directory")?;
    debug!("Directory created successfully");

    // Generate the completion script content
    debug!("Preparing fish completion script...");
    let content = r#"# Dynamic Kubernetes context completion for khelp in Fish

function __khelp_get_contexts
    kubectl config get-contexts -o name 2>/dev/null
end

# Define command completions
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a list -d "List all available contexts"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a current -d "Get the current context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a switch -d "Switch to a different context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a edit -d "Edit a specific context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a export -d "Export a specific context to stdout"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export completions" -a completions -d "Generate shell completions"

# Define context name completions for the relevant commands
complete -c khelp -f -n "__fish_seen_subcommand_from switch" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from edit" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from export" -a "(__khelp_get_contexts)" -d "Kubernetes context"

# Define shell completions for the completions command
complete -c khelp -f -n "__fish_seen_subcommand_from completions" -a "bash zsh fish powershell elvish" -d "Shell"
"#;

    // Write the completion script
    let completions_file = completions_dir.join("khelp.fish");
    debug!(
        "Writing completion script to: {}",
        completions_file.display()
    );
    fs::write(&completions_file, content).context("Failed to write fish completion script")?;
    debug!("Fish completion script written successfully");

    println!(
        "{}",
        style("Fish completions installed successfully!")
            .green()
            .bold()
    );
    println!("Fish will automatically load the completions for new sessions.");

    Ok(())
}
