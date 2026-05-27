---
name: cyberfabric
description: cli reference to help with the development of cyberfabric framework. It helps with the development of
  the framework from its initialization, adding/removing modules, modifying configuration files, build and/or run project
  and deploy them.
---

# CyberFabric CLI Skills Guide

This document summarizes the CLI parser implemented under `crates/cyberware-cli/src`
and the orchestration logic implemented under `crates/cyberware-cli-core/src`.
It focuses on:

- **[command surface]** Every top-level command and nested subcommand
- **[arguments]** The supported flags and positional arguments
- **[purpose]** What each command is meant to do
- **[examples]** Realistic usage patterns

## Invocation Forms

The crate exposes a single entrypoint:

- **[`cargo cyberfabric`]** Cargo subcommand form via the `cargo-cyberfabric` binary

Example:

```bash
cargo cyberfabric generate workspace /tmp/my-app
```

## Objective

This CLI is a tool for automating cyberfabric development, a Rust framework. You can get more information about it in:

- CyberFabric repository main: https://github.com/cyberfabric/cyberfabric-core
- Modkit libraries(the ones that leverage this CLI tool) are located
  in https://github.com/cyberfabric/cyberfabric-core/tree/main/libs
- More documentation of the project will be located in https://github.com/cyberfabric/cyberfabric-core/tree/main/docs

Clone(shallow) the repo to .cyberfabric folder (create it if it doesn't exist), and use it as a reference.

## Command Tree

```text
cargo cyberfabric
├── generate
│   ├── workspace
│   ├── module
│   └── config
├── new
├── config
│   ├── mod
│   │   ├── list
│   │   ├── add
│   │   ├── db
│   │   │   ├── add
│   │   │   ├── edit
│   │   │   └── rm
│   │   └── rm
│   └── db
│       ├── add
│       ├── edit
│       └── rm
├── src
├── help
│   ├── schema
│   ├── src
│   └── topic
├── lint
├── list
│   ├── modules
│   ├── local-modules
│   ├── system-modules
│   ├── configs
│   └── apps
├── manifest
│   ├── validate
│   └── ls
├── test
├── tools
├── run
├── build
└── deploy
```

## Shared Argument Patterns

- **[`-p, --path <PATH>`]** Optional workspace path. When provided to `config ...`, `build`, `run`, `deploy`, and
  `lint`, the CLI immediately changes the current working directory to this directory. Relative config paths, generated
  project locations, and workspace-scoped lint resolution then resolve from that directory. When omitted, the current
  working directory is left unchanged.
- **[`-c, --config <PATH>`]** Config file path. This is required for `config ...` and `deploy` commands because there is
  no default. `build` and `run` no longer accept `--config`; they compose their generation inputs from `Cyberware.toml`
  and forward the manifest-declared runtime config path through the `CF_CLI_CONFIG` environment variable.
- **[`--manifest <PATH>`]** Cyberware manifest path, defaulting to `Cyberware.toml`, for `manifest`, `build`, and `run`.
  For `manifest`, you can combine this with `-p/--path` to resolve relative manifest paths from a selected workspace.
- **[`--app <APP> --env <ENV>`]** Selects a manifest app/environment for manifest-driven `build` and `run`.
- **[`--name <NAME>`]** For `build` and `run`, overrides the generated server project and binary name that would
  otherwise default to the config filename stem.
- **[`-v, --verbose`]** Usually enables more logging or richer output.
- **[name validation]** Config-managed names for modules, DB servers, and generated server names only allow letters,
  numbers, `-`, and `_`.

## What the Tool Manages

From the current implementation, the CLI is mainly for:

- **[workspace scaffolding]** Initialize a CyberFabric workspace and add module templates
- **[config management]** Enable modules and patch YAML config sections
- **[server generation]** Generate a runnable Cargo project under the manifest `workspace.generated-dir` directory
  (default `.cyberware/<name>/`)
- **[manifest orchestration]** Read `Cyberware.toml` to separate generation metadata from runtime YAML config
- **[build/run/deploy]** Build, run, or package that generated server as a Docker image
- **[source inspection]** Resolve Rust source for crates/items through workspace metadata or crates.io
- **[module inspection]** List workspace-discovered and system-registry modules
- **[tool bootstrap]** Install or upgrade `rustup`, `cargofmt`, and `clippy`

## Top-Level Commands

### `generate`

Generate workspace, module, and config scaffolding from built-in templates or explicit local/Git template sources.

#### `generate workspace`

Synopsis:

```bash
cargo cyberfabric generate workspace <path> [--template <TEMPLATE>] [--verbose] [--name <NAME>] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>] [--override]
```

Arguments:

- **[`<path>`]** Target directory to initialize
- **[`-t, --template <TEMPLATE>`]** Workspace template name, defaults to `default`
- **[`-v, --verbose`]** Verbose output from `cargo-generate`
- **[`-n, --name <NAME>`]** Override the generated project name; inferred from the final path segment by default
- **[`--local-path <PATH>`]** Use a local template directory instead of the default Git template
- **[`--git <URL>`]** Override the Git URL for the template
- **[`--subfolder <NAME>`]** Override the template subfolder
- **[`--branch <NAME>`]** Override the template branch
- **[`--override`]** Allow generated files to overwrite existing files

Behavior:

- **[creates target directory]** If it does not exist
- **[fails on file path]** Errors if `<path>` already exists and is not a directory
- **[uses directory name as project name]** The final path segment becomes the generated project name unless `--name` is
  provided
- **[uses template registry]** `--template default` resolves to the built-in workspace template
- **[forces git init]** Template generation runs with Git initialization enabled

Examples:

```bash
cargo cyberfabric generate workspace /tmp/cf-demo
```

```bash
cargo cyberfabric generate workspace /tmp/cf-demo --git https://github.com/cyberfabric/cf-template-rust --branch main --subfolder Init
```

```bash
cargo cyberfabric generate workspace /tmp/cf-demo --local-path ~/dev/cf-template-rust
```

#### `generate module`

Generate a module template inside an existing workspace's `modules/` directory and wire Cargo workspace dependencies.

Synopsis:

```bash
cargo cyberfabric generate module --template <TEMPLATE> [--name <NAME>] [--path <PATH>] [--verbose] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>]
```

Available built-in templates:

- **[`background-worker`]** Background worker module template
- **[`api-db-handler`]** API/database handler module template
- **[`api-gateway`]** API gateway module template

Arguments:

- **[`-t, --template <TEMPLATE>`]** Module template name
- **[`-n, --name <NAME>`]** Generated module folder/crate name; defaults to the template name. Prefer passing this when
  the generated module should use the user's chosen name.
- **[`-p, --path <PATH>`]** Workspace root, defaults to `.`
- **[`-v, --verbose`]** Verbose template generation output
- **[`--local-path <PATH>`]** Use a local template directory instead of the default Git template
- **[`--git <URL>`]** Override the Git URL for the template
- **[`--subfolder <NAME>`]** Override the template subfolder
- **[`--branch <NAME>`]** Override the template branch

Behavior:

- **[requires `modules/`]** Fails unless `<workspace>/modules` already exists
- **[creates `modules/<name>`]** Uses `--name` when provided, otherwise the template name
- **[prevents duplicates]** Fails if that module directory already exists
- **[updates workspace members]** Adds generated modules to `workspace.members`
- **[promotes dependencies]** Moves new module dependency source/version metadata into `workspace.dependencies`
- **[rewrites module Cargo files]** Rewrites module dependencies to `workspace = true`
- **[inherits workspace lints]** Adds `lints.workspace = true` to generated modules if needed
- **[includes SDK crate when present]** If the generated module contains `sdk/`, it is also added as a workspace member

Examples:

```bash
cargo cyberfabric generate module --template background-worker -p /tmp/cf-demo
```

```bash
cargo cyberfabric generate module --template background-worker --name jobs -p /tmp/cf-demo
```

```bash
cargo cyberfabric generate module --template api-db-handler -p /tmp/cf-demo --local-path ~/dev/cf-template-rust --subfolder Modules/api-db-handler
```

#### `generate config`

Generate a runtime YAML config file from a built-in template.

Synopsis:

```bash
cargo cyberfabric generate config --template <dev|prod|db> [--app <APP>] [--env <ENV>] [--name <NAME>] [--path <PATH>]
```

Built-in templates:

- **[`dev`]** Development-friendly runtime config
- **[`prod`]** Production-oriented runtime config
- **[`db`]** Development config with a `database.servers.main` section

Arguments:

- **[`-t, --template <TEMPLATE>`]** Config template name
- **[`--app <APP>`]** Application name used for output filename
- **[`--env <ENV>`]** Environment name used for output filename
- **[`--name <NAME>`]** Custom output filename; `.yml` is appended when no YAML extension is provided
- **[`-p, --path <PATH>`]** Workspace root, defaults to `.`

Behavior:

- **[writes under `config/`]** Output path is `<workspace>/config/<filename>`
- **[filename resolution]** `--name` wins; otherwise `<app>-<env>.yml`, `<app>.yml`, `<env>.yml`, or `<template>.yml`
- **[prevents overwrites]** Fails if the target config file already exists
- **[runtime values only]** Generated config contains runtime settings, not dependency metadata

Examples:

```bash
cargo cyberfabric generate config --template dev --app app1 --env dev -p /tmp/cf-demo
```

```bash
cargo cyberfabric generate config --template db --name local-db -p /tmp/cf-demo
```

### `new`

Alias for `generate workspace`.

Synopsis:

```bash
cargo cyberfabric new <path> [--template <TEMPLATE>] [--verbose] [--name <NAME>] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>] [--override]
```

### `config`

Manages the YAML application config file used by `build` and `run`.

There are two branches:

- **[`config mod ...`]** Module configuration
- **[`config db ...`]** Global database server configuration

### `config mod`

Manage the `modules` section in the app config.

#### `config mod list`

List workspace modules, configured modules, and optionally known system crates.

Synopsis:

```bash
cargo cyberfabric config mod list -c <CONFIG> [-p <PATH>] [--system] [--verbose] [--registry <REGISTRY>]
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`-s, --system`]** Also print built-in system registry modules
- **[`-v, --verbose`]** Print full metadata
- **[`--registry <REGISTRY>`]** Registry used only for verbose system lookups, defaults to `crates.io`

Behavior:

- **[discovers local modules]** Scans the workspace for module crates
- **[loads configured modules]** Reads enabled modules from the config file
- **[path activation]** If `-p/--path` is provided, the CLI first changes the current working directory there before
  resolving `-c/--config`
- **[marks enabled locals]** Shows when a workspace module is enabled in config
- **[shows missing locals]** Shows when a configured module is not present in the workspace
- **[optional crates.io fetch]** If both `--system` and `--verbose` are used, the CLI fetches registry metadata and
  `src/module.rs` details from crates.io
- **[registry support]** Only `crates.io` is currently supported

Built-in system module names:

- **[`credstore`]**
- **[`file-parser`]**
- **[`api-gateway`]**
- **[`authn-resolver`]**
- **[`static-authn-plugin`]**
- **[`authz-resolver`]**
- **[`static-authz-plugin`]**
- **[`grpc-hub`]**
- **[`module-orchestrator`]**
- **[`nodes-registry`]**
- **[`oagw`]**
- **[`single-tenant-tr-plugin`]**
- **[`static-tr-plugin`]**
- **[`tenant-resolver`]**
- **[`types-registry`]**

Examples:

```bash
cargo cyberfabric config mod list -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo cyberfabric config mod list -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --system
```

```bash
cargo cyberfabric config mod list -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --system --verbose
```

#### `config mod add`

Add or update a module entry in the config file's `modules` section.

Synopsis:

```bash
cargo cyberfabric config mod add -c <CONFIG> [-p <PATH>] [--package <NAME>] [--module-version <VER>] [--default-features <BOOL>] [-F, --feature <FEATURES>]... [--dep <NAME>]... <module>
```

Arguments:

- **[`<module>`]** Module name in the config
- **[`-c, --config <CONFIG>`]** Required config file path
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`--package <NAME>`]** Override metadata package name
- **[`--module-version <VER>`]** Override metadata version
- **[`--default-features <BOOL>`]** Persist Cargo `default_features`
- **[`-F, --feature <FEATURES>`]** Feature list; accepts comma-separated values and can be repeated
- **[`--dep <NAME>`]** Metadata dependency name; repeat to add more

Behavior:

- **[upsert]** Creates or updates `modules.<module>`
- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `-c/--config` is resolved
- **[local-first discovery]** Tries to discover module metadata from the workspace
- **[remote module support]** If the module is not local, you must provide both `--package` and `--module-version`
- **[portable metadata]** Local filesystem paths are intentionally not persisted into config metadata
- **[merge semantics]** Existing metadata fields are preserved unless you explicitly override them
- **[metadata requirements]** Package and version are required in the resulting metadata, whether sourced locally or
  passed explicitly

Examples:

```bash
cargo cyberfabric config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo cyberfabric config mod add api-gateway -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml -F json,metrics -F tracing --dep authn-resolver --dep tenant-resolver
```

```bash
cargo cyberfabric config mod add credstore -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --package cf-credstore --module-version 0.4.2
```

```bash
cargo cyberfabric config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --default-features false
```

#### `config mod rm`

Remove a module from the config file's `modules` section.

Synopsis:

```bash
cargo cyberfabric config mod rm -c <CONFIG> [-p <PATH>] <module>
```

Behavior:

- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `-c/--config` is resolved
- **[strict removal]** Fails if the module is not present in config

Example:

```bash
cargo cyberfabric config mod rm background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

#### `config mod db`

Manage module-level database config under `modules.<module>.database`.

Subcommands:

- **[`add`]** Add or patch a module DB config
- **[`edit`]** Edit an existing module DB config
- **[`rm`]** Remove a module DB config

Shared DB flags:

- **[`--engine <postgres|mysql|sqlite>`]**
- **[`--dsn <DSN>`]**
- **[`--host <HOST>`]**
- **[`--port <PORT>`]**
- **[`--user <USER>`]**
- **[`--password <PASSWORD>`]**
- **[`--dbname <NAME>`]**
- **[`--params <K=V,...>`]**
- **[`--sqlite-file <FILE>`]**
- **[`--sqlite-path <PATH>`]**
- **[`--pool-max-conns <N>`]**
- **[`--pool-min-conns <N>`]**
- **[`--pool-acquire-timeout-secs <SECS>`]**
- **[`--pool-idle-timeout-secs <SECS>`]**
- **[`--pool-max-lifetime-secs <SECS>`]**
- **[`--pool-test-before-acquire <BOOL>`]**
- **[`--server <NAME>`]** Reference a named global DB server

Rules:

- **[path activation]** If `-p/--path` is provided, each subcommand changes the current working directory while Clap is
  parsing that value, before `-c/--config` is resolved
- **[payload required]** `add` and `edit` require at least one DB-related field
- **[module must exist]** `add` requires the module already exist in config and recommends `config mod add` first
- **[edit requires existing DB config]** `edit` fails if no module DB config exists yet
- **[patch semantics]** `add` and `edit` patch only the fields you provide

Examples:

```bash
cargo cyberfabric config mod db add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --server primary
```

```bash
cargo cyberfabric config mod db add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb --pool-max-conns 20
```

```bash
cargo cyberfabric config mod db edit api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --pool-acquire-timeout-secs 30 --pool-test-before-acquire true
```

```bash
cargo cyberfabric config mod db rm api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

### `config db`

Manage global database server config under `database.servers`.

Subcommands:

- **[`add`]** Add or upsert a named global DB server
- **[`edit`]** Edit an existing global DB server
- **[`rm`]** Remove an existing global DB server

Synopsis:

```bash
cargo cyberfabric config db add  -c <CONFIG> [-p <PATH>] <name> <db-flags...>
cargo cyberfabric config db edit -c <CONFIG> [-p <PATH>] <name> <db-flags...>
cargo cyberfabric config db rm   -c <CONFIG> [-p <PATH>] <name>
```

Behavior:

- **[path activation]** If `-p/--path` is provided, each subcommand changes the current working directory while Clap is
  parsing that value, before `-c/--config` is resolved
- **[server name]** Stored under `database.servers.<name>`
- **[add is upsert]** `add` creates or patches an existing server entry
- **[edit is strict]** `edit` requires the server to already exist
- **[payload required]** `add` and `edit` require at least one DB-related field
- **[cleanup]** `rm` removes the top-level `database` section if it becomes empty and `auto_provision` is unset

Examples:

```bash
cargo cyberfabric config db add primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
```

```bash
cargo cyberfabric config db edit primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --pool-max-conns 30 --pool-idle-timeout-secs 120
```

```bash
cargo cyberfabric config db add local-sqlite -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine sqlite --sqlite-path /tmp/cf-demo/dev.db
```

```bash
cargo cyberfabric config db rm primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

### `src`

Resolve Rust source for a crate/module/item query from local workspace metadata, the local source cache, or crates.io.

Synopsis:

```bash
cargo cyberfabric src [--path <PATH>] [--registry <REGISTRY>] [--verbose] [--libs] [--version <VERSION>] [--clean] [<query>]
```

Arguments:

- **[`-p, --path <PATH>`]** Workspace or crate to inspect, defaults to `.`
- **[`--registry <REGISTRY>`]** Registry fallback, defaults to `crates.io`
- **[`-v, --verbose`]** Print resolution metadata before the source
- **[`-l, --libs`]** Print `library_name -> package_name` mappings for a package query instead of source
- **[`--version <VERSION>`]** Resolve a specific crate version after metadata/cache lookup misses
- **[`--clean`]** Remove the source cache for the selected registry before resolving
- **[`[<query>]`]** Rust path to resolve, starting with the package name; omitted only when `--clean` is used by itself

Supported query examples:

- **[`cf-modkit`]**
- **[`tokio::sync`]**
- **[`cf-modkit::gts::plugin::BaseModkitPluginV1`]**
- **[`cf-modkit::gts::schemas::get_core_gts_schemas`]**

Behavior:

- **[query requirement]** A query is required unless `--clean` is passed by itself
- **[package-only libs mode]** `--libs` requires a package-only query such as `cf-modkit`
- **[local resolution first]** Tries workspace metadata before hitting the network
- **[cache-first registry fallback]** Reuses cached crate sources before downloading from the registry
- **[crates.io fallback]** Downloads and extracts crate source if local resolution and cache lookup both fail
- **[exact version fallback]** `--version` pins the registry/cache fallback to that exact crate version
- **[recursive re-export resolution]** Follows re-exports across `crate`, `self`, `super`, and dependency boundaries
  until it reaches the final source
- **[library mapping output]** `--libs` prints the Rust source-code library name on the left and the Cargo package
  name on the right, including renamed dependencies like `modkit_macros -> cf-modkit-macros`
- **[cache location]** Registry sources are cached under the OS temp directory in `cyberfabric-docs-cache/<registry>/` (legacy name)
- **[cache cleaning]** `--clean` removes the selected registry cache before resolution
- **[source output]** Prints the resolved Rust source to stdout
- **[verbose metadata]** Also prints query, package, library, version, manifest path, and source path
- **[registry support]** Only `crates.io` is supported today

Examples:

```bash
cargo cyberfabric src -p /tmp/cf-demo cf-modkit
```

```bash
cargo cyberfabric src cf-modkit::module
```

```bash
cargo cyberfabric src --verbose tokio::sync
```

```bash
cargo cyberfabric src --libs cf-modkit
```

```bash
cargo cyberfabric src --version 1.0.217 serde::de::Deserialize
```

```bash
cargo cyberfabric src --clean
```

```bash
cargo cyberfabric src --clean -p /tmp/cf-demo tokio::sync
```

### `help`

Schema, topic, and source-code help for developers and LLMs. Groups three subcommands under a
single discoverable entry point.

#### `help schema`

Print the schema for manifest, config, or module formats.

Synopsis:

```bash
cargo cyberfabric help schema <manifest|config|module> [--section <SECTION>]
```

Arguments:

- **[`<target>`]** Schema target: `manifest`, `config`, or `module`
- **[`--section <SECTION>`]** Drill into a specific section of the schema

Behavior:

- **[overview mode]** Without `--section`, prints a top-level overview of the schema
- **[section mode]** With `--section`, prints detailed field documentation for that section
- **[manifest sections]** `workspace`, `apps`, `templates`
- **[config sections]** `server`, `database`, `logging`, `opentelemetry`, `modules`
- **[module sections]** None (only the overview is available)

Examples:

```bash
cargo cyberfabric help schema manifest
```

```bash
cargo cyberfabric help schema config --section database
```

```bash
cargo cyberfabric help schema module
```

#### `help src`

Alias for the top-level `src` command. Resolves Rust source code from a crate.

Synopsis:

```bash
cargo cyberfabric help src [--path <PATH>] [--registry <REGISTRY>] [--verbose] [--libs] [--version <VERSION>] [--clean] [<query>]
```

Behavior identical to `src`; see the `src` section above.

#### `help topic`

Print operational documentation for a named topic.

Synopsis:

```bash
cargo cyberfabric help topic <TOPIC>
```

Available topics:

- **[`manifest`]** Overview of `Cyberware.toml` and manifest-driven workflows
- **[`module-refs`]** How local and remote modules are referenced
- **[`generated-server`]** How the ephemeral generated server project works
- **[`fips`]** FIPS mode activation and usage
- **[`otel`]** OpenTelemetry activation and runtime configuration

Examples:

```bash
cargo cyberfabric help topic manifest
```

```bash
cargo cyberfabric help topic generated-server
```

```bash
cargo cyberfabric help topic otel
```

### `tools`

Install or upgrade a small set of Rust tooling dependencies.

Known tool names:

- **[`rustup`]**
- **[`cargofmt`]** Installs the `rustfmt` rustup component
- **[`clippy`]**

Synopsis:

```bash
cargo cyberfabric tools (--all | --install <tool,...>) [--upgrade] [--yolo] [--verbose]
```

Arguments:

- **[`-a, --all`]** Select all known tools
- **[`--install <tool,...>`]** Comma-separated tool names
- **[`-u, --upgrade`]** Upgrade instead of initial install
- **[`-y, --yolo`]** Skip confirmation prompts
- **[`-v, --verbose`]** Show subprocess output

Behavior:

- **[selection required]** You must pass either `--all` or `--install`
- **[interactive by default]** Without `--yolo`, the command prompts before installing/upgrading
- **[rustup bootstrap]** If `rustup` is missing, the CLI can attempt to install it
- **[component installs]** `cargofmt` and `clippy` are installed through `rustup component add`
- **[upgrade mode]** Selected `rustup` upgrades via `rustup self update`; selected components upgrade via
  `rustup update`

Examples:

```bash
cargo cyberfabric tools --all
```

```bash
cargo cyberfabric tools --install clippy,cargofmt --yolo
```

```bash
cargo cyberfabric tools --install rustup,clippy --upgrade --verbose
```

### `run`

Generate a server project under the manifest `<workspace.generated-dir>/<name>` and run it.

Synopsis:

```bash
cargo cyberfabric run --app <APP> --env <ENV> [--manifest <Cyberware.toml>] [-p <PATH>] [--name <NAME>] [--watch|--no-watch] [--otel|--no-otel] [--fips|--no-fips] [--release|--no-release] [--clean|--no-clean] [--dry-run]
```

Arguments:

- **[`--manifest <PATH>`]** Manifest file, defaults to `Cyberware.toml`
- **[`--app <APP> --env <ENV>`]** Required manifest app/environment selection
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`--name <NAME>`]** Override the generated server project and binary name; defaults to the config filename stem
- **[`-w, --watch` / `--no-watch`]** Override manifest watch policy on or off
- **[`--otel` / `--no-otel`]** Override manifest OpenTelemetry policy on or off
- **[`--fips` / `--no-fips`]** Override manifest FIPS policy on or off
- **[`-r, --release` / `--no-release`]** Override manifest build profile to release or non-release
- **[`--clean` / `--no-clean`]** Override manifest clean policy on or off
- **[`--dry-run`]** Generate the project structure and print the generated files without building or running

Behavior:

- **[name resolution]** Uses the manifest build policy name when present, otherwise `<app>-<env>`
- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `Cyberware.toml` is resolved and the generated project directory is created
- **[generates server structure]** Writes `<generated-dir>/<name>/Cargo.toml`,
  `<generated-dir>/<name>/.cargo/config.toml`, and `<generated-dir>/<name>/src/main.rs`; `generated-dir` comes from
  manifest `workspace.generated-dir` and defaults to `.cyberware`
- **[runtime config handoff]** The generated `src/main.rs` reads the config path from `CF_CLI_CONFIG`, and
  `cargo cyberfabric run` sets that environment variable automatically before invoking `cargo run`
- **[dry run]** `--dry-run` writes the generated project structure under `<generated-dir>/<name>/` and prints JSON with the
  generated project directory plus each generated file path and contents; it does not invoke Cargo build/run
- **[manifest mode]** Reads generation dependencies, runtime config path, and policies from `Cyberware.toml`; runtime YAML
  stays focused on server configuration
- **[exclusive boolean overrides]** Boolean flag pairs are mutually exclusive. Use either the positive or negative form,
  not both; for example, `--otel --no-otel` is rejected. When neither side is present, the manifest policy is used.
- **[runs inside generated project]** Executes `cargo run` in `<generated-dir>/<name>/`
- **[watch mode]** Restarts on config changes, workspace `Cargo.toml` changes, and changes in path-based dependencies
- **[dependency watch management]** Reconciles watched dependency paths when config dependencies change
- **[manual generated-project execution]** If you invoke the generated project or compiled binary yourself instead of
  using `cargo cyberfabric run`, you must set `CF_CLI_CONFIG` manually

Examples:

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev
```

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --watch
```

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --otel --fips --release --clean
```

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --name demo-server
```

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --dry-run
```

```bash
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --manifest /tmp/cf-demo/Cyberware.toml
```

### `manifest`

Inspect and validate Cyberware manifest files.

Synopsis:

```bash
cargo cyberfabric manifest [-p <PATH>] [--manifest <Cyberware.toml>] validate [--format table|json]
cargo cyberfabric manifest [-p <PATH>] [--manifest <Cyberware.toml>] ls [--format table|json]
```

Behavior:

- **[path activation]** If `-p/--path` is provided, relative manifest paths are resolved from that workspace directory.
- **[validate]** Parses the manifest and resolves every app/environment entry.
- **[ls]** Lists configured app/environment pairs.
- **[generated structure]** Use `build --dry-run` or `run --dry-run` to write and print the generated project structure.

### `build`

Generate a server project under the manifest `<workspace.generated-dir>/<name>` and build it.

Synopsis:

```bash
cargo cyberfabric build --app <APP> --env <ENV> [--manifest <Cyberware.toml>] [-p <PATH>] [--name <NAME>] [--otel|--no-otel] [--fips|--no-fips] [--release|--no-release] [--clean|--no-clean] [--dry-run]
```

Arguments:

- **[`--manifest <PATH>`]** Manifest file, defaults to `Cyberware.toml`
- **[`--app <APP> --env <ENV>`]** Required manifest app/environment selection
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`--name <NAME>`]** Override the generated server project and binary name; defaults to the config filename stem
- **[`--otel` / `--no-otel`]** Override manifest OpenTelemetry policy on or off
- **[`--fips` / `--no-fips`]** Override manifest FIPS policy on or off
- **[`-r, --release` / `--no-release`]** Override manifest build profile to release or non-release
- **[`--clean` / `--no-clean`]** Override manifest clean policy on or off
- **[`--dry-run`]** Generate the project structure and print the generated files without building

Behavior:

- **[generates before build]** Recreates the generated server project before invoking Cargo
- **[name resolution]** Uses the manifest build policy name when present, otherwise `<app>-<env>`
- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `Cyberware.toml` is resolved and the generated project directory is created
- **[exclusive boolean overrides]** Boolean flag pairs are mutually exclusive. Use either the positive or negative form,
  not both; for example, `--clean --no-clean` is rejected. When neither side is present, the manifest policy is used.
- **[manifest mode]** Reads module dependency metadata, runtime config path, and build/run policy from `Cyberware.toml`
  instead of runtime YAML module metadata
- **[builds inside generated project]** Executes `cargo build` in `<generated-dir>/<name>/`
- **[runtime config source]** The generated server no longer embeds the config path; the resulting binary reads it from
  `CF_CLI_CONFIG` when you execute it
- **[dry run]** `--dry-run` writes the generated project structure under `<generated-dir>/<name>/` and prints JSON with the
  generated project directory plus each generated file path and contents; it does not invoke Cargo build
- **[manual generated-project execution]** If you later run the generated project or binary outside the CLI, you must
  set `CF_CLI_CONFIG` yourself

Examples:

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env dev
```

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env dev --release
```

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env dev --otel --fips --clean
```

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env dev --name demo-server
```

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env prod
```

```bash
cargo cyberfabric build -p /tmp/cf-demo --app app1 --env prod --dry-run
```

### `deploy`

Generate a server project under the default generated directory and build a Docker image with the workspace `Dockerfile`.

Synopsis:

```bash
cargo cyberfabric deploy -c <CONFIG> [-p <PATH>] [--manifest <Cargo.toml>] [--debug] [--dockerfile] [--args <KEY=VALUE>]...
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path; copied into the image and used as the runtime
  `CF_CLI_CONFIG` target
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`-m, --manifest <Cargo.toml>`]** Optional Cargo manifest to build instead of generating the server project;
  the path must point to a file named `Cargo.toml`
- **[`--debug`]** Docker build mode; defaults to release mode. Use this flag to build in debug mode.
- **[`--dockerfile <Dockerfile>`]** Dockerfile path to use instead of the default(Dockerfile from cwd)
- **[`--args <KEY=VALUE>`]** Dockerfile `ARG` override passed as `docker build --build-arg`; repeat for multiple
  overrides

Behavior:

- **[generates by default]** Without `--manifest`, recreates the generated server project from the config, matching
  `build` and `run`
- **[manifest override]** With `--manifest`, does not generate the server project; Docker builds the provided
  manifest instead and uses its `package.name` as the artifact name
- **[Dockerfile bootstrap]** If `Dockerfile` is missing from the selected workspace root, writes the shared CLI
  Dockerfile there before running Docker
- **[build context requirement]** The config file and selected manifest must be inside the workspace root because Docker
  can only copy files from the build context
- **[Docker args]** The CLI provides `BUILDER_MANIFEST`, `BUILD_MODE`, `ARTIFACT_NAME`, `LOCAL_CONFIG_PATH`, and
  `CONFIG_EXT`; repeated `--args` values are appended afterward so they can override Dockerfile arguments

Examples:

```bash
cargo cyberfabric deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo cyberfabric deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --debug
```

```bash
cargo cyberfabric deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --manifest /tmp/cf-demo/Cargo.toml
```

```bash
cargo cyberfabric deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --args BUILDER_FLAGS="--features metrics"
```

### `lint`

Run workspace linting helpers from the selected workspace directory.

Synopsis:

```bash
cargo cyberfabric lint [-p <PATH>] [--all] [--fmt] [--clippy] [--strict] [--dylint]
```

Arguments:

- **[`-p, --path <PATH>`]** Optional workspace directory; changes the current working directory while Clap parses it
- **[`--all`]** Runs the default lint suites; this is also the default when neither `--fmt`, `--clippy`, nor `--dylint`
  is passed
- **[`--fmt`]** Runs `cargo fmt --check --all`; if passed by itself, it disables the default implicit `--all`
- **[`--clippy`]** Runs workspace Clippy checks; if passed by itself, it disables the default implicit `--all`
- **[`--strict`]** Turns Clippy warnings into errors; valid only when Clippy is selected explicitly or through `--all`
- **[`--dylint`]** Runs embedded Dylint rules against the workspace rooted at the current or selected directory

Behavior:

- **[path activation]** If `-p/--path` is provided, it changes the current working directory
- **[default lint selection]** With no explicit lint-selection flags, `lint` behaves as if `--all` was enabled
- **[explicit selection disables default all]** Passing `--fmt`, `--clippy`, and/or `--dylint` opts into just those
  requested lint suites unless `--all` is also provided
- **[workspace formatting check]** `--fmt` runs `cargo fmt --check --all`
- **[workspace Clippy]** Clippy runs as `cargo clippy --workspace --all-targets --all-features`. The `--all-features`
  flag ensures every feature-gated code path is checked. The workspace currently has no mutually exclusive features, so
  enabling all features simultaneously is safe.
- **[strict scope]** `--strict` is rejected unless Clippy is active through `--clippy` or `--all`
- **[workspace-scoped dylint]** Dylint resolves the workspace from the current working directory, so `-p/--path` is the
  way to lint another workspace without manually changing directories
- **[toolchain bootstrap]** Before running Dylint, the CLI ensures the toolchains required by the embedded lint dylibs
  are installed

Examples:

```bash
cargo cyberfabric lint
```

```bash
cargo cyberfabric lint --clippy --strict
```

```bash
cargo cyberfabric lint --fmt
```

```bash
cargo cyberfabric lint --dylint
```

```bash
cargo cyberfabric lint -p /tmp/cf-demo --dylint
```

### `list`

Inspect workspace modules, system modules, and project state.

#### `list modules`

List all modules — both system-registry and workspace-discovered — in a single unified view.

Synopsis:

```bash
cargo cyberfabric list modules [-p <PATH>] [--verbose] [--registry crates.io] [--format table|json|yaml|toml]
```

Arguments:

- **[`-p, --path <PATH>`]** Optional workspace directory; changes the current working directory while Clap parses it
- **[`-v, --verbose`]** Show full metadata for both system and local modules (fetches registry metadata for system
  modules)
- **[`--registry <REGISTRY>`]** Registry to query for system-crate metadata; defaults to `crates.io`
- **[`-f, --format <FORMAT>`]** Output format; defaults to `table`. Only `table` is currently implemented

Behavior:

- **[combined output]** Prints system modules first, then workspace modules, separated by a blank line
- **[config-independent]** Does not require a `-c/--config` file

Examples:

```bash
cargo cyberfabric list modules
```

```bash
cargo cyberfabric list modules -p /tmp/cf-demo --verbose
```

#### `list local-modules`

List workspace-discovered modules by scanning Cargo metadata for crates that contain `src/module.rs`.

Synopsis:

```bash
cargo cyberfabric list local-modules [-p <PATH>] [--verbose] [--format table|json|yaml|toml]
```

Arguments:

- **[`-p, --path <PATH>`]** Optional workspace directory; changes the current working directory while Clap parses it
- **[`-v, --verbose`]** Show full module metadata: package, version, path, features, deps, and capabilities
- **[`-f, --format <FORMAT>`]** Output format; defaults to `table`. Supported values: `table`, `json`, `yaml`, `toml`.
  Only `table` is currently implemented; other formats will be added in a future release

Behavior:

- **[workspace scanning]** Runs `cargo metadata --no-deps` and discovers crates with a `src/module.rs` target
- **[config-independent]** Does not require a `-c/--config` file; only inspects the workspace
- **[sorted output]** Modules are listed alphabetically by name
- **[verbose metadata]** With `--verbose`, prints package name, version, path, default-features, features, deps, and
  capabilities for each module

Examples:

```bash
cargo cyberfabric list local-modules
```

```bash
cargo cyberfabric list local-modules -p /tmp/cf-demo --verbose
```

#### `list system-modules`

List built-in system modules from the CyberFabric registry.

Synopsis:

```bash
cargo cyberfabric list system-modules [--verbose] [--registry crates.io] [--format table|json|yaml|toml]
```

Arguments:

- **[`-v, --verbose`]** Fetch registry metadata and show latest version, features, deps, and capabilities for each
  system module
- **[`--registry <REGISTRY>`]** Registry to query for system-crate metadata; defaults to `crates.io`
- **[`-f, --format <FORMAT>`]** Output format; defaults to `table`. Supported values: `table`, `json`, `yaml`, `toml`.
  Only `table` is currently implemented; other formats will be added in a future release

Behavior:

- **[static list]** Without `--verbose`, prints the known system module names and their crate names from a compiled-in
  registry
- **[registry fetch]** With `--verbose`, fetches crate metadata and `src/module.rs` from the registry for each system
  module (concurrent, capped at 4 in-flight requests)
- **[config-independent]** Does not require a workspace or config file

Examples:

```bash
cargo cyberfabric list system-modules
```

```bash
cargo cyberfabric list system-modules --verbose
```

#### `list configs`

List configuration files, their inferred app/environment links, and runtime sections.

**Currently unimplemented** — blocked on the manifest-first design (`Cyberware.toml`). Requires manifest
parsing to resolve app/environment links and runtime sections.

Synopsis:

```bash
cargo cyberfabric list configs [--format table|json|yaml|toml]
```

#### `list apps`

List apps, environments, and build outputs.

**Currently unimplemented** — blocked on the manifest-first design (`Cyberware.toml`). Requires manifest
parsing to enumerate apps, environments, and build outputs.

Synopsis:

```bash
cargo cyberfabric list apps [--format table|json|yaml|toml]
```

### `test`

Declared in the CLI but **currently unimplemented**.

Synopsis:

```bash
cargo cyberfabric test [--e2e] [--module <NAME>] [--coverage]
```

Arguments:

- **[`--e2e`]**
- **[`--module <NAME>`]**
- **[`--coverage`]**

Current status:

- **[placeholder only]** Calling this subcommand currently reaches `unimplemented!("Not implemented yet")`

## Practical End-to-End Flows

### Create a workspace and run it

```bash
cargo cyberfabric {new|generate workspace} /tmp/cf-demo
cargo cyberfabric generate module --template background-worker -p /tmp/cf-demo
cargo cyberfabric generate config --template dev --app app1 --env dev -p /tmp/cf-demo
cargo cyberfabric config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/app1-dev.yml
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev
```

### Add a module and wire a shared DB server

```bash
cargo cyberfabric generate module --template api-db-handler -p /tmp/cf-demo
cargo cyberfabric config db add primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
cargo cyberfabric config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo cyberfabric config mod db add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --server primary
cargo cyberfabric run -p /tmp/cf-demo --app app1 --env dev --watch
```

### Inspect source for a dependency

```bash
cargo cyberfabric src --verbose tokio::sync
```

## Important Caveats

- **[`-c/--config` is mandatory]** For `config ...` and `deploy`; `build` and `run` use `Cyberware.toml` instead
- **[generated servers expect `CF_CLI_CONFIG`]** `cargo cyberfabric run` sets it for you, but manual execution of
  generated project directory or its compiled binary must provide it explicitly
- **[`lint --dylint` needs the feature build]** Without the `dylint-rules` feature enabled, it currently reaches
  an error
- **[`lint --strict` depends on Clippy]** Use it together with `--clippy` or `--all`
- **[`test` is not ready]** It is part of the CLI surface but currently panics at runtime
- **[`tools` can mutate your system]** It may install `rustup` or rustup components
- **[`src --registry`]** Only `crates.io` is supported
- **[`src`]** Accepts a single query, and that query is only optional when `--clean` is used by itself
- **[`config mod add`]** Remote modules require both `--package` and `--module-version`
- **[`config mod db add`]** The module must already exist in config

## Quick Reference

```bash
cargo cyberfabric {new|generate workspace} <path> [--template <template>] [--name <name>]
cargo cyberfabric generate module --template <background-worker|api-db-handler|api-gateway> [--name <name>] [-p <workspace>]
cargo cyberfabric generate config --template <dev|prod|db> [--app <app>] [--env <env>] [--name <name>] [-p <workspace>]

cargo cyberfabric config mod list [-p <workspace>] -c <config>
cargo cyberfabric config mod add <module> [-p <workspace>] -c <config>
cargo cyberfabric config mod rm <module> [-p <workspace>] -c <config>
cargo cyberfabric config mod db add <module> [-p <workspace>] -c <config> ...
cargo cyberfabric config mod db edit <module> [-p <workspace>] -c <config> ...
cargo cyberfabric config mod db rm <module> [-p <workspace>] -c <config>

cargo cyberfabric config db add <name> [-p <workspace>] -c <config> ...
cargo cyberfabric config db edit <name> [-p <workspace>] -c <config> ...
cargo cyberfabric config db rm <name> [-p <workspace>] -c <config>

cargo cyberfabric list modules [-p <workspace>] [--verbose] [--registry crates.io] [-f table|json|yaml|toml]
cargo cyberfabric list local-modules [-p <workspace>] [--verbose] [-f table|json|yaml|toml]
cargo cyberfabric list system-modules [--verbose] [--registry crates.io] [-f table|json|yaml|toml]
cargo cyberfabric list configs [-f table|json|yaml|toml]           # unimplemented
cargo cyberfabric list apps [-f table|json|yaml|toml]              # unimplemented

cargo cyberfabric src [-p <path>] [--version <version>] [--clean] [<query>]
cargo cyberfabric lint [-p <workspace>] [--all] [--clippy] [--strict] [--dylint]
cargo cyberfabric tools --all
cargo cyberfabric run [-p <workspace>] --app <app> --env <env> [--manifest <Cyberware.toml>] [--name <name>] [--watch]
cargo cyberfabric build [-p <workspace>] --app <app> --env <env> [--manifest <Cyberware.toml>] [--name <name>]
cargo cyberfabric deploy [-p <workspace>] -c <config> [--manifest <Cargo.toml>] [--args <KEY=VALUE>]...
