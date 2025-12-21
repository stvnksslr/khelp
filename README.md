# Khelp(er)

[![Crates.io](https://img.shields.io/crates/v/khelp.svg?color=blue)](https://crates.io/crates/khelp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A user-friendly CLI tool to manage Kubernetes contexts with ease.

## Features

- 🔍 **List** all available Kubernetes contexts
- 👁️ **View** the current active context with details
- 🔄 **Switch** between contexts with interactive selection
- ✏️ **Edit** context configurations with your preferred editor
- 📤 **Export** specific contexts for sharing or backup
- 🗑️ **Delete** contexts with optional cleanup of orphaned resources
- ➕ **Add** contexts from external kubeconfig files
- 🔄 **Update** to the latest version using built-in self-update
- 🛠️ **Shell Completions** for bash, zsh, fish, and PowerShell

## Installation

easy install script, source located at [install.sh](https://github.com/stvnksslr/khelp/blob/main/install.sh)

```sh
curl https://files.stvnksslr.com/khelp/install.sh | bash
```

### From Cargo

```bash
cargo install khelp
```

### Windows

#### Using Cargo
```powershell
cargo install khelp
```

#### Manual Download
Download the latest `.zip` file for Windows from [Releases](https://github.com/stvnksslr/khelp/releases), extract it, and add the directory containing `khelp.exe` to your PATH.

## Usage

### Overview

```sh
A tool to manage Kubernetes contexts

Usage: khelp [COMMAND]

Commands:
  list         List all available contexts
  current      Get the current context
  switch       Switch to a different context
  edit         Edit a specific context
  export       Export a specific context to stdout (can be redirected to a file)
  delete       Delete a specific context
  add          Add contexts from an external kubeconfig file
  completions  Generate or install shell completions
  update       Check for updates to khelp
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### List

View all available Kubernetes contexts in your configuration:

```bash
khelp list
```

Example output:

```
Kubernetes available contexts:
------------------------
* minikube
  production
  staging
  development
```

### Current

Display details about the currently active context:

```bash
khelp current
```

Example output:

```
Current context: minikube
  Cluster: minikube
  User: minikube
  Namespace: default
```

### Switch

Switch to a different context:

```bash
khelp switch
```

This will display an interactive menu to select the target context. You can also specify the context directly:

```bash
khelp switch production
```

### Edit

Open a context configuration in your default editor:

```bash
khelp edit
```

Or specify a context to edit:

```bash
khelp edit staging
```

The tool will open your default editor (defined by `$EDITOR` or `$VISUAL` environment variables) with the context configuration. Changes are automatically saved back to your Kubernetes config file.

### Export

Export a specific context configuration to stdout:

```bash
khelp export dev > dev-context.yaml
```

```yaml
apiVersion: v1
clusters:
  - cluster:
      certificate-authority-data: example
      server: https://192.168.64.2:8443
    name: dev
contexts:
  - context:
      cluster: dev
      user: dev-user
      namespace: default
    name: dev
current-context: dev
kind: Config
preferences: {}
users:
  - name: dev-user
    user:
      client-certificate-data: example
      client-key-data: example
```

This is useful for sharing configurations or creating backups of specific contexts.

### Delete

Delete a Kubernetes context:

```bash
khelp delete
```

This will display an interactive menu to select the context to delete. You can also specify the context directly:

```bash
khelp delete staging
```

By default, you'll be prompted for confirmation. You can skip the confirmation with `--force`:

```bash
khelp delete staging --force
```

The delete command can also clean up orphaned clusters and users (resources no longer referenced by any context):

```bash
khelp delete staging --cleanup
```

Example output:

```
✓ Deleted context: staging
✓ Deleted orphaned cluster: staging-cluster
✓ Deleted orphaned user: staging-user
```

**Note:** If you delete the current context, khelp will automatically switch you to another available context.

### Add

Import contexts from an external kubeconfig file:

```bash
khelp add ~/Downloads/new-cluster.yaml
```

By default, if a context with the same name already exists, it will be skipped:

```bash
khelp add ~/Downloads/cluster.yaml
```

```
Import Summary:
───────────────
− Skipped context(s): production
− Skipped cluster(s): production-cluster
− Skipped user(s): production-user

Tip: Use --rename to rename conflicting entries or --overwrite to overwrite them.
```

To rename conflicting entries automatically:

```bash
khelp add ~/Downloads/cluster.yaml --rename
```

```
Import Summary:
───────────────
✓ Added context(s): production-imported
✓ Added cluster(s): production-cluster-imported
✓ Added user(s): production-user-imported
```

To overwrite existing entries:

```bash
khelp add ~/Downloads/cluster.yaml --overwrite
```

You can also automatically switch to the newly imported context:

```bash
khelp add ~/Downloads/cluster.yaml --rename --switch
```

### Completions

Generate shell completions for supported shells:

```bash
khelp completions [SHELL]
```

Where `[SHELL]` can be one of: `bash`, `zsh`, `fish`, `powershell`, or `elvish`.

Example:

```bash
# Output bash completions to stdout
khelp completions bash

# Install completions for your current shell
khelp completions --install

# Install completions for a specific shell
khelp completions bash --install
```

Installing completions will:
- Create the appropriate completions directory if needed
- Generate and save the completion script
- Update your shell configuration file to load the completions
- Make the script executable

After installing completions, you'll need to restart your shell or source your configuration file:

```bash
# For bash
source ~/.bashrc

# For zsh
source ~/.zshrc

# For fish
# No action needed, fish loads completions automatically
```

### Update

Check for and apply updates to khelp:

```bash
# Check for updates (without applying)
khelp update

# Check for and apply updates
khelp update --apply
```

The update command connects to GitHub to check for new releases. If a newer version is available, it can automatically download and update your installation.

## Environment Variables

- `EDITOR` or `VISUAL`: Specifies the editor to use when editing context configurations (defaults to `vi` on Unix systems and `notepad` on Windows)
