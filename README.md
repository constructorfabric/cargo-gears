# CyberFabric CLI

Command-line interface for the whole development cycle of CyberFabric projects.

## Quickstart

### Prerequisites

- Rust toolchain with `cargo` (https://rust-lang.org/tools/install/)

### Install the CLI

Install it from the repository root:

```bash
cargo install --git https://github.com/cyberfabric/cf-cli
```

After installation, you can check for:

```bash
cargo cyberfabric --help
```

## Typical usage flow

First you can create a new workspace with a basic hello-world module with:

```bash
cargo cyberfabric {new|generate workspace} /tmp/cf-demo
```

You can run it straight away and you will see in the console a hello world message:

```bash
cd /tmp/cf-demo
cargo cyberfabric run -c ./config/quickstart.yml
```

The generated server reads its config path from the `CF_CLI_CONFIG` environment variable. `cargo cyberfabric build` and
`cargo cyberfabric run` set this automatically for the generated project.

Second, add a module to the workspace. You can choose among a set of templates: `background-worker`, `api-db-handler`,
and `api-gateway`. Prefer passing `--name` when you want the generated module to use your chosen name instead of the
template name. For this example we'll use background-worker:

```bash
# bring the module to the workspace
cargo cyberfabric generate module --template background-worker
# add the module to the config
cargo cyberfabric config mod add background-worker -c ./config/quickstart.yml
```

Now, we run it again. We'll see every couple of seconds, the background worker printing a random Pokémon:

```bash
cargo cyberfabric run -c ./config/quickstart.yml
```

You can run the tool from any directory by specifying the path to the workspace with the `-p` flag. The default will be
the current directory. `cargo cyberfabric run -p /tmp/cf-demo -c /tmp/cf-demo/config/quickstart.yml`

## What the CLI can do

The current CLI surface is centered on CyberFabric workspace setup, configuration, code generation, and execution.

### Workspace scaffolding

- `{new|generate workspace}` initializes a new CyberFabric workspace from a template
- `generate module --template <template>` adds module templates such as `background-worker`, `api-db-handler`, and `api-gateway`
- `generate config --template <template>` creates runtime config files such as `dev`, `prod`, or `db`

### Configuration management

- `config mod list` inspects available and configured modules
- `config mod add` and `config mod rm` manage module entries in the YAML config
- `config mod db add|edit|rm` manages module-level database settings
- `config db add|edit|rm` manages shared database server definitions

You need to provide the path to the configuration file with the `-c` flag. `-c config/quickstart.yml`

### Build and run generated servers

- `build` generates a runnable Cargo project under `.cyberfabric/<CONFIG_NAME>` and builds it based on the `-c`
  configuration
  provided.
- `run` generates the same project and runs it. You can provide `-w` to enable watch mode, `--otel` to enable
  OpenTelemetry, and `--fips` to enable the generated manifest's `fips` feature.
- `deploy` builds a Docker image with the workspace `Dockerfile`. By default it generates the same server project from
  `-c`; pass `--manifest <Cargo.toml>` to build an existing manifest instead.
- `build` and `run` both pass `--otel` and `--fips` through as Cargo features on the generated project manifest.

The generated `src/main.rs` does not embed the config path. Instead, the generated server reads it from
`CF_CLI_CONFIG` at runtime. The CLI sets that variable for `build` and `run`, but if you execute `.cyberfabric/<name>/`
or the compiled binary yourself, you need to set `CF_CLI_CONFIG` manually.

Example manual run of the generated project:

```bash
CF_CLI_CONFIG=/tmp/cf-demo/config/quickstart.yml cargo run --manifest-path /tmp/cf-demo/.cyberfabric/quickstart/Cargo.toml
```

### Source inspection

- `docs` resolves Rust source for crates, modules, and items from the workspace, local cache, or `crates.io`

### Linting

`cargo cyberfabric lint --app <APP> --env <ENV>` reads the selected manifest lint policy and orchestrates `cargo fmt`,
`cargo clippy`, and `dylint` custom rules for that workspace. It respects your custom settings from `Cargo.toml`.

If the CLI is built without the `dylint-rules` feature, `lint --dylint` returns an error.

### Tool bootstrap

- `tools` installs or upgrades `rustup`, `rustfmt`, and `clippy`

### Current placeholders

- `test` is declared but not implemented yet

## Command overview

For the full command surface, arguments, and examples, check [SKILL.md](SKILL.md).

## Local development

To run the CLI from source:

```bash
cargo run -p cli -- --help
```

## License

This project is licensed under the Apache License, Version 2.0.

- Full license text: [LICENSE](./LICENSE)
- License URL: <http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, the software is distributed on an `AS IS` basis, without
warranties or conditions of any kind.
