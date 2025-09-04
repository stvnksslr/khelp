# Khelp(er)

[![Crates.io](https://img.shields.io/crates/v/khelp.svg)](https://crates.io/crates/khelp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A user-friendly CLI tool to manage Kubernetes contexts with ease.

## Features

- 🔍 **List** all available Kubernetes contexts
- 👁️ **View** the current active context with details
- 🔄 **Switch** between contexts with interactive selection
- ✏️ **Edit** context configurations with your preferred editor
- 📤 **Export** specific contexts for sharing or backup

## Installation

easy install script, source located at [install.sh](https://github.com/stvnksslr/uv-migrator/blob/main/khelp/install.sh)

```sh
curl https://files.stvnksslr.com/khelp/install.sh | bash
```

### From Cargo

```bash
cargo install khelp
```

## Usage

### Overview

```sh
khelp --help
A tool to manage Kubernetes contexts

Usage: khelp [COMMAND]

Commands:
  list     List all available contexts
  current  Get the current context
  switch   Switch to a different context
  edit     Edit a specific context
  export   Export a specific context to stdout (can be redirected to a file)
  help     Print this message or the help of the given subcommand(s)

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

The tool will open your default editor (defined by `$EDITOR` or `$VISUAL` environment variables) with the context configuration. Changes are automatically saved back to your Kubernetes config file with a backup created.

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

## Environment Variables

- `EDITOR` or `VISUAL`: Specifies the editor to use when editing context configurations (defaults to `vi` on Unix systems and `notepad` on Windows)

## How It Works

Khelp interacts with your Kubernetes configuration file (typically located at `~/.kube/config`). When making changes, it automatically creates a backup of your original configuration file with a `.bak` extension.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
