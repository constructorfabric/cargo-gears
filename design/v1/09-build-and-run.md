# 09. Build and Run

## Table of Contents

1. [Purpose](#purpose)
2. [Run Command](#run-command)
3. [Build Command](#build-command)
4. [Build Outputs](#build-outputs)
5. [Feature Flags](#feature-flags)
6. [Watch Mode](#watch-mode)
7. [Manifest Integration](#manifest-integration)
8. [Dry Run](#dry-run)
9. [Generated Server Project](#generated-server-project)

## Purpose

Build and run are the action commands that take the resolved generation model and produce executable artifacts.
Both share the same generation pipeline (manifest resolution -> module resolution -> code generation).

## Run Command

### Synopsis

```bash
cargo gears run [--app <app>] [--env <env>] [-c <config>] [-w <workspace>] [--name <name>] [--watch] [--otel] [--no-otel] [--fips] [--no-fips] [--release] [--clean] [--dry-run]
```

### Behavior

1. Resolve manifest app with `cargo gears run --app app1 --env dev`.
2. Generate `.gears/<app>-<env>/` with a cargo manifest, a main function and a .cargo config.
3. Execute `cargo run` inside the generated project while providing as first argument the resolved config path.

### Name Resolution

| Input                  | Generated Project Path    |
|------------------------|---------------------------|
| `--app app1 --env dev` | `.gears/app1-dev/`    |
| `--name demo-server`   | `.gears/demo-server/` |

`--name` always takes highest precedence.

## Build Command

### Synopsis

```bash
cargo gears build [--app <app>] [--env <env>] [-c <config>] [-w <workspace>] [--name <name>] [--output <output>] [--otel] [--no-otel] [--fips] [--no-fips] [--release] [--clean] [--dry-run]
```

### Behavior

Same pipeline as `run`, but invokes `cargo build` instead of `cargo run` and supports multiple output types.

## Build Outputs

```bash
cargo gears build --app app1 --env prod
```

### Binary Build

```toml
[apps.app1.prod.build]
profile = "release"
name = "app1"
```

The generated project path is deterministic: `.gears/<app>-<env>/`. The binary is placed in the generated
project's `target/<profile>/` directory.

## Feature Flags

FIPS and OpenTelemetry are Cargo features on the generated project, controlled by the manifest or CLI flags.

### FIPS

```toml
[apps.app1.prod.run]
fips = true
```

Or via CLI: `--fips`.

The CLI passes `-F fips` to the generated project's Cargo invocation.

Validation:

- Warn if `fips = true` in a non-production environment.
- Fail if selected modules declare a known incompatible capability.

### OpenTelemetry

```toml
[apps.app1.prod.run]
otel = true
```

Or via CLI: `--otel`.

The CLI passes `-F otel` to the generated project's Cargo invocation.

Runtime values (endpoints, sampling) remain in the runtime config:

```yaml
opentelemetry:
  tracing:
    endpoint: http://localhost:4317
```

## Watch Mode

### Synopsis

```bash
cargo gears run --app app1 --env dev --watch
```

### Watched Paths

Watch mode observes:

- Workspace module source directories (path-based dependencies).
- Runtime config file.
- Manifest file (`Gears.toml`).
- Workspace `Cargo.toml`.

When a change is detected, the CLI:

1. Regenerates the generated server project if manifest or workspace metadata changed.
2. Restarts the `cargo run` process.

All notify events will have a debounced of 300ms

### Manifest Watch Policy

To override default values(detected members, configs and generated files)

```toml
[apps.app1.dev.run.watch]
enabled = true
include = ["modules", "config/app1-dev.yml", "Gears.toml"]
exclude = ["target", ".gears"]
```

### Watch Flags

| Flag                  | Description                |
|-----------------------|----------------------------|
| `--watch`             | Enable watch mode          |
| `--watch-path <path>` | Add an extra path to watch |
| `--ignore <glob>`     | Add an ignore pattern      |

For the first implementation, the current watch behavior is preserved with manifest selection added. Path-level
customization from the manifest follows.

## Manifest Integration

### Resolution Order

For all action commands, settings are resolved in this order:

1. **CLI flags** (highest precedence).
2. **App manifest policy** (`apps.<app>.<env>.run`, `.build`).
3. **CLI defaults** (lowest precedence).

### Resolved Run Model

The resolved model can be inspected with `--dry-run`:

```bash
cargo gears run --app app1 --env dev --dry-run --format json
```

```json
{
  "environment": "dev",
  "app": "app1",
  "config": "config/app1-dev.yml",
  "generated_project": ".gears/app1-dev",
  "modules": [
    "background-worker",
    "credstore"
  ],
  "features": [
    "otel"
  ],
  "watch": true,
  "profile": "debug"
}
```

## Dry Run

It prints the resolved execution plan without performing side effects:

- No files are generated.
- No Cargo commands are invoked.

## Generated Server Project

The generated project under `.gears/<name>/` contains:

```text
.gears/<name>/
├── Cargo.toml             # Generated from manifest module list
├── .cargo/
│   └── config.toml        # Cargo configuration
└── src/
    └── main.rs            # Generated entry point
```

### `Cargo.toml`

Dependencies are derived from the resolved module list. Each module becomes a dependency with its source (path for
local, version for registry), features, and default-features.

```toml
[package]
name = "<app>-<env>"
version = "0.1.0"

[features]
default = []
otel = ["cf-modkit/otel"]
fips = ["cf-modkit/fips"]

[dependencies]
anyhow = "..." # Same version as the workspace, otherwise 1 by default
tokio = "..." # Same version as the workspace, otherwise 1 by default
cf-modkit = "..." # Same version as the workspace plus the bootstrap feature
# module dependencies generated based on the manifest
# {{dependencies}}

[workspace] # We stop cargo workspace resolution here
```

### `main.rs`

The generated main.rs is a simple entry point that:

1. Reads the first argument as the config path.
2. Loads the runtime config.
3. Runs the server.

The CLI generates based on the manifest the module list dependencies that the application will have,
so that manual modification is abstracted.

```rust
use anyhow::{Context, Result};
// ALL module dependencies are added here
// {{dependencies}}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = std::env::args().nth(1)
        .map(std::path::PathBuf::from)
        .context("first argument must be the configuration path")?;
    let config = modkit::bootstrap::AppConfig::load_or_default(Some(&config_path))?;

    modkit::bootstrap::run_server(config).await
}
```
### .cargo/config.toml

It will reuse the artifacts from the workspace. This is to avoid recompiling the modules.

```toml
[build]
target-dir = "../../target"
build-dir = "../../target"
```

### `.gears/` Is Derived Output

The `.gears/` directory is **derived output** that can be regenerated at any time. It is:

- Listed in `.gitignore`.
- Never manually edited.
- Regenerated by every `run` or `build` invocation.
- Safe to delete (`--clean` flag removes the Cargo lock to force a fresh resolution).
