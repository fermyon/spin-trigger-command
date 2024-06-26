# Spin `command` Trigger

This is a very simple Spin trigger that executes the WASI main function.

## Installation

The trigger is installed as a Spin plugin. It can be installed from a release or build.

To install from a release, reference a plugin manifest from a [release](https://github.com/fermyon/spin-trigger-command/releases). For example, to install the canary release:

```sh
spin plugins install --url https://github.com/fermyon/spin-trigger-command/releases/download/canary/trigger-command.json
```

Alternatively, use the `spin pluginify` plugin to install from a fresh build. This will use the pluginify manifest (`spin-pluginify.toml`) to package the plugin and proceed to install it:

```sh
spin plugins install pluginify
cargo build --release
spin pluginify install
```
