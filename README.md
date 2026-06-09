# Gears CLI

Command-line interface for the whole development cycle of Gears projects.

## Quickstart

### Prerequisites

- Rust toolchain with `cargo` (https://rust-lang.org/tools/install/)
- If you want to run coverage, install `cargo-llvm-cov` with `cargo +stable install cargo-llvm-cov --locked`

### Install the CLI

Install it from source:

```bash
cargo install cargo-gears
```

After installation, verify the command surface with:

```bash
cargo gears --help
```

## Typical usage flow

### 1. Create a workspace

Start by generating a new workspace:

```bash
cargo gears new /tmp/cf-demo
cd /tmp/cf-demo
```

The generated workspace is manifest-driven, so you should expect a
`Gears.toml` file alongside the runtime config files under `config/`.

### 2. Run the generated server

You can run the workspace straight away with:

```bash
cargo gears run --app quickstart --env dev
```

`build` and `run` read generation inputs from `Gears.toml`, create the generated server project under the workspace
generated directory (by default `.gears/<name>/`), and set `GEARS_CONFIG` automatically for the generated project.

### 3. Add a module to the workspace

You can add module templates such as `background-worker`, `api-db-handler`, and `api-gateway`. Prefer passing
`--name` when you want the generated module to use your chosen name instead of the template name. For this example,
we'll use `background-worker`:

```bash
cargo gears generate module --template background-worker
cargo gears config mod add background-worker -c /tmp/cf-demo/config/quickstart-dev.yml
```

Now run it again. You should see the generated background worker outputting its sample messages:

```bash
cargo gears run --app quickstart --env dev
```

You can run the CLI from any directory by specifying the workspace with `-p`. When omitted, the current directory is
used as the workspace root.

## What the CLI can do

The current CLI surface is centered on Gears workspace setup, manifest-driven generation, runtime configuration,
source inspection, and execution.

### Workspace scaffolding

- `new` initializes a new Gears workspace from a template
- `generate module --template <template>` adds module templates such as `background-worker`, `api-db-handler`, and `api-gateway`
- `generate config --template <template>` creates runtime config files such as `dev`, `prod`, or `db`

### Configuration management

- `config mod add` and `config mod rm` manage module entries in the YAML config
- `config mod db add|edit|rm` manages module-level database settings
- `config db add|edit|rm` manages shared database server definitions

Configuration commands require the path to the runtime config file with `-c`, for example:

```bash
cargo gears config mod add background-worker -c /tmp/cf-demo/config/app1-dev.yml
```

### Manifest inspection

- `ls modules` inspects available system and workspace modules
- `manifest validate` validates `Gears.toml` and the selected app/environment inputs
- `manifest ls` lists the manifest app/environment entries and their resolved config paths

These commands are the safest way to confirm what `build`, `run`, `lint`, and `test` will use.

### Build and run generated servers

- `build` generates a runnable Cargo project under the manifest `workspace.generated-dir` directory and builds it
- `run` generates the same project and runs it; you can use `-w` for watch mode, `--otel` for OpenTelemetry, and
  `--fips` for the generated manifest's `fips` feature
- `deploy` builds a Docker image with the workspace `Dockerfile`. It requires `-c` and can optionally use
  `--manifest <Cargo.toml>` to build an existing manifest instead of generating one first
- `build` and `run` also support `--release`, `--clean`, and `--dry-run`

The generated `src/main.rs` does not embed the config path. Instead, the generated server reads it from
`GEARS_CONFIG` at runtime. The CLI sets that variable for `build` and `run`, but if you execute the generated project
or compiled binary yourself, you need to set `GEARS_CONFIG` manually.

Example manual run of the generated project:

```bash
GEARS_CONFIG=/tmp/cf-demo/config/app1-dev.yml cargo run --manifest-path /tmp/cf-demo/.gears/app1-dev/Cargo.toml
```

### Source inspection and help

- `src` resolves Rust source for crates, modules, and items from the workspace, local cache, or `crates.io`
- `help schema` prints manifest, config, or module schema guidance
- `help topic` prints operational documentation for topics such as `manifest`, `generated-server`, `fips`, and `otel`
- `help src` is an alias for `src`

### Linting

`cargo gears lint --app <APP> --env <ENV>` reads the selected manifest lint policy and orchestrates `cargo fmt`,
`cargo clippy`, and `dylint` custom rules for that workspace. It respects your custom settings from `Cargo.toml`.

Use `--all` when you want to run every available lint stage explicitly. If the CLI is built without the
`dylint-rules` feature, `lint --dylint` returns an error.

### Testing

`cargo gears test --app <APP> --env <ENV>` is the manifest-driven test entrypoint. It passes the selected environment
config to tests through `GEARS_CONFIG`, and `--coverage` uses `cargo llvm-cov` with the selected runner.

At the moment, the command surface exists, but the implementation is not yet ready for general use.

### Tool bootstrap

`cargo gears tools` can install or upgrade required Rust tooling such as `rustup`, `cargofmt`, and `clippy`.

## Command overview

For the full command surface, arguments, examples, and caveats, check [SKILL.md](SKILL.md).

## Local development

To run the CLI from source:

```bash
cargo run -p cargo-gears -- gears --help
```

## License

This project is licensed under the Apache License, Version 2.0.

- Full license text: [LICENSE](./LICENSE)
- License URL: <http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, the software is distributed on an `AS IS` basis, without
warranties or conditions of any kind.
