# Spin `command` Trigger

This is a very simple Spin trigger that executes the WASI main function.

## Installation

The trigger is installed as a Spin plugin. It can be installed from a release or build.

## Install the latest version of the plugin

The latest stable release of the command trigger plugin can be installed like so:

```sh
spin plugins update
spin plugin install trigger-command
```

## Install the canary version of the plugin

The canary release of the command trigger plugin represents the most recent commits on `main` and may not be stable, with some features still in progress.

```sh
spin plugins install --url https://github.com/fermyon/spin-trigger-command/releases/download/canary/trigger-command.json
```

## Install from a local build

Alternatively, use the `spin pluginify` plugin to install from a fresh build. This will use the pluginify manifest (`spin-pluginify.toml`) to package the plugin and proceed to install it:

```sh
spin plugins install pluginify
cargo build --release
spin pluginify install
```
