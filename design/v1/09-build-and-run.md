# 08. Build and Run

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

Build, run, and deploy are the action commands that take the resolved generation model and produce executable artifacts.
All three share the same generation pipeline (manifest resolution -> module resolution -> code generation) and diverge
only at the final execution step.

## Run Command

### Synopsis

```bash
cargo cyberfabric run [--env <env>] [--app <app>] [-c <config>] [-p <path>] [--name <name>] [--watch] [--otel] [--fips] [--release] [--clean] [--dry-run]
```

### Behavior

1. Resolve manifest app or fall back to `--config`.
2. Resolve modules and build the generation model.
3. Generate `.cyberfabric/<env>-<app>/` (or `<config-stem>/` in config-only mode).
4. Set `CF_CLI_CONFIG` to the resolved runtime config path.
5. Execute `cargo run` inside the generated project.

### Backward Compatibility

The current `--config` flow is fully preserved:

```bash
# Manifest-first (new)
cargo cyberfabric run --env dev --app app1

# Config-only (existing, still supported)
cargo cyberfabric run -c config/quickstart.yml -p /tmp/cf-demo
```

When both manifest app and `--config` are provided, the explicit config path overrides the manifest-declared config
but not the manifest module selection.

### Name Resolution

| Input                      | Generated Project Path      |
|----------------------------|-----------------------------|
| `--env dev --app app1`     | `.cyberfabric/dev-app1/`    |
| `-c config/quickstart.yml` | `.cyberfabric/quickstart/`  |
| `--name demo-server`       | `.cyberfabric/demo-server/` |

`--name` always takes highest precedence.

## Build Command

### Synopsis

```bash
cargo cyberfabric build [--env <env>] [--app <app>] [-c <config>] [-p <path>] [--name <name>] [--output <output>] [--otel] [--fips] [--release] [--clean] [--dry-run]
```

### Behavior

Same pipeline as `run`, but invokes `cargo build` instead of `cargo run` and supports multiple output types.

### Build Output Types

```bash
cargo cyberfabric build --env prod --app app1 --output binary
cargo cyberfabric build --env prod --app app1 --output docker
cargo cyberfabric build --env prod --app app1 --output all
```

| Output   | Description                                                |
|----------|------------------------------------------------------------|
| `binary` | `cargo build` in the generated project. Default.           |
| `docker` | Build a Docker image (subsumes current `deploy` behavior). |
| `all`    | Binary + Docker (whatever is configured).                  |

When `--output` is omitted, the manifest `build.outputs` list is used. If no manifest exists, `binary` is the default.

## Build Outputs

### Binary Build

```toml
[env.app1.prod.build]
profile = "release"
name = "app1"
outputs = ["binary"]
```

The generated project path is deterministic: `.cyberfabric/<env>-<app>/`. The binary is placed in the generated
project's `target/<profile>/` directory.

### Docker Image

```toml
[env.app1.prod.build]
outputs = ["binary", "docker"]

[env.app1.prod.build.docker]
image = "registry.example.com/app1"
tag = "1.2.3"
dockerfile = "Dockerfile"         # optional, defaults to workspace root Dockerfile
```

Docker build process:

1. Ensure Dockerfile exists (generate if missing, with notice).
2. Build binary if not already built.
3. Invoke `docker build` with controlled arguments.
4. Tag the image.
5. Print summary.

### Build Summary

After all outputs are produced, the CLI prints a summary:

```text
Built app prod/app1
  binary: .cyberfabric/prod-app1/target/release/app1
  image:  registry.example.com/app1:1.2.3
```

In `--format json` mode:

```json
{
  "environment": "prod",
  "app": "app1",
  "outputs": {
    "binary": ".cyberfabric/prod-app1/target/release/app1",
    "docker": "registry.example.com/app1:1.2.3"
  },
  "duration_ms": 45000
}
```

## Feature Flags

FIPS and OpenTelemetry are Cargo features on the generated project, controlled by the manifest or CLI flags.

### FIPS

```toml
[env.app1.prod.run]
fips = true
```

Or via CLI: `--fips`.

The CLI passes `-F fips` to the generated project's Cargo invocation.

Validation:

- Warn if `fips = true` in a non-production environment.
- Fail if selected modules declare a known incompatible capability.

### OpenTelemetry

```toml
[env.app1.prod.run]
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
cargo cyberfabric run --env dev --app app1 --watch
```

### Watched Paths

Watch mode observes:

- Workspace module source directories (path-based dependencies).
- Runtime config file.
- Manifest file (`Cyberfabric.toml`).
- Workspace `Cargo.toml`.

When a change is detected, the CLI:

1. Regenerates the server project if manifest or workspace metadata changed.
2. Restarts the `cargo run` process.

### Manifest Watch Policy

To override default values(detected members, configs and generated files)

```toml
[env.app1.dev.run.watch]
enabled = true
paths = ["modules", "config/dev-app1.yml", "Cyberfabric.toml"]
ignore = ["target", ".cyberfabric"]
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

For all action commands (`run`, `build`, `deploy`), settings are resolved in this order:

1. **CLI flags** (highest precedence).
2. **App manifest policy** (`env.<app>.<env>.run`, `.build`).
3. **CLI defaults** (lowest precedence).

### Resolved Run Model

The resolved model can be inspected with `--dry-run`:

```bash
cargo cyberfabric run --env dev --app app1 --dry-run --format json
```

```json
{
  "environment": "dev",
  "app": "app1",
  "config": "config/dev-app1.yml",
  "generated_project": ".cyberfabric/dev-app1",
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

`--dry-run` is available on `run`, `build`, and `deploy`. It prints the resolved execution plan without performing
side effects:

- No files are generated.
- No Cargo commands are invoked.
- No Docker images are built.

Dry-run output supports `--format json` for structured consumption. This is the stable contract for CI pre-flight
checks, LLM planning, and debugging.

> **Note on `--format` for action commands:** Outside of `--dry-run`, action commands (`run`, `build`, `deploy`) do
> not support `--format` because their primary output is subprocess execution (Cargo/Docker stdout/stderr). The
> `--format json` flag is only meaningful with `--dry-run`, where it serializes the resolved generation model.
> Build summary output is always printed as a human-readable summary to stderr; use `--dry-run --format json` for
> machine-parseable pre-flight data.

## Generated Server Project

The generated project under `.cyberfabric/<name>/` contains:

```text
.cyberfabric/<name>/
├── Cargo.toml             # Generated from manifest module list
├── .cargo/
│   └── config.toml        # Cargo configuration
└── src/
    └── main.rs            # Generated entry point
```

### `Cargo.toml`

Dependencies are derived from the resolved module list. Each module becomes a dependency with its source (path for
local, version for registry), features, and default-features.

### `main.rs`

The generated `main.rs`:

- Reads `CF_CLI_CONFIG` from the environment.
- Loads the runtime config.
- Calls `modkit::run_server(config)`.
- Does not embed any hardcoded paths.

### `.cyberfabric/` Is Derived Output

The `.cyberfabric/` directory is **derived output** that can be regenerated at any time. It is:

- Listed in `.gitignore`.
- Never manually edited.
- Regenerated by every `run`, `build`, and `deploy` invocation.
- Safe to delete (`--clean` flag removes the Cargo lock to force a fresh resolution).
