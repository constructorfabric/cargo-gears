---
title: Getting started with the CLI
description: Create a Gears workspace, add a module, wire it into config, and run it — end to end.
sidebar:
  label: Getting started
  order: 2
---

This walkthrough creates a new Gears workspace from scratch using the `cargo gears` CLI,
adds a background-worker module, wires it into the runtime config, and runs the
generated server.

## Prerequisites

- **Rust toolchain** with `cargo` — [https://rust-lang.org/tools/install/](https://rust-lang.org/tools/install/)
- **Protobuf** (for gears that use gRPC) — [https://protobuf.dev/installation/](https://protobuf.dev/installation/)

## Install the CLI

```bash
cargo install cargo-gears
```

## 1. Create a workspace

```bash
cargo gears new /tmp/cf-demo
cd /tmp/cf-demo
```

This generates a workspace with a `Gears.toml` manifest, a `config/` directory with
runtime YAML files, and a `modules/` directory for your modules.

## 2. Add a module

```bash
cargo gears generate module --template background-worker
```

This creates a `modules/background-worker/` crate inside the workspace, adds it to
the workspace `Cargo.toml` members, and promotes dependencies into workspace-level
metadata.

Available built-in module templates:

- **`background-worker`** — background worker module
- **`api-db-handler`** — API handler with database access
- **`api-gateway`** — API gateway module (prefer the system module `cf-api-gateway` unless you need a custom one)

## 3. Generate a runtime config

```bash
cargo gears generate config --template dev --app app1 --env dev
```

This writes `config/app1-dev.yml`. Built-in config templates: `dev`, `prod`, `db`
(the last includes a `database.servers.main` section).

## 4. Wire the module into config

```bash
cargo gears config mod add background-worker -c /tmp/cf-demo/config/app1-dev.yml
```

This adds a `modules.background-worker` entry to the runtime config. The CLI
discovers module metadata from the workspace automatically.

## 5. Run the server

```bash
cargo gears run --app app1 --env dev
```

The CLI reads `Gears.toml`, generates a runnable Cargo project under `.gears/app1-dev/`,
sets `GEARS_CONFIG` to the resolved config path, and invokes `cargo run`.

You should see the background worker's sample messages in the console.

## 6. Iterate with watch mode

```bash
cargo gears run --app app1 --env dev --watch
```

Watch mode restarts the server when runtime config files or watched local modules
change, and regenerates the project when `Cargo.toml` or `Gears.toml` changes.

## 7. Build for release

```bash
cargo gears build --app app1 --env dev --release
```

The compiled binary is in `.gears/app1-dev/target/release/`. To run it manually,
set `GEARS_CONFIG` yourself:

```bash
GEARS_CONFIG=/tmp/cf-demo/config/app1-dev.yml .gears/app1-dev/target/release/app1-dev
```

## Add a database

To wire a shared database server and attach it to a module:

```bash
cargo gears config db add primary -c /tmp/cf-demo/config/app1-dev.yml \
  --engine postgres --host localhost --port 5432 --user app \
  --password '${DB_PASSWORD}' --dbname appdb

cargo gears config mod db add background-worker \
  -c /tmp/cf-demo/config/app1-dev.yml --server primary
```

## See also

- [Command reference](/cli/commands/) — every command with full arguments and examples
- [Gears.toml manifest](/cli/manifest/) — the manifest schema that drives `build` and `run`
- [LLM & CI workflow](/cli/llm-workflow/) — using `src`, `help topic`, and structured output
