---
title: Command reference
description: Every cargo-gears command, its arguments, behavior, and examples.
sidebar:
  label: Command reference
  order: 3
---

This page documents every command in the `cargo gears` CLI. Commands are grouped
by function. For the manifest schema that drives `build`, `run`, `lint`, and `test`,
see [Gears.toml manifest](/cli/manifest/).

## Shared argument patterns

Several flags appear across multiple commands:

- **`-p, --path <PATH>`** — Workspace root directory. When omitted, the current working directory is used. Used by `config`, `build`, `run`, `deploy`, `lint`, `test`, `ls`, `src`, and `manifest`.
- **`-c, --config <PATH>`** — Runtime config file path. Required for `config` subcommands and `deploy`. `build` and `run` do not accept `--config`; they resolve it from `Gears.toml`.
- **`--manifest <PATH>`** — Gears manifest path, defaults to `Gears.toml`. Used by `manifest`, `build`, `run`, `lint`, and `test`.
- **`--app <APP> --env <ENV>`** — Selects a manifest app/environment. When omitted, inferred from the manifest: a single app is auto-selected; for environments, `dev` is the default if it exists.
- **`--name <NAME>`** — For `build` and `run`, overrides the generated server project and binary name (default: config filename stem or `<app>-<env>`).
- **`-v, --verbose`** — Enables more detailed output.

## generate

Generate workspace, module, and config scaffolding from built-in templates or
explicit local/Git template sources.

### generate workspace

```bash
cargo gears generate workspace <path> [options]
```

Options:

- `-t, --template <TEMPLATE>` — workspace template name (default: `default`)
- `-n, --name <NAME>` — override the generated project name (default: final path segment)
- `--local-path <PATH>` — use a local template directory
- `--git <URL>` — override the Git template URL
- `--subfolder <NAME>` — override the template subfolder
- `--branch <NAME>` — override the template branch
- `--override` — allow generated files to overwrite existing files
- `-v, --verbose` — verbose output from `cargo-generate`

Creates the target directory if it does not exist. Forces Git initialization.

```bash
cargo gears generate workspace /tmp/cf-demo
```

### generate module

```bash
cargo gears generate module --template <TEMPLATE> [options]
```

Options:

- `-t, --template <TEMPLATE>` — module template name
- `-n, --name <NAME>` — generated module folder/crate name (default: template name)
- `-p, --path <PATH>` — workspace root (default: `.`)
- `--local-path`, `--git`, `--subfolder`, `--branch` — override template source
- `-v, --verbose`

Built-in templates: `background-worker`, `api-db-handler`, `api-gateway`.

Requires a `modules/` directory in the workspace. Adds the generated module to
workspace `Cargo.toml` members, promotes dependencies to workspace-level metadata,
and rewrites module dependencies to `workspace = true`.

```bash
cargo gears generate module --template background-worker -p /tmp/cf-demo
cargo gears generate module --template background-worker --name jobs -p /tmp/cf-demo
```

### generate config

```bash
cargo gears generate config --template <dev|prod|db> [options]
```

Options:

- `--app <APP>` — application name for output filename
- `--env <ENV>` — environment name for output filename
- `--name <NAME>` — custom output filename (`.yml` appended if no extension)
- `-p, --path <PATH>` — workspace root (default: `.`)

Writes to `<workspace>/config/<filename>`. Fails if the file already exists.

```bash
cargo gears generate config --template dev --app app1 --env dev -p /tmp/cf-demo
cargo gears generate config --template db --name local-db -p /tmp/cf-demo
```

## new

Alias for `generate workspace`.

```bash
cargo gears new <path> [options]
```

Same arguments as `generate workspace`.

## config

Manages the YAML runtime config file. Two branches: `config mod` (module
configuration) and `config db` (global database server configuration).

### config mod add

```bash
cargo gears config mod add <module> -c <CONFIG> [options]
```

Options:

- `-c, --config <CONFIG>` — required config file path
- `-p, --path <PATH>` — workspace directory
- `--package <NAME>` — override metadata package name
- `--module-version <VER>` — override metadata version
- `--default-features <BOOL>` — persist Cargo `default_features`
- `-F, --feature <FEATURES>` — feature list (comma-separated, repeatable)
- `--dep <NAME>` — metadata dependency name (repeatable)

Creates or updates `modules.<module>` in the config. Discovers module metadata
from the workspace when available. Remote modules require both `--package` and
`--module-version`.

```bash
cargo gears config mod add background-worker -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo gears config mod add credstore -c config/quickstart.yml --package cf-credstore --module-version 0.4.2
cargo gears config mod add api-gateway -c config/quickstart.yml -F json,metrics -F tracing --dep authn-resolver
```

### config mod rm

```bash
cargo gears config mod rm <module> -c <CONFIG> [-p <PATH>]
```

Removes `modules.<module>` from the config. Fails if the module is not present.

```bash
cargo gears config mod rm background-worker -c /tmp/cf-demo/config/quickstart.yml
```

### config mod db

Manage module-level database config under `modules.<module>.database`.

Subcommands: `add`, `edit`, `rm`.

Shared DB flags:

- `--engine <postgres|mysql|sqlite>`
- `--dsn <DSN>`
- `--host <HOST>`, `--port <PORT>`, `--user <USER>`, `--password <PASSWORD>`, `--dbname <NAME>`
- `--params <K=V,...>`
- `--sqlite-file <FILE>`, `--sqlite-path <PATH>`
- `--pool-max-conns <N>`, `--pool-min-conns <N>`, `--pool-acquire-timeout-secs <SECS>`
- `--pool-idle-timeout-secs <SECS>`, `--pool-max-lifetime-secs <SECS>`
- `--pool-test-before-acquire <BOOL>`
- `--server <NAME>` — reference a named global DB server

`add` and `edit` require at least one DB-related field. `add` requires the module
to already exist in config (use `config mod add` first). `edit` fails if no
module DB config exists yet. Both patch only the fields you provide.

```bash
cargo gears config mod db add background-worker -c config/quickstart.yml --server primary
cargo gears config mod db add api-db-handler -c config/quickstart.yml \
  --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
cargo gears config mod db edit api-db-handler -c config/quickstart.yml --pool-acquire-timeout-secs 30
cargo gears config mod db rm api-db-handler -c config/quickstart.yml
```

### config db

Manage global database server config under `database.servers`.

Subcommands: `add`, `edit`, `rm`.

```bash
cargo gears config db add <name> -c <CONFIG> [-p <PATH>] <db-flags...>
cargo gears config db edit <name> -c <CONFIG> [-p <PATH>] <db-flags...>
cargo gears config db rm <name> -c <CONFIG> [-p <PATH>]
```

`add` is upsert (creates or patches). `edit` requires the server to already exist.
`rm` removes the top-level `database` section if it becomes empty and
`auto_provision` is unset.

```bash
cargo gears config db add primary -c config/quickstart.yml \
  --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
cargo gears config db add local-sqlite -c config/quickstart.yml --engine sqlite --sqlite-path /tmp/cf-demo/dev.db
cargo gears config db edit primary -c config/quickstart.yml --pool-max-conns 30
cargo gears config db rm primary -c config/quickstart.yml
```

## build

Generate a server project and build it.

```bash
cargo gears build [options]
```

Options:

- `--manifest <PATH>` — manifest file (default: `Gears.toml`)
- `--app <APP>`, `--env <ENV>` — manifest app/environment selection
- `-p, --path <PATH>` — workspace directory
- `--name <NAME>` — override generated project name
- `--otel` / `--no-otel` — override OpenTelemetry policy
- `--fips` / `--no-fips` — override FIPS policy
- `-r, --release` / `--no-release` — override build profile
- `--clean` / `--no-clean` — override clean policy
- `--dry-run` — generate project structure and print JSON without building

Boolean flag pairs are mutually exclusive. When neither is present, the manifest
policy is used. Executes `cargo build` in `<generated-dir>/<name>/`.

```bash
cargo gears build -p /tmp/cf-demo --app app1 --env dev
cargo gears build --app app1 --env dev --release
cargo gears build --app app1 --env prod --dry-run
```

## run

Generate a server project and run it.

```bash
cargo gears run [options]
```

Options (same as `build`, plus):

- `-w, --watch` / `--no-watch` — override watch policy
- `--dry-run` — generate and print JSON without running

Sets `GEARS_CONFIG` automatically before invoking `cargo run`. Watch mode
restarts on config changes and regenerates when `Cargo.toml` or `Gears.toml`
changes.

```bash
cargo gears run -p /tmp/cf-demo --app app1 --env dev
cargo gears run --app app1 --env dev --watch
cargo gears run --app app1 --env dev --otel --fips --release --clean
```

:::note[Manual execution]
If you run the generated project or compiled binary yourself, set `GEARS_CONFIG`
manually:

```bash
GEARS_CONFIG=/tmp/cf-demo/config/app1-dev.yml \
  cargo run --manifest-path /tmp/cf-demo/.gears/app1-dev/Cargo.toml
```
:::

## deploy

Build a Docker image from the workspace `Dockerfile`.

```bash
cargo gears deploy -c <CONFIG> [options]
```

Options:

- `-c, --config <CONFIG>` — required config file path (copied into the image as `GEARS_CONFIG`)
- `-p, --path <PATH>` — workspace directory
- `-m, --manifest <Cargo.toml>` — build an existing Cargo manifest instead of generating
- `--debug` — Docker build in debug mode (default: release)
- `--dockerfile <Dockerfile>` — override Dockerfile path
- `--args <KEY=VALUE>` — Dockerfile `ARG` override (repeatable)

Without `--manifest`, generates the server project first (matching `build`/`run`).
The CLI provides `BUILDER_MANIFEST`, `BUILD_MODE`, `ARTIFACT_NAME`,
`LOCAL_CONFIG_PATH`, and `CONFIG_EXT` as Docker build args.

```bash
cargo gears deploy -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml
cargo gears deploy -c config/quickstart.yml --debug
cargo gears deploy -c config/quickstart.yml --manifest /tmp/cf-demo/Cargo.toml
```

## lint

Run workspace linting from the selected manifest app/environment.

```bash
cargo gears lint [options]
```

Options:

- `--manifest <PATH>` — manifest file (default: `Gears.toml`)
- `--app <APP>`, `--env <ENV>` — manifest selection
- `-p, --path <PATH>` — workspace directory
- `--all` — run all available lint suites
- `--fmt` — run `cargo fmt --check --all`
- `--clippy` — run workspace Clippy
- `--strict` — turn Clippy warnings into errors (requires `--clippy` or `--all`)
- `--dylint` — run embedded Dylint rules (requires the `dylint-rules` feature)

With no explicit lint-selection flags, runs the manifest lint policy. Manifest
`lint.dylint.skip` entries are passed to Dylint as allowed rustc lints.

```bash
cargo gears lint --app app1 --env dev
cargo gears lint --app app1 --env dev --clippy --strict
cargo gears lint --app app1 --env dev --dylint
```

Manifest Dylint skip example:

```toml
[apps.app1.dev.lint]
dylint = { enabled = true, skip = ["de0301_no_infra_in_domain"] }
```

## test

:::caution[Not yet ready]
The `test` command is part of the CLI surface but currently panics at runtime.
It is documented here for completeness; use `cargo test` or `cargo nextest` directly
until the implementation is complete.
:::

```bash
cargo gears test [options]
```

Options:

- `--manifest <PATH>` — manifest file (default: `Gears.toml`)
- `--app <APP>`, `--env <ENV>` — manifest selection
- `-p, --path <PATH>` — workspace directory
- `--runner <cargo|nextest>` — override manifest test runner (default: `nextest`)
- `--module <NAME>` — limit tests to a module/package
- `--coverage` — run with `cargo llvm-cov` using the selected runner

Manifest `test.feature-set` entries expand by `mode`: `all-features`,
`no-default-features`, `default-features`, or `features` (with an explicit list).

## ls modules

List all modules — both system-registry and workspace-discovered — in a unified view.

```bash
cargo gears ls modules [options]
```

Options:

- `-p, --path <PATH>` — workspace directory
- `--system` — only print built-in system modules
- `--local` — only print workspace-discovered modules
- `-v, --verbose` — show full metadata (fetches registry metadata for system modules)
- `--registry <REGISTRY>` — registry for system-crate metadata (default: `crates.io`)
- `-f, --format <table|json>` — output format (default: `json`)

```bash
cargo gears ls modules
cargo gears ls modules -p /tmp/cf-demo --verbose
cargo gears ls modules --local
```

## manifest

Inspect and validate `Gears.toml`.

```bash
cargo gears manifest [options] <validate|ls>
```

Options:

- `-p, --path <PATH>` — workspace directory
- `--manifest <PATH>` — manifest file (default: `Gears.toml`)

`validate` parses the manifest and resolves every app/environment entry.
`ls` lists configured app/environment pairs with their resolved config paths and
generated build names. Both accept `--format table|json`.

```bash
cargo gears manifest validate
cargo gears manifest ls --format table
```

## src

Resolve Rust source for a crate/module/item query from local workspace metadata,
the local source cache, or crates.io.

```bash
cargo gears src [options] [<query>]
```

Options:

- `-p, --path <PATH>` — workspace or crate to inspect (default: `.`)
- `--registry <REGISTRY>` — registry fallback (default: `crates.io`)
- `-v, --verbose` — print resolution metadata before the source
- `-l, --libs` — print `library_name -> package_name` mappings for a package query
- `--version <VERSION>` — pin a specific crate version for registry fallback
- `--clean` — remove the source cache before resolving

A query is required unless `--clean` is passed alone. Only `crates.io` is supported
as a registry.

```bash
cargo gears src cf-gears-toolkit
cargo gears src --verbose tokio::sync
cargo gears src --libs cf-gears-toolkit
cargo gears src --version 1.0.217 serde::de::Deserialize
cargo gears src --clean
```

## help

Schema, topic, and source-code help.

### help schema

```bash
cargo gears help schema <manifest|config|module> [--section <SECTION>]
```

Manifest sections: `workspace`, `apps`, `templates`.
Config sections: `server`, `database`, `logging`, `opentelemetry`, `modules`.

```bash
cargo gears help schema manifest
cargo gears help schema config --section database
```

### help topic

Print operational documentation for a named topic.

```bash
cargo gears help topic <TOPIC>
```

Available topics: `architecture`, `cli`, `clienthub`, `database`, `errors`, `fips`,
`gear-layout`, `gear-refs`, `gears-catalog`, `generated-server`, `lifecycle`,
`manifest`, `otel`, `rest-api`, `security`.

```bash
cargo gears help topic architecture
cargo gears help topic generated-server
```

### help src

Alias for the top-level `src` command.

## tools

Install or upgrade Rust tooling dependencies.

```bash
cargo gears tools (--all | --install <tool,...>) [--upgrade] [--yolo] [--verbose]
```

Known tools: `rustup`, `cargofmt` (installs the `rustfmt` component), `clippy`.

```bash
cargo gears tools --all
cargo gears tools --install clippy,cargofmt --yolo
cargo gears tools --install rustup,clippy --upgrade --verbose
```
