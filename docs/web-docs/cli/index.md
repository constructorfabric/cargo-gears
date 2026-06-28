---
title: Gears CLI
description: The cargo-gears command-line interface for scaffolding, building, running, and deploying Gears projects.
sidebar:
  label: Overview
  order: 1
---

The `cargo-gears` CLI is the command-line interface for the full development cycle of
Gears projects. It scaffolds workspaces, generates runnable servers from a manifest,
manages runtime configuration, builds, deploys, lints, and inspects source code.

## Install

Install from source:

```bash
cargo install cargo-gears
```

Verify the installation:

```bash
cargo gears --help
```

## Prerequisites

- **Rust toolchain** with `cargo` — [https://rust-lang.org/tools/install/](https://rust-lang.org/tools/install/)
- **Protobuf** (for gears that use gRPC) — [https://protobuf.dev/installation/](https://protobuf.dev/installation/)

## Manifest-driven model

The CLI is **manifest-driven**: a `Gears.toml` file at the workspace root declares
which apps, environments, modules, and policies (build, lint, run, test) the CLI
should use. Runtime values live in separate YAML config files.

| Question | Source |
| --- | --- |
| Which app, environment, modules, feature sets, lint policy, build profile? | `Gears.toml` (manifest) |
| What values does the server read at runtime? | Runtime YAML config |

This separation means `build` and `run` read generation inputs from `Gears.toml`,
compose the generated server project, and forward the runtime config path through
the `GEARS_CONFIG` environment variable automatically.

## What the CLI manages

- **Workspace scaffolding** — `new` / `generate workspace` initializes a Gears workspace; `generate module` adds module templates
- **Configuration management** — `config mod` and `config db` manage module and database entries in YAML config
- **Server generation** — `build` and `run` generate a runnable Cargo project under the manifest `generated-dir` (default `.gears/`)
- **Manifest orchestration** — `manifest validate` and `manifest ls` inspect `Gears.toml` entries
- **Deployment** — `deploy` builds a Docker image from the workspace `Dockerfile`
- **Linting** — `lint` orchestrates `cargo fmt`, `cargo clippy`, and Dylint custom rules
- **Testing** — `test` runs manifest-driven tests with `cargo test` or the embedded Nextest runner
- **Source inspection** — `src` resolves Rust source for crates and items from the workspace, local cache, or crates.io
- **Tool bootstrap** — `tools` installs or upgrades `rustup`, `rustfmt`, and `clippy`

## Command tree

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

## Where to go next

- [Getting started](/cli/getting-started/) — create a workspace, add a module, and run it end-to-end
- [Command reference](/cli/commands/) — every command, its arguments, and examples
- [Gears.toml manifest](/cli/manifest/) — the full manifest schema
