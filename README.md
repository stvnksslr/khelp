# khelp(er)

A command-line tool for managing Kubernetes contexts.

## Overview

khelp simplifies working with kubeconfig files by providing intuitive commands to list, switch, import, export, and manage Kubernetes contexts.

## Installation

### From Source

```bash
git clone https://github.com/stvnksslr/khelp.git
cd khelp
cargo build --release
```

The binary will be available at `target/release/khelp`.

### With Self-Update Feature

```bash
cargo build --release --features self_update
```

## Platform Support

- Linux
- macOS
- Windows

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all available contexts (current context marked with *) |
| `current` | Display details about the active context |
| `switch [name]` | Switch to a different context (interactive if no name given) |
| `edit [name]` | Edit a context configuration in your default editor |
| `export [names...]` | Export one or more contexts to stdout in YAML format |
| `delete [name]` | Delete a context and its orphaned cluster/user (supports --force) |
| `cleanup` | Remove orphaned clusters and users not referenced by any context |
| `rename <old> <new>` | Rename an existing context |
| `add <file>` | Import contexts from an external kubeconfig file |
| `completions [shell]` | Generate shell completions (bash, zsh, fish, powershell, elvish) |
| `update` | Check for and apply updates (requires self_update feature) |

## Usage Examples

List all contexts:
```bash
khelp list
```

Switch to a context interactively:
```bash
khelp switch
```

Switch to a specific context:
```bash
khelp switch my-cluster
```

Import contexts from another kubeconfig:
```bash
khelp add ~/Downloads/new-cluster.yaml
```

Import with automatic rename for conflicts:
```bash
khelp add ~/Downloads/cluster.yaml --rename
```

Export a context for backup:
```bash
khelp export my-cluster > my-cluster-backup.yaml
```

Export multiple contexts at once:
```bash
khelp export dev-cluster staging-cluster prod-cluster > all-clusters.yaml
```

Delete a context (automatically removes orphaned cluster/user):
```bash
khelp delete old-cluster
```

Clean up any orphaned clusters and users:
```bash
khelp cleanup
```

Install shell completions:
```bash
khelp completions --install
```

## Shell Completions

Generate completions for your shell:

```bash
# Bash
khelp completions bash >> ~/.bash_completion.d/khelp

# Zsh
khelp completions zsh > ~/.zfunc/_khelp

# Fish
khelp completions fish > ~/.config/fish/completions/khelp.fish
```

Or use automatic installation:
```bash
khelp completions --install
```

## License

MIT
