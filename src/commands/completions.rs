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
                    "    COMPREPLY=( $(compgen -W \"list current switch edit export delete rename add completions\" -- \"$cur\") )"
                );
                println!("    return 0");
                println!("  fi");
                println!();
                println!("  if [ \"$COMP_CWORD\" -ge 2 ]; then");
                println!("    case \"$prev\" in");
                println!("      switch|edit|export|delete|rename)");
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
                println!("    'delete:Delete a specific context'");
                println!("    'rename:Rename a context'");
                println!("    'add:Add contexts from an external kubeconfig file'");
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
                println!("        (switch|edit|export|delete|rename)");
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
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a list -d \"List all available contexts\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a current -d \"Get the current context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a switch -d \"Switch to a different context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a edit -d \"Edit a specific context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a export -d \"Export a specific context to stdout\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a delete -d \"Delete a specific context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a rename -d \"Rename a context\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a add -d \"Add contexts from an external kubeconfig file\""
                );
                println!(
                    "complete -c khelp -f -n \"not __fish_seen_subcommand_from list current switch edit export delete rename add completions\" -a completions -d \"Generate shell completions\""
                );
                println!();
                println!("# File path completion for add command");
                println!(
                    "complete -c khelp -F -n \"__fish_seen_subcommand_from add\" -d \"Kubeconfig file\""
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
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from delete\" -a \"(__khelp_get_contexts)\" -d \"Kubernetes context\""
                );
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from rename\" -a \"(__khelp_get_contexts)\" -d \"Kubernetes context\""
                );
                println!();
                println!("# Shell completions");
                println!(
                    "complete -c khelp -f -n \"__fish_seen_subcommand_from completions\" -a \"bash zsh fish powershell elvish\" -d \"Shell\""
                );
            }
            Shell::PowerShell => {
                // PowerShell completions
                println!("# PowerShell completions for khelp");
                println!("# Add this to your PowerShell profile ($PROFILE)");
                println!();
                println!("Register-ArgumentCompleter -Native -CommandName khelp -ScriptBlock {{");
                println!("    param($wordToComplete, $commandAst, $cursorPosition)");
                println!();
                println!("    $commands = @(");
                println!(
                    "        @{{ Name = 'list'; Description = 'List all available contexts' }}"
                );
                println!(
                    "        @{{ Name = 'current'; Description = 'Get the current context' }}"
                );
                println!(
                    "        @{{ Name = 'switch'; Description = 'Switch to a different context' }}"
                );
                println!("        @{{ Name = 'edit'; Description = 'Edit a specific context' }}");
                println!(
                    "        @{{ Name = 'export'; Description = 'Export a specific context to stdout' }}"
                );
                println!(
                    "        @{{ Name = 'delete'; Description = 'Delete a specific context' }}"
                );
                println!("        @{{ Name = 'rename'; Description = 'Rename a context' }}");
                println!(
                    "        @{{ Name = 'add'; Description = 'Add contexts from an external kubeconfig file' }}"
                );
                println!(
                    "        @{{ Name = 'completions'; Description = 'Generate shell completions' }}"
                );
                println!(
                    "        @{{ Name = 'update'; Description = 'Check for updates to khelp' }}"
                );
                println!("    )");
                println!();
                println!("    $elements = $commandAst.CommandElements");
                println!("    $command = $elements[1].Value");
                println!();
                println!("    # Complete subcommands");
                println!(
                    "    if ($elements.Count -eq 1 -or ($elements.Count -eq 2 -and $wordToComplete)) {{"
                );
                println!(
                    "        $commands | Where-Object {{ $_.Name -like \"$wordToComplete*\" }} | ForEach-Object {{"
                );
                println!(
                    "            [System.Management.Automation.CompletionResult]::new($_.Name, $_.Name, 'ParameterValue', $_.Description)"
                );
                println!("        }}");
                println!("        return");
                println!("    }}");
                println!();
                println!("    # Complete context names for relevant commands");
                println!(
                    "    if ($command -in @('switch', 'edit', 'export', 'delete', 'rename')) {{"
                );
                println!("        $contexts = kubectl config get-contexts -o name 2>$null");
                println!("        if ($contexts) {{");
                println!(
                    "            $contexts | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{"
                );
                println!(
                    "                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', \"Kubernetes context\")"
                );
                println!("            }}");
                println!("        }}");
                println!("        return");
                println!("    }}");
                println!();
                println!("    # Complete shells for completions command");
                println!("    if ($command -eq 'completions') {{");
                println!(
                    "        @('bash', 'zsh', 'fish', 'powershell', 'elvish') | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{"
                );
                println!(
                    "            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', \"Shell\")"
                );
                println!("        }}");
                println!("    }}");
                println!("}}");
            }
            _ => {
                println!("# Completions not supported for this shell");
                println!("# Supported shells: bash, zsh, fish, powershell");
            }
        }
    }

    // Ensure output is flushed
    io::stdout().flush().context("Failed to flush stdout")?;

    Ok(())
}

/// Detect the current shell
///
/// On Unix systems, this checks the $SHELL environment variable.
/// On Windows, this checks for PowerShell via PSModulePath environment variable.
pub fn detect_shell() -> Result<Shell> {
    // Try $SHELL first (Unix systems and some Windows terminals like Git Bash)
    if let Ok(shell_path) = env::var("SHELL") {
        let path = PathBuf::from(&shell_path);
        if let Some(shell_name) = path.file_name().and_then(|s| s.to_str()) {
            debug!("Detected shell from $SHELL: {}", shell_name);
            return match shell_name {
                "bash" => Ok(Shell::Bash),
                "zsh" => Ok(Shell::Zsh),
                "fish" => Ok(Shell::Fish),
                "pwsh" | "powershell" => Ok(Shell::PowerShell),
                _ => anyhow::bail!(
                    "Unsupported shell: {}. Please specify a supported shell (bash, zsh, fish, powershell)",
                    shell_name
                ),
            };
        }
    }

    // Windows: Check for PowerShell via PSModulePath environment variable
    if env::var("PSModulePath").is_ok() {
        debug!("Detected PowerShell via PSModulePath environment variable");
        return Ok(Shell::PowerShell);
    }

    // Windows: Check if running in cmd.exe
    if let Ok(comspec) = env::var("COMSPEC")
        && comspec.to_lowercase().contains("cmd.exe")
    {
        anyhow::bail!(
            "cmd.exe does not support tab completions. Please use PowerShell instead, or specify a shell explicitly."
        );
    }

    anyhow::bail!(
        "Could not detect shell. Please specify a shell explicitly (bash, zsh, fish, powershell)"
    )
}

/// Install completions for the specified shell
fn install_completions(shell: Shell) -> Result<()> {
    debug!(
        "Starting installation process for {:?} shell completions",
        shell
    );

    let shell = if shell == Shell::Bash
        || shell == Shell::Zsh
        || shell == Shell::Fish
        || shell == Shell::PowerShell
    {
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
        Shell::PowerShell => {
            debug!("Installing PowerShell completions");
            install_powershell_completions()
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
        COMPREPLY=($(compgen -W "list current switch edit export delete rename add completions" -- "$cur"))
        return 0
    fi

    # Complete second argument based on first argument
    if [[ $cword -eq 2 ]]; then
        case "$prev" in
            switch|edit|export|delete|rename)
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
                "delete[Delete a specific context]" \
                "rename[Rename a context]" \
                "add[Add contexts from an external kubeconfig file]" \
                "completions[Generate shell completions]"
            ;;
        argument)
            case $line[1] in
                switch|edit|export|delete|rename)
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
    if let Ok(zshrc_content) = fs::read_to_string(&zshrc_path)
        && !zshrc_content.contains("fpath=(~/.zfunc")
    {
        let mut zshrc_file = fs::OpenOptions::new()
            .append(true)
            .open(zshrc_path)
            .context("Failed to open .zshrc")?;

        writeln!(zshrc_file, "\n# Add khelp completions to fpath")?;
        writeln!(zshrc_file, "fpath=(~/.zfunc $fpath)")?;
        writeln!(zshrc_file, "autoload -Uz compinit && compinit")?;
        debug!("Added fpath configuration to ~/.zshrc");
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
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a list -d "List all available contexts"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a current -d "Get the current context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a switch -d "Switch to a different context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a edit -d "Edit a specific context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a export -d "Export a specific context to stdout"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a delete -d "Delete a specific context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a rename -d "Rename a context"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a add -d "Add contexts from an external kubeconfig file"
complete -c khelp -f -n "not __fish_seen_subcommand_from list current switch edit export delete rename add completions" -a completions -d "Generate shell completions"

# File path completion for add command
complete -c khelp -F -n "__fish_seen_subcommand_from add" -d "Kubeconfig file"

# Define context name completions for the relevant commands
complete -c khelp -f -n "__fish_seen_subcommand_from switch" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from edit" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from export" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from delete" -a "(__khelp_get_contexts)" -d "Kubernetes context"
complete -c khelp -f -n "__fish_seen_subcommand_from rename" -a "(__khelp_get_contexts)" -d "Kubernetes context"

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

/// Install PowerShell completions
fn install_powershell_completions() -> Result<()> {
    info!("Installing PowerShell completions for khelp...");

    // Determine the PowerShell profile path based on platform
    let profile_dir = if cfg!(target_os = "windows") {
        // Windows: Use Documents\PowerShell for PowerShell 7+ or Documents\WindowsPowerShell for 5.x
        dirs::document_dir()
            .context("Could not find Documents directory")?
            .join("PowerShell")
    } else {
        // Unix: PowerShell Core uses ~/.config/powershell
        dirs::config_dir()
            .context("Could not find config directory")?
            .join("powershell")
    };

    debug!("PowerShell profile directory: {}", profile_dir.display());

    // Create the profile directory if it doesn't exist
    fs::create_dir_all(&profile_dir).context("Failed to create PowerShell profile directory")?;

    // Generate the completion script content
    let content = r#"# khelp PowerShell completions
# Generated by khelp completions --install

Register-ArgumentCompleter -Native -CommandName khelp -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        @{ Name = 'list'; Description = 'List all available contexts' }
        @{ Name = 'current'; Description = 'Get the current context' }
        @{ Name = 'switch'; Description = 'Switch to a different context' }
        @{ Name = 'edit'; Description = 'Edit a specific context' }
        @{ Name = 'export'; Description = 'Export a specific context to stdout' }
        @{ Name = 'delete'; Description = 'Delete a specific context' }
        @{ Name = 'rename'; Description = 'Rename a context' }
        @{ Name = 'add'; Description = 'Add contexts from an external kubeconfig file' }
        @{ Name = 'completions'; Description = 'Generate shell completions' }
        @{ Name = 'update'; Description = 'Check for updates to khelp' }
    )

    $elements = $commandAst.CommandElements
    $command = if ($elements.Count -gt 1) { $elements[1].Value } else { $null }

    # Complete subcommands
    if ($elements.Count -eq 1 -or ($elements.Count -eq 2 -and $wordToComplete)) {
        $commands | Where-Object { $_.Name -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_.Name, $_.Name, 'ParameterValue', $_.Description)
        }
        return
    }

    # Complete context names for relevant commands
    if ($command -in @('switch', 'edit', 'export', 'delete', 'rename')) {
        $contexts = kubectl config get-contexts -o name 2>$null
        if ($contexts) {
            $contexts | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Kubernetes context")
            }
        }
        return
    }

    # Complete shells for completions command
    if ($command -eq 'completions') {
        @('bash', 'zsh', 'fish', 'powershell', 'elvish') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', "Shell")
        }
    }
}
"#;

    // Write the completion script to a separate file
    let completions_file = profile_dir.join("khelp_completions.ps1");
    debug!(
        "Writing completion script to: {}",
        completions_file.display()
    );
    fs::write(&completions_file, content)
        .context("Failed to write PowerShell completion script")?;

    // Update the PowerShell profile to source the completions
    let profile_path = profile_dir.join("Microsoft.PowerShell_profile.ps1");
    let source_line = format!(". \"{}\"", completions_file.display());

    // Check if the profile exists and if it already sources our completions
    let should_update = if let Ok(profile_content) = fs::read_to_string(&profile_path) {
        !profile_content.contains("khelp_completions.ps1")
    } else {
        true
    };

    if should_update {
        let mut profile_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&profile_path)
            .context("Failed to open PowerShell profile")?;

        writeln!(profile_file, "\n# khelp completions")?;
        writeln!(profile_file, "{}", source_line)?;
        debug!("Added source line to PowerShell profile");
    }

    println!(
        "{}",
        style("PowerShell completions installed successfully!")
            .green()
            .bold()
    );
    println!("Completions will be loaded automatically in new PowerShell sessions.");
    println!(
        "To enable in current session, run: . \"{}\"",
        completions_file.display()
    );

    Ok(())
}
