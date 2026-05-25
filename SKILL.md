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
cargo cyberfabric init /tmp/my-app
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
├── init
├── mod
│   └── add
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
├── docs
├── lint
├── list
│   ├── local-modules
│   └── system-modules
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
- **[`-c, --config <PATH>`]** Config file path. This is required for `config ...`, `build`, `run`, and `deploy` commands
  because there is no default. For `build` and `run`, the CLI forwards this path to the generated server through the
  `CF_CLI_CONFIG` environment variable.
- **[`--name <NAME>`]** For `build` and `run`, overrides the generated server project and binary name that would
  otherwise default to the config filename stem.
- **[`-v, --verbose`]** Usually enables more logging or richer output.
- **[name validation]** Config-managed names for modules, DB servers, and generated server names only allow letters,
  numbers, `-`, and `_`.

## What the Tool Manages

From the current implementation, the CLI is mainly for:

- **[workspace scaffolding]** Initialize a CyberFabric workspace and add module templates
- **[config management]** Enable modules and patch YAML config sections
- **[server generation]** Generate a runnable Cargo project under `.cyberfabric/<name>/`
- **[build/run/deploy]** Build, run, or package that generated server as a Docker image
- **[source inspection]** Resolve Rust source for crates/items through workspace metadata or crates.io
- **[module inspection]** List workspace-discovered and system-registry modules
- **[tool bootstrap]** Install or upgrade `rustup`, `cargofmt`, and `clippy`

## Top-Level Commands

### `init`

Initialize a new project from the default CyberFabric template repo or a local template.

Synopsis:

```bash
cargo cyberfabric init <path> [--verbose] [--name <NAME>] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>]
```

Arguments:

- **[`<path>`]** Target directory to initialize
- **[`-v, --verbose`]** Verbose output from `cargo-generate`
- **[`-n, --name <NAME>`]** Override the generated project name; inferred from the final path segment by default
- **[`--local-path <PATH>`]** Use a local template directory instead of Git
- **[`--git <URL>`]** Template Git URL, defaults to `https://github.com/cyberfabric/cf-template-rust`
- **[`--subfolder <NAME>`]** Template subfolder, defaults to `Init`
- **[`--branch <NAME>`]** Git branch, defaults to `main`

Behavior:

- **[creates target directory]** If it does not exist
- **[fails on file path]** Errors if `<path>` already exists and is not a directory
- **[uses directory name as project name]** The final path segment becomes the generated project name unless `--name` is
  provided
- **[forces git init]** Template generation runs with Git initialization enabled

Examples:

```bash
cargo cyberfabric init /tmp/cf-demo
```

```bash
cargo cyberfabric init /tmp/cf-demo --git https://github.com/cyberfabric/cf-template-rust --branch main --subfolder Init
```

```bash
cargo cyberfabric init /tmp/cf-demo --local-path ~/dev/cf-template-rust
```

### `mod`

Scaffolds workspace content from templates.

#### `mod add`

Generate a module template inside an existing workspace's `modules/` directory and wire Cargo workspace dependencies.

Synopsis:

```bash
cargo cyberfabric mod add [--path <PATH>] [--verbose] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>] <template>
```

Available templates:

- **[`background-worker`]** Background worker module template
- **[`api-db-handler`]** API/database handler module template
- **[`rest-gateway`]** REST gateway module template

Arguments:

- **[`<template>`]** One of the three value-enum names above
- **[`-p, --path <PATH>`]** Workspace root, defaults to `.`
- **[`-v, --verbose`]** Verbose template generation output
- **[`--local-path <PATH>`]** Local template root instead of Git
- **[`--git <URL>`]** Template repo URL, defaults to `https://github.com/cyberfabric/cf-template-rust`
- **[`--subfolder <NAME>`]** Template subfolder root, defaults to `Modules`
- **[`--branch <NAME>`]** Template branch, defaults to `main`

Behavior:

- **[requires `modules/`]** Fails unless `<workspace>/modules` already exists
- **[creates `modules/<template>`]** The generated module name matches the template name
- **[prevents duplicates]** Fails if that module directory already exists
- **[updates workspace members]** Adds generated modules to `workspace.members`
- **[promotes dependencies]** Moves new module dependency source/version metadata into `workspace.dependencies`
- **[rewrites module Cargo files]** Rewrites module dependencies to `workspace = true`
- **[inherits workspace lints]** Adds `lints.workspace = true` to generated modules if needed
- **[includes SDK crate when present]** If the generated module contains `sdk/`, it is also added as a workspace member

Examples:

```bash
cargo cyberfabric mod add background-worker -p /tmp/cf-demo
```

```bash
cargo cyberfabric mod add rest-gateway -p /tmp/cf-demo --verbose
```

```bash
cargo cyberfabric mod add api-db-handler -p /tmp/cf-demo --local-path ~/dev/cf-template-rust --subfolder Modules
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
cargo cyberfabric config mod add rest-gateway -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml -F json,metrics -F tracing --dep authn-resolver --dep tenant-resolver
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

### `docs`

Resolve Rust source for a crate/module/item query from local workspace metadata, the local docs cache, or crates.io.

Synopsis:

```bash
cargo cyberfabric docs [--path <PATH>] [--registry <REGISTRY>] [--verbose] [--libs] [--version <VERSION>] [--clean] [<query>]
```

Arguments:

- **[`-p, --path <PATH>`]** Workspace or crate to inspect, defaults to `.`
- **[`--registry <REGISTRY>`]** Registry fallback, defaults to `crates.io`
- **[`-v, --verbose`]** Print resolution metadata before the source
- **[`-l, --libs`]** Print `library_name -> package_name` mappings for a package query instead of source
- **[`--version <VERSION>`]** Resolve a specific crate version after metadata/cache lookup misses
- **[`--clean`]** Remove the docs cache for the selected registry before resolving
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
- **[cache location]** Registry sources are cached under the OS temp directory in `cyberfabric-docs-cache/<registry>/`
- **[cache cleaning]** `--clean` removes the selected registry cache before resolution
- **[source output]** Prints the resolved Rust source to stdout
- **[verbose metadata]** Also prints query, package, library, version, manifest path, and source path
- **[registry support]** Only `crates.io` is supported today

Examples:

```bash
cargo cyberfabric docs -p /tmp/cf-demo cf-modkit
```

```bash
cargo cyberfabric docs cf-modkit::module
```

```bash
cargo cyberfabric docs --verbose tokio::sync
```

```bash
cargo cyberfabric docs --libs cf-modkit
```

```bash
cargo cyberfabric docs --version 1.0.217 serde::de::Deserialize
```

```bash
cargo cyberfabric docs --clean
```

```bash
cargo cyberfabric docs --clean -p /tmp/cf-demo tokio::sync
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

Generate a server project under `.cyberfabric/<name>/` and run it.

Synopsis:

```bash
cargo cyberfabric run -c <CONFIG> [-p <PATH>] [--name <NAME>] [--watch] [--otel] [--fips] [--release] [--clean]
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`--name <NAME>`]** Override the generated server project and binary name; defaults to the config filename stem
- **[`-w, --watch`]** Re-run when watched inputs change
- **[`--otel`]** Pass Cargo feature `otel`
- **[`--fips`]** Pass Cargo feature `fips`
- **[`-r, --release`]** Use release mode
- **[`--clean`]** Remove `.cyberfabric/<name>/Cargo.lock` before running

Behavior:

- **[name resolution]** Uses the config filename stem by default, so `config/quickstart.yml` generates under
  `.cyberfabric/quickstart/`; `--name` overrides that default
- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `-c/--config` is resolved and `.cyberfabric/<name>/` is generated
- **[generates server structure]** Writes `.cyberfabric/<name>/Cargo.toml`, `.cyberfabric/<name>/.cargo/config.toml`,
  and `.cyberfabric/<name>/src/main.rs`
- **[runtime config handoff]** The generated `src/main.rs` reads the config path from `CF_CLI_CONFIG`, and
  `cargo cyberfabric run` sets that environment variable automatically before invoking `cargo run`
- **[loads config dependencies]** Builds dependencies from the config and local module metadata
- **[feature passthrough]** `--otel` and `--fips` enable the generated project's matching Cargo features
- **[runs inside `.cyberfabric/<name>`]** Executes `cargo run` in the generated directory
- **[watch mode]** Restarts on config changes, workspace `Cargo.toml` changes, and changes in path-based dependencies
- **[dependency watch management]** Reconciles watched dependency paths when config dependencies change
- **[manual generated-project execution]** If you invoke the generated project or compiled binary yourself instead of
  using `cargo cyberfabric run`, you must set `CF_CLI_CONFIG` manually

Examples:

```bash
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --watch
```

```bash
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --otel --fips --release --clean
```

```bash
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --name demo-server
```

### `build`

Generate a server project under `.cyberfabric/<name>/` and build it.

Synopsis:

```bash
cargo cyberfabric build -c <CONFIG> [-p <PATH>] [--name <NAME>] [--otel] [--fips] [--release] [--clean]
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`--name <NAME>`]** Override the generated server project and binary name; defaults to the config filename stem
- **[`--otel`]** Pass Cargo feature `otel`
- **[`--fips`]** Pass Cargo feature `fips`
- **[`-r, --release`]** Use release mode
- **[`--clean`]** Remove `.cyberfabric/<name>/Cargo.lock` before building

Behavior:

- **[generates before build]** Recreates the generated server project before invoking Cargo
- **[name resolution]** Uses the config filename stem by default, so `config/quickstart.yml` builds from
  `.cyberfabric/quickstart/`; `--name` overrides that default
- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `-c/--config` is resolved and `.cyberfabric/<name>/` is generated
- **[feature passthrough]** `--otel` and `--fips` enable the generated project's matching Cargo features
- **[builds inside `.cyberfabric/<name>`]** Executes `cargo build` in the generated directory
- **[runtime config source]** The generated server no longer embeds the config path; the resulting binary reads it from
  `CF_CLI_CONFIG` when you execute it
- **[manual generated-project execution]** If you later run the generated project or binary outside the CLI, you must
  set `CF_CLI_CONFIG` yourself

Examples:

```bash
cargo cyberfabric build -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo cyberfabric build -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --release
```

```bash
cargo cyberfabric build -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --otel --fips --clean
```

```bash
cargo cyberfabric build -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --name demo-server
```

### `deploy`

Generate a server project under `.cyberfabric/<name>/` and build a Docker image with the workspace `Dockerfile`.

Synopsis:

```bash
cargo cyberfabric deploy -c <CONFIG> [-p <PATH>] [--manifest <Cargo.toml>] [--debug] [--dockerfile] [--args <KEY=VALUE>]...
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path; copied into the image and used as the runtime
  `CF_CLI_CONFIG` target
- **[`-p, --path <PATH>`]** Optional workspace directory
- **[`-m, --manifest <Cargo.toml>`]** Optional Cargo manifest to build instead of generating `.cyberfabric/<name>/`;
  the path must point to a file named `Cargo.toml`
- **[`--debug`]** Docker build mode; defaults to release mode. Use this flag to build in debug mode.
- **[`--dockerfile <Dockerfile>`]** Dockerfile path to use instead of the default(Dockerfile from cwd)
- **[`--args <KEY=VALUE>`]** Dockerfile `ARG` override passed as `docker build --build-arg`; repeat for multiple
  overrides

Behavior:

- **[generates by default]** Without `--manifest`, recreates the generated server project from the config, matching
  `build` and `run`
- **[manifest override]** With `--manifest`, does not generate `.cyberfabric/<name>/`; Docker builds the provided
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
cargo cyberfabric init /tmp/cf-demo
cargo cyberfabric mod add background-worker -p /tmp/cf-demo
cargo cyberfabric config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

### Add a module and wire a shared DB server

```bash
cargo cyberfabric mod add api-db-handler -p /tmp/cf-demo
cargo cyberfabric config db add primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
cargo cyberfabric config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo cyberfabric config mod db add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --server primary
cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --watch
```

### Inspect source for a dependency

```bash
cargo cyberfabric docs --verbose tokio::sync
```

## Important Caveats

- **[`-c/--config` is mandatory]** For `config ...`, `build`, `run`, and `deploy`
- **[generated servers expect `CF_CLI_CONFIG`]** `cargo cyberfabric run` sets it for you, but manual execution of
  `.cyberfabric/<name>/` or its compiled binary must provide it explicitly
- **[`lint --dylint` needs the feature build]** Without the `dylint-rules` feature enabled, it currently reaches
  an error
- **[`lint --strict` depends on Clippy]** Use it together with `--clippy` or `--all`
- **[`test` is not ready]** It is part of the CLI surface but currently panics at runtime
- **[`tools` can mutate your system]** It may install `rustup` or rustup components
- **[`docs --registry`]** Only `crates.io` is supported
- **[`docs`]** Accepts a single query, and that query is only optional when `--clean` is used by itself
- **[`config mod add`]** Remote modules require both `--package` and `--module-version`
- **[`config mod db add`]** The module must already exist in config

## Quick Reference

```bash
cargo cyberfabric init <path> [--name <name>]
cargo cyberfabric mod add <background-worker|api-db-handler|rest-gateway> [-p <workspace>]

cargo cyberfabric config mod list [-p <workspace>] -c <config>
cargo cyberfabric config mod add <module> [-p <workspace>] -c <config>
cargo cyberfabric config mod rm <module> [-p <workspace>] -c <config>
cargo cyberfabric config mod db add <module> [-p <workspace>] -c <config> ...
cargo cyberfabric config mod db edit <module> [-p <workspace>] -c <config> ...
cargo cyberfabric config mod db rm <module> [-p <workspace>] -c <config>

cargo cyberfabric config db add <name> [-p <workspace>] -c <config> ...
cargo cyberfabric config db edit <name> [-p <workspace>] -c <config> ...
cargo cyberfabric config db rm <name> [-p <workspace>] -c <config>

cargo cyberfabric list local-modules [-p <workspace>] [--verbose] [-f table|json|yaml|toml]
cargo cyberfabric list system-modules [--verbose] [--registry crates.io] [-f table|json|yaml|toml]

cargo cyberfabric docs [-p <path>] [--version <version>] [--clean] [<query>]
cargo cyberfabric lint [-p <workspace>] [--all] [--clippy] [--strict] [--dylint]
cargo cyberfabric tools --all
cargo cyberfabric run [-p <workspace>] -c <config> [--name <name>] [--watch]
cargo cyberfabric build [-p <workspace>] -c <config> [--name <name>]
cargo cyberfabric deploy [-p <workspace>] -c <config> [--manifest <Cargo.toml>] [--args <KEY=VALUE>]...
