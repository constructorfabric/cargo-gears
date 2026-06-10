---
name: gears
description: cli reference to help with the development of constructor fabric gears framework. It helps with the development of
  the framework from its initialization, adding/removing modules, modifying configuration files,
  build and/or run project, lint the project and managing applications through its manifest.
  
  Always load this skill whenever you detect Gears.toml or you locate a reference to gears, cargo-gears, gears-toolkit, 
  modules, plugins or packages that include the prefix cf- in its name.
---

# General Guidelines to follow

- When adding new dependencies use `cargo add`, do not edit Cargo.toml manually
- When linting, use lint command of the cli to check if there are any lint errors.
  Do not try to run cargo check, clippy or fmt by your own.
- Always verify that the application runs successfully after modifying the code.
- Unless the user specifically mentions to use a custom module, prefer to use system modules
  instead of implementing your own ones. Pay attention to the "deps" section of the modules, as the module will be
  required to be added in order to be used.
- Do not try to create a module from scratch, always use the generate module command to create a new module.

## Invocation Forms

The crate exposes a single entrypoint:

- **[`cargo gears`]** Cargo subcommand form via the `cargo-gears` binary

Example:

```bash
cargo gears generate workspace /tmp/my-app
```

## Objective

This CLI is a tool for automating gears development, a Rust framework. You can get more information about it in:

- Gears repository main: https://github.com/constructorfabric/cyberware-rust
- Gears libraries(the ones that leverage this CLI tool) are located
  in https://github.com/constructorfabric/cyberware-rust/tree/main/libs
- More documentation of the project will be located in https://github.com/constructorfabric/cyberware-rust/tree/main/docs

Clone(shallow) the repo to .gears folder (create it if it doesn't exist), and use it as a reference.
If so, prefer to use the ssh version instead of https to avoid authentication issues.

## Command Tree

```text
cargo gears
├── generate
│   ├── workspace
│   ├── module
│   └── config
├── new
├── config
│   ├── mod
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
├── ls
│   └── modules
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

- **[`-p, --path <PATH>`]** Optional workspace path. When provided to `config ...`, `build`, `run`, `deploy`, `lint`,
  and `test`, relative config paths, manifest paths, generated project locations, and workspace-scoped lint/test resolution use
  that directory as the workspace root. When omitted, the current working directory is used as the workspace root.
- **[`-c, --config <PATH>`]** Config file path. This is required for `config ...` and `deploy` commands because there is
  no default. `build` and `run` no longer accept `--config`; they compose their generation inputs from `Gears.toml`
  and forward the manifest-declared runtime config path through the `GEARS_CONFIG` environment variable.
- **[`--manifest <PATH>`]** Gears manifest path, defaulting to `Gears.toml`, for `manifest`, `build`, `run`,
  `lint`, and `test`.
  For `manifest`, you can combine this with `-p/--path` to resolve relative manifest paths from a selected workspace.
- **[`--app <APP> --env <ENV>`]** Selects a manifest app/environment for manifest-driven `build`, `run`, `lint`, and `test`.
  When omitted, inferred from the manifest: a single app is used automatically; with multiple apps the command
  fails listing available names. For environments, `dev` is selected by default if it exists; otherwise the
  command fails listing available names.
- **[`--name <NAME>`]** For `build` and `run`, overrides the generated server project and binary name that would
  otherwise default to the config filename stem.
- **[`-v, --verbose`]** Usually enables more logging or richer output.
- **[name validation]** Config-managed names for modules, DB servers, and generated server names only allow letters,
  numbers, `-`, and `_`.

## What the Tool Manages

From the current implementation, the CLI is mainly for:

- **[workspace scaffolding]** Initialize a Gears workspace and add module templates
- **[config management]** Enable modules and patch YAML config sections
- **[server generation]** Generate a runnable Cargo project under the manifest `workspace.generated-dir` directory
  (default `.gears/<name>/`)
- **[manifest orchestration]** Read `Gears.toml` to separate generation metadata from runtime YAML config
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
cargo gears generate workspace <path> [--template <TEMPLATE>] [--verbose] [--name <NAME>] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>] [--override]
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
cargo gears generate workspace /tmp/cf-demo
```

```bash
cargo gears generate workspace /tmp/cf-demo --git https://github.com/constructorfabric/cf-template-rust --branch main --subfolder Init
```

```bash
cargo gears generate workspace /tmp/cf-demo --local-path ~/dev/cf-template-rust
```

#### `generate module`

Generate a module template inside an existing workspace's `modules/` directory and wire Cargo workspace dependencies.

Synopsis:

```bash
cargo gears generate module --template <TEMPLATE> [--name <NAME>] [--path <PATH>] [--verbose] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>]
```

Available built-in templates:

- **[`background-worker`]** Background worker module template
- **[`api-db-handler`]** API/database handler module template
- **[`api-gateway`]** API gateway module template. Unless the user instruct to implement its own api-gateway, prefer the system module cf-api-gateway

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
- **[prevents duplicate]** Fails if that module directory already exists
- **[updates workspace members]** Adds generated modules to `workspace.members`
- **[promotes dependencies]** Moves new module dependency source/version metadata into `workspace.dependencies`
- **[rewrites module Cargo files]** Rewrites module dependencies to `workspace = true`
- **[inherits workspace lints]** Adds `lints.workspace = true` to generated modules if needed
- **[includes SDK crate when present]** If the generated module contains `sdk/`, it is also added as a workspace member

Examples:

```bash
cargo gears generate module --template background-worker -p /tmp/cf-demo
```

```bash
cargo gears generate module --template background-worker --name jobs -p /tmp/cf-demo
```

```bash
cargo gears generate module --template api-db-handler -p /tmp/cf-demo --local-path ~/dev/cf-template-rust --subfolder Modules/api-db-handler
```

#### `generate config`

Generate a runtime YAML config file from a built-in template.

Synopsis:

```bash
cargo gears generate config --template <dev|prod|db> [--app <APP>] [--env <ENV>] [--name <NAME>] [--path <PATH>]
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
cargo gears generate config --template dev --app app1 --env dev -p /tmp/cf-demo
```

```bash
cargo gears generate config --template db --name local-db -p /tmp/cf-demo
```

### `new`

Alias for `generate workspace`.

Synopsis:

```bash
cargo gears new <path> [--template <TEMPLATE>] [--verbose] [--name <NAME>] [--local-path <PATH>] [--git <URL>] [--subfolder <NAME>] [--branch <NAME>] [--override]
```

### `config`

Manages the YAML application config file used by `build` and `run`.

There are two branches:

- **[`config mod ...`]** Module configuration
- **[`config db ...`]** Global database server configuration

### `config mod`

Manage the `modules` section in the app config.

#### `config mod add`

Add or update a module entry in the config file's `modules` section.

Synopsis:

```bash
cargo gears config mod add -c <CONFIG> [-p <PATH>] [--package <NAME>] [--module-version <VER>] [--default-features <BOOL>] [-F, --feature <FEATURES>]... [--dep <NAME>]... <module>
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
cargo gears config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo gears config mod add api-gateway -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml -F json,metrics -F tracing --dep authn-resolver --dep tenant-resolver
```

```bash
cargo gears config mod add credstore -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --package cf-credstore --module-version 0.4.2
```

```bash
cargo gears config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --default-features false
```

#### `config mod rm`

Remove a module from the config file's `modules` section.

Synopsis:

```bash
cargo gears config mod rm -c <CONFIG> [-p <PATH>] <module>
```

Behavior:

- **[path activation]** If `-p/--path` is provided, Clap changes the current working directory while parsing that value,
  before `-c/--config` is resolved
- **[strict removal]** Fails if the module is not present in config

Example:

```bash
cargo gears config mod rm background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
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
cargo gears config mod db add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --server primary
```

```bash
cargo gears config mod db add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb --pool-max-conns 20
```

```bash
cargo gears config mod db edit api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --pool-acquire-timeout-secs 30 --pool-test-before-acquire true
```

```bash
cargo gears config mod db rm api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

### `config db`

Manage global database server config under `database.servers`.

Subcommands:

- **[`add`]** Add or upsert a named global DB server
- **[`edit`]** Edit an existing global DB server
- **[`rm`]** Remove an existing global DB server

Synopsis:

```bash
cargo gears config db add  -c <CONFIG> [-p <PATH>] <name> <db-flags...>
cargo gears config db edit -c <CONFIG> [-p <PATH>] <name> <db-flags...>
cargo gears config db rm   -c <CONFIG> [-p <PATH>] <name>
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
cargo gears config db add primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
```

```bash
cargo gears config db edit primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --pool-max-conns 30 --pool-idle-timeout-secs 120
```

```bash
cargo gears config db add local-sqlite -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine sqlite --sqlite-path /tmp/cf-demo/dev.db
```

```bash
cargo gears config db rm primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

### `src`

Resolve Rust source for a crate/module/item query from local workspace metadata, the local source cache, or crates.io.

Synopsis:

```bash
cargo gears src [--path <PATH>] [--registry <REGISTRY>] [--verbose] [--libs] [--version <VERSION>] [--clean] [<query>]
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

- **[`cf-gears-toolkit`]**
- **[`tokio::sync`]**
- **[`cf-gears-toolkit::gts::plugin::BaseGearsPluginV1`]**
- **[`cf-gears-toolkit::gts::schemas::get_core_gts_schemas`]**

Behavior:

- **[query requirement]** A query is required unless `--clean` is passed by itself
- **[package-only libs mode]** `--libs` requires a package-only query such as `cf-gears-toolkit`
- **[local resolution first]** Tries workspace metadata before hitting the network
- **[cache-first registry fallback]** Reuses cached crate sources before downloading from the registry
- **[crates.io fallback]** Downloads and extracts crate source if local resolution and cache lookup both fail
- **[exact version fallback]** `--version` pins the registry/cache fallback to that exact crate version
- **[recursive re-export resolution]** Follows re-exports across `crate`, `self`, `super`, and dependency boundaries
  until it reaches the final source
- **[library mapping output]** `--libs` prints the Rust source-code library name on the left and the Cargo package
  name on the right, including renamed dependencies like `gears_toolkit_macros -> cf-gears-toolkit-macros`
- **[cache location]** Registry sources are cached under the OS temp directory in `gears-docs-cache/<registry>/` (legacy name)
- **[cache cleaning]** `--clean` removes the selected registry cache before resolution
- **[source output]** Prints the resolved Rust source to stdout
- **[verbose metadata]** Also prints query, package, library, version, manifest path, and source path
- **[registry support]** Only `crates.io` is supported today

Examples:

```bash
cargo gears src -p /tmp/cf-demo cf-gears-toolkit
```

```bash
cargo gears src cf-gears-toolkit::module
```

```bash
cargo gears src --verbose tokio::sync
```

```bash
cargo gears src --libs cf-gears-toolkit
```

```bash
cargo gears src --version 1.0.217 serde::de::Deserialize
```

```bash
cargo gears src --clean
```

```bash
cargo gears src --clean -p /tmp/cf-demo tokio::sync
```

### `help`

Schema, topic, and source-code help for developers and LLMs. Groups three subcommands under a
single discoverable entry point.

#### `help schema`

Print the schema for manifest, config, or module formats.

Synopsis:

```bash
cargo gears help schema <manifest|config|module> [--section <SECTION>]
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
cargo gears help schema manifest
```

```bash
cargo gears help schema config --section database
```

```bash
cargo gears help schema module
```

#### `help src`

Alias for the top-level `src` command. Resolves Rust source code from a crate.

Synopsis:

```bash
cargo gears help src [--path <PATH>] [--registry <REGISTRY>] [--verbose] [--libs] [--version <VERSION>] [--clean] [<query>]
```

Behavior identical to `src`; see the `src` section above.

#### `help topic`

Print operational documentation for a named topic.

Synopsis:

```bash
cargo gears help topic <TOPIC>
```

Available topics:

- **[`architecture`]** Framework architecture, three-tier hierarchy, and principles
- **[`cli`]** CLI reference, guidelines, and command overview
- **[`clienthub`]** Typed ClientHub, plugins, and GTS
- **[`database`]** SecureConn, transactions, migrations, and repository pattern
- **[`errors`]** RFC-9457 Problem error handling
- **[`fips`]** FIPS mode activation and usage
- **[`gear-layout`]** Gear directory structure and SDK pattern
- **[`gear-refs`]** How local and remote gears are referenced
- **[`gears-catalog`]** Gear categories and dependency rules
- **[`generated-server`]** How the ephemeral generated server project works
- **[`lifecycle`]** Gear lifecycle, cancellation, and background tasks
- **[`manifest`]** Overview of `Gears.toml` and manifest-driven workflows
- **[`otel`]** OpenTelemetry activation and runtime configuration
- **[`rest-api`]** OperationBuilder, OpenAPI, SSE, and OData
- **[`security`]** AuthN, AuthZ, SecureConn, and AccessScope

Examples:

```bash
cargo gears help topic architecture
```

```bash
cargo gears help topic gear-layout
```

```bash
cargo gears help topic generated-server
```

```bash
cargo gears help topic otel
```

### `tools`

Install or upgrade a small set of Rust tooling dependencies.

Known tool names:

- **[`rustup`]**
- **[`cargofmt`]** Installs the `rustfmt` rustup component
- **[`clippy`]**

Synopsis:

```bash
cargo gears tools (--all | --install <tool,...>) [--upgrade] [--yolo] [--verbose]
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
cargo gears tools --all
```

```bash
cargo gears tools --install clippy,cargofmt --yolo
```

```bash
cargo gears tools --install rustup,clippy --upgrade --verbose
```

### `run`

Generate a server project under the manifest `<workspace.generated-dir>/<name>` and run it.

Synopsis:

```bash
cargo gears run [--app <APP>] [--env <ENV>] [--manifest <Gears.toml>] [-p <PATH>] [--name <NAME>] [--watch|--no-watch] [--otel|--no-otel] [--fips|--no-fips] [--release|--no-release] [--clean|--no-clean] [--dry-run]
```

Arguments:

- **[`--manifest <PATH>`]** Manifest file, defaults to `Gears.toml`
- **[`--app <APP> --env <ENV>`]** Manifest app/environment selection (inferred from manifest if omitted)
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
  before `Gears.toml` is resolved and the generated project directory is created
- **[generates server structure]** Writes `<generated-dir>/<name>/Cargo.toml`,
  `<generated-dir>/<name>/.cargo/config.toml`, and `<generated-dir>/<name>/src/main.rs`; `generated-dir` comes from
  manifest `workspace.generated-dir` and defaults to `.gears`
- **[runtime config handoff]** The generated `src/main.rs` reads the config path from `GEARS_CONFIG`, and
  `cargo gears run` sets that environment variable automatically before invoking `cargo run`
- **[dry run]** `--dry-run` writes the generated project structure under `<generated-dir>/<name>/` and prints JSON with the
  generated project directory plus each generated file path and contents; it does not invoke Cargo build/run
- **[manifest mode]** Reads generation dependencies, runtime config path, and policies from `Gears.toml`; runtime YAML
  stays focused on server configuration
- **[exclusive boolean overrides]** Boolean flag pairs are mutually exclusive. Use either the positive or negative form,
  not both; for example, `--otel --no-otel` is rejected. When neither side is present, the manifest policy is used.
- **[runs inside generated project]** Executes `cargo run` in `<generated-dir>/<name>/`
- **[watch mode]** Restarts on runtime config changes and changes in watched local modules; regenerates and reruns when
  workspace `Cargo.toml` or the selected `Gears.toml` changes
- **[watch policy]** Manifest `run.watch.include` replaces the default watch set; `run.watch.exclude` removes matching
  paths from either the default set or an explicit include set
- **[manual generated-project execution]** If you invoke the generated project or compiled binary yourself instead of
  using `cargo gears run`, you must set `GEARS_CONFIG` manually

Examples:

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev
```

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev --watch
```

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev --otel --fips --release --clean
```

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev --name demo-server
```

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev --dry-run
```

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev --manifest /tmp/cf-demo/Gears.toml
```

### `manifest`

Inspect and validate Gears manifest files.

Synopsis:

```bash
cargo gears manifest [-p <PATH>] [--manifest <Gears.toml>] validate [--format table|json]
cargo gears manifest [-p <PATH>] [--manifest <Gears.toml>] ls [--format table|json]
```

Behavior:

- **[path activation]** If `-p/--path` is provided, relative manifest paths are resolved from that workspace directory.
- **[validate]** Parses the manifest and resolves every app/environment entry.
- **[ls]** Lists configured app/environment pairs with their resolved config paths and generated build names.
- **[generated structure]** Use `build --dry-run` or `run --dry-run` to write and print the generated project structure.

### `build`

Generate a server project under the manifest `<workspace.generated-dir>/<name>` and build it.

Synopsis:

```bash
cargo gears build [--app <APP>] [--env <ENV>] [--manifest <Gears.toml>] [-p <PATH>] [--name <NAME>] [--otel|--no-otel] [--fips|--no-fips] [--release|--no-release] [--clean|--no-clean] [--dry-run]
```

Arguments:

- **[`--manifest <PATH>`]** Manifest file, defaults to `Gears.toml`
- **[`--app <APP> --env <ENV>`]** Manifest app/environment selection (inferred from manifest if omitted)
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
  before `Gears.toml` is resolved and the generated project directory is created
- **[exclusive boolean overrides]** Boolean flag pairs are mutually exclusive. Use either the positive or negative form,
  not both; for example, `--clean --no-clean` is rejected. When neither side is present, the manifest policy is used.
- **[manifest mode]** Reads module dependency metadata, runtime config path, and build/run policy from `Gears.toml`
  instead of runtime YAML module metadata
- **[builds inside generated project]** Executes `cargo build` in `<generated-dir>/<name>/`
- **[runtime config source]** The generated server no longer embeds the config path; the resulting binary reads it from
  `GEARS_CONFIG` when you execute it
- **[dry run]** `--dry-run` writes the generated project structure under `<generated-dir>/<name>/` and prints JSON with the
  generated project directory plus each generated file path and contents; it does not invoke Cargo build
- **[manual generated-project execution]** If you later run the generated project or binary outside the CLI, you must
  set `GEARS_CONFIG` yourself

Examples:

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env dev
```

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env dev --release
```

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env dev --otel --fips --clean
```

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env dev --name demo-server
```

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env prod
```

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env prod --dry-run
```

### `deploy`

Generate a server project under the default generated directory and build a Docker image with the workspace `Dockerfile`.

Synopsis:

```bash
cargo gears deploy -c <CONFIG> [-p <PATH>] [--manifest <Cargo.toml>] [--debug] [--dockerfile] [--args <KEY=VALUE>]...
```

Arguments:

- **[`-c, --config <CONFIG>`]** Required config file path; copied into the image and used as the runtime
  `GEARS_CONFIG` target
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
cargo gears deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
```

```bash
cargo gears deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --debug
```

```bash
cargo gears deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --manifest /tmp/cf-demo/Cargo.toml
```

```bash
cargo gears deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --args BUILDER_FLAGS="--features metrics"
```

### `lint`

Run workspace linting helpers from the selected workspace directory.

Synopsis:

```bash
cargo gears lint [--app <APP>] [--env <ENV>] [--manifest <Gears.toml>] [-p <PATH>] [--all] [--fmt] [--clippy] [--strict] [--dylint]
```

Arguments:

- **[`-p, --path <PATH>`]** Optional workspace directory used to resolve relative manifest paths
- **[`--manifest <PATH>`]** Manifest file, defaults to `Gears.toml`
- **[`--app <APP> --env <ENV>`]** Manifest app/environment selection (inferred from manifest if omitted)
- **[`--all`]** Runs all available lint suites instead of the selected manifest lint policy
- **[`--fmt`]** Runs `cargo fmt --check --all`; if passed by itself, it runs only formatting checks
- **[`--clippy`]** Runs workspace Clippy checks; if passed by itself, it runs only Clippy
- **[`--strict`]** Turns Clippy warnings into errors; valid only when Clippy is selected explicitly or through `--all`
- **[`--dylint`]** Runs the embedded `cargo-gears-lints` Dylint rules against the workspace rooted at the current or selected
  directory

Behavior:

- **[path activation]** If `-p/--path` is provided, relative manifest paths are resolved from that workspace directory
- **[manifest mode]** Reads lint policy from the selected `Gears.toml` app/environment and runs from the resolved
  manifest workspace root
- **[default lint selection]** With no explicit lint-selection flags, `lint` runs the selected manifest lint policy
- **[explicit selection disables default all]** Passing `--fmt`, `--clippy`, and/or `--dylint` opts into just those
  requested lint suites unless `--all` is also provided
- **[workspace formatting check]** `--fmt` runs `cargo fmt --check --all`
- **[workspace Clippy]** Clippy runs as `cargo clippy --workspace --all-targets`. Manifest `feature-set-test` policy is
  reserved for feature-matrix linting and is not expanded into `--all-features`.
- **[strict scope]** `--strict` is rejected unless Clippy is active through `--clippy` or `--all`
- **[workspace-scoped dylint]** Dylint receives the resolved workspace manifest path, so `-p/--path` is the way to lint
  another workspace without manually changing directories
- **[manifest dylint skips]** Manifest `lint.dylint.skip` entries are passed to Dylint as allowed rustc lints, so
  listed rules are ignored for that lint run
- **[toolchain bootstrap]** The build script ensures the lint package toolchain and components are installed when
  compiling embedded rules. Before running Dylint, the CLI ensures the embedded dylib toolchain is installed.

Examples:

```bash
cargo gears lint --app app1 --env dev
```

```bash
cargo gears lint --app app1 --env dev --clippy --strict
```

```bash
cargo gears lint --app app1 --env dev --fmt
```

```bash
cargo gears lint --app app1 --env dev --dylint
```

```bash
cargo gears lint -p /tmp/cf-demo --app app1 --env dev --dylint
```

Manifest Dylint skip example:

```toml
[apps.app1.dev.lint]
dylint = { enabled = true, skip = ["de0301_no_infra_in_domain"] }
```

### `ls`

Inspect workspace modules, system modules, and project state.

#### `ls modules`

List all modules — both system-registry and workspace-discovered — in a single unified view.

Synopsis:

```bash
cargo gears ls modules [-p <PATH>] [--system] [--local] [--verbose] [--registry crates.io] [--format table|json]
```

Arguments:

- **[`-p, --path <PATH>`]** Optional workspace directory; changes the current working directory while Clap parses it
- **[`--system`]** Only print built-in system registry modules
- **[`--local`]** Only print workspace-discovered modules
- **[`-v, --verbose`]** Show full metadata for both system and local modules (fetches registry metadata for system
  modules)
- **[`--registry <REGISTRY>`]** Registry to query for system-crate metadata; defaults to `crates.io`
- **[`-f, --format <FORMAT>`]** Output format; defaults to `json`. Supported values: `table`, `json`

Behavior:

- **[combined output]** Prints system modules first, then workspace modules, separated by a blank line
- **[filtered output]** `--system` prints only system modules; `--local` prints only workspace modules; passing both is
  equivalent to the default combined output
- **[json output]** `--format json` prints a `modules` array with each module's `source` (`system` or `local`), static
  metadata, and verbose metadata when `--verbose` is enabled
- **[system usage marker]** System modules include `used: yes/no` in table output and `used: true/false` in JSON output,
  based on whether `Gears.toml` references the system module as a remote module or the workspace has a discovered module
  with the same module name
- **[config-independent]** Does not require a `-c/--config` file
- **[workspace scanning]** Local output runs `cargo metadata --no-deps` and discovers crates with a `src/module.rs` target
- **[system registry]** System output uses the compiled-in registry, and `--verbose` fetches crate metadata from the
  selected registry

Examples:

```bash
cargo gears ls modules
```

```bash
cargo gears ls modules -p /tmp/cf-demo --verbose
```

```bash
cargo gears ls modules --local
```

```bash
cargo gears ls modules --system --verbose
```

### `test`

Orchestrates manifest-driven Rust tests with either `cargo test` or the in-process nextest runner.

Synopsis:

```bash
cargo gears test [-p <PATH>] [--manifest <PATH>] [--app <APP>] [--env <ENV>] [--runner <cargo|nextest>] [--module <NAME>] [--coverage]
```

Arguments:

- **[`--runner <cargo|nextest>`]** Overrides the manifest `test.runner`; defaults to `nextest` that it's integrated in the tool
- **[`--module <NAME>`]** Limits tests to a module/package. When manifest `test.feature-set` contains that module, its feature matrix is used.
- **[`--coverage`]** Runs coverage with `cargo llvm-cov` using the selected or manifest test runner

Behavior:

- **[manifest selection]** Resolves the selected app/environment and passes its config path to tests via `GEARS_CONFIG`
- **[cargo runner]** Runs `cargo test` with `--workspace` by default, or `-p <package>` for module-specific runs
- **[nextest runner]** Builds test binaries with Cargo JSON messages, then executes them through the embedded nextest runner from `cargo-nextest`
- **[coverage runner]** Cleans llvm-cov artifacts once, executes every selected module/feature-set run, then generates one aggregate report across all collected profiles
- **[cargo coverage]** With `cargo`, runs each selected module/feature-set through `cargo llvm-cov --no-report --no-clean`, then reports once with `cargo llvm-cov report`
- **[nextest coverage]** With `nextest`, uses `cargo llvm-cov show-env`, runs the embedded nextest runner under the coverage environment for every selected module/feature-set, then reports once with `cargo llvm-cov report`
- **[feature-set policy]** Expands manifest `test.feature-set` module entries by `mode`: `all-features` uses `--all-features`, `no-default-features` uses `--no-default-features`, `default-features` uses Cargo defaults, and `features` uses `--no-default-features --features <LIST>`

## Practical End-to-End Flows

### Create a workspace and run it

```bash
cargo gears {new|generate workspace} /tmp/cf-demo
cargo gears generate module --template background-worker -p /tmp/cf-demo
cargo gears generate config --template dev --app app1 --env dev -p /tmp/cf-demo
cargo gears config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/app1-dev.yml
cargo gears run -p /tmp/cf-demo --app app1 --env dev
```

### Add a module and wire a shared DB server

```bash
cargo gears generate module --template api-db-handler -p /tmp/cf-demo
cargo gears config db add primary -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
cargo gears config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo gears config mod db add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml --server primary
cargo gears run -p /tmp/cf-demo --app app1 --env dev --watch
```

### Inspect source for a dependency

```bash
cargo gears src --verbose tokio::sync
```

## Important Caveats

- **[`-c/--config` is mandatory]** For `config ...` and `deploy`; `build` and `run` use `Gears.toml` instead
- **[generated servers expect `GEARS_CONFIG`]** `cargo gears run` sets it for you, but manual execution of
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
cargo gears {new|generate workspace} <path> [--template <template>] [--name <name>]
cargo gears generate module --template <background-worker|api-db-handler|api-gateway> [--name <name>] [-p <workspace>]
cargo gears generate config --template <dev|prod|db> [--app <app>] [--env <env>] [--name <name>] [-p <workspace>]

cargo gears config mod add <module> [-p <workspace>] -c <config>
cargo gears config mod rm <module> [-p <workspace>] -c <config>
cargo gears config mod db add <module> [-p <workspace>] -c <config> ...
cargo gears config mod db edit <module> [-p <workspace>] -c <config> ...
cargo gears config mod db rm <module> [-p <workspace>] -c <config>

cargo gears config db add <name> [-p <workspace>] -c <config> ...
cargo gears config db edit <name> [-p <workspace>] -c <config> ...
cargo gears config db rm <name> [-p <workspace>] -c <config>

cargo gears ls modules [-p <workspace>] [--system] [--local] [--verbose] [--registry crates.io] [-f table|json]

cargo gears src [-p <path>] [--version <version>] [--clean] [<query>]
cargo gears lint [-p <workspace>] [--app <app>] [--env <env>] [--manifest <Gears.toml>] [--all] [--clippy] [--strict] [--dylint]
cargo gears tools --all
cargo gears run [-p <workspace>] [--app <app>] [--env <env>] [--manifest <Gears.toml>] [--name <name>] [--watch]
cargo gears build [-p <workspace>] [--app <app>] [--env <env>] [--manifest <Gears.toml>] [--name <name>]
cargo gears deploy [-p <workspace>] -c <config> [--manifest <Cargo.toml>] [--args <KEY=VALUE>]...
```
