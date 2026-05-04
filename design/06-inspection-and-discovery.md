# 06. Inspection and Discovery

## Table of Contents

1. [Purpose](#purpose)
2. [List Commands](#list-commands)
3. [Help Commands](#help-commands)
4. [Docs Command](#docs-command)
5. [Output Formats](#output-formats)

## Purpose

The CLI provides read-only inspection commands that let developers and automation tools understand the current workspace
state: what modules exist, what apps are configured, how the manifest resolves, and what the framework's schemas look
like. These commands have **no side effects** and are safe to run at any time.

## List Commands

### `list modules`

Shows all modules visible to the current workspace: local modules discovered from Cargo metadata, system modules from
the built-in registry, and their configuration status in the manifest.

```bash
cargo cyberfabric list modules [--env <env>] [--app <app>] [-p <path>] [--format table|json|yaml|toml]
```

Output fields:

| Field | Description |
|---|---|
| `name` | Module name |
| `package` | Cargo package name |
| `source` | `local`, `registry`, or `system` |
| `version` | Resolved version |
| `status` | `enabled`, `available`, `missing` |
| `apps` | Which manifest apps reference it |
| `capabilities` | Discovered capabilities (e.g., `db`, `grpc`) |

When `--env` and `--app` are provided, the `status` column reflects whether the module is enabled in that specific app.

### `list local-modules`

Shows only modules discovered from the workspace.

```bash
cargo cyberfabric list local-modules [-p <path>] [--verbose] [--format table|json|yaml|toml]
```

Output fields:

| Field | Description |
|---|---|
| `name` | Module name from `src/module.rs` |
| `package` | Cargo package name |
| `version` | Package version |
| `path` | Relative path from workspace root |
| `capabilities` | Discovered capabilities |
| `dependencies` | Declared module dependencies |
| `apps` | Manifest apps that reference it |

Reuses the existing `get_module_name_from_crate()` discovery logic.

### `list system-modules`

Shows the built-in system module registry.

```bash
cargo cyberfabric list system-modules [--verbose] [--registry <registry>] [--format table|json|yaml|toml]
```

Output includes: module name, crate name, latest version (when `--verbose`), capabilities, and whether the module is
used by any manifest app.

Built-in system module registry:

- `credstore`
- `file-parser`
- `api-gateway`
- `authn-resolver`
- `static-authn-plugin`
- `authz-resolver`
- `static-authz-plugin`
- `grpc-hub`
- `module-orchestrator`
- `nodes-registry`
- `oagw`
- `single-tenant-tr-plugin`
- `static-tr-plugin`
- `tenant-resolver`
- `types-registry`

When both `--verbose` and `--registry` are used, the CLI fetches registry metadata and `src/module.rs` details. Only
`crates.io` is supported initially.

### `list configs`

Shows runtime config files and their relationship to manifest apps.

```bash
cargo cyberfabric list configs [-p <path>] [--format table|json|yaml|toml]
```

Output fields:

| Field | Description |
|---|---|
| `path` | Config file path relative to workspace |
| `environment` | Linked environment from manifest (if any) |
| `app` | Linked app from manifest (if any) |
| `modules` | Module names with runtime config sections |
| `has_metadata` | Whether the config contains legacy module metadata |

### `list apps`

Shows manifest-defined apps with their configuration summary.

```bash
cargo cyberfabric list apps [--manifest <path>] [--format table|json|yaml|toml]
```

Output fields:

| Field | Description |
|---|---|
| `environment` | Environment name |
| `app` | App name |
| `config` | Config file path |
| `modules` | Module count |
| `run` | Run policy summary (watch, fips, otel) |
| `build` | Build outputs (binary, docker, helm) |
| `test` | Test policy summary (runner, sets) |

Example table output:

```text
Environment  App    Config                Modules  Run             Build
dev          app1   config/dev-app1.yml   3        watch,otel      binary
prod         app1   config/prod-app1.yml  2        fips,otel       binary,docker
```

## Help Commands

### `help schema`

Prints the schema reference for manifest, config, or module structures.

```bash
cargo cyberfabric help schema <manifest|config|module> [--section <path>] [--format markdown|json|yaml]
```

Examples:

```bash
cargo cyberfabric help schema manifest
cargo cyberfabric help schema manifest --section env.app.test
cargo cyberfabric help schema config --format json
cargo cyberfabric help schema module
```

Output includes:

- Schema version.
- Supported fields with types.
- Default values.
- Enum variants.
- Examples.
- Compatibility notes.

Schema help is generated from the same Rust types used by parsing (via derive macros or a build-time schema generator)
so documentation cannot drift from implementation.

### `help topic`

Prints short operational documentation for CyberFabric concepts.

```bash
cargo cyberfabric help topic <topic>
```

Available topics:

| Topic | Content |
|---|---|
| `manifest` | Manifest file purpose, location, schema overview |
| `module-refs` | How module references work (local vs registry) |
| `generated-server` | What `.cyberfabric/` contains and how it is used |
| `fips` | FIPS mode: what it enables, constraints, validation |
| `otel` | OpenTelemetry integration: features, config, endpoints |
| `docker` | Docker build process, Dockerfile expectations, build args |
| `helm` | Helm chart structure, values, packaging |
| `config-migration` | How to migrate from config-centric to manifest-first |
| `env-vars` | Environment variable expansion syntax and behavior |

Topics are embedded in the CLI binary as static strings. They link to schema help where relevant.

### `help docs` / `docs`

Resolves Rust source for a crate, module, or item from local workspace metadata, the local docs cache, or crates.io.

```bash
cargo cyberfabric help docs <rust-path> [flags]
cargo cyberfabric docs <rust-path> [flags]
```

`docs` is the primary command; `help docs` is an alias for discoverability within the `help` namespace.

## Docs Command

The `docs` command retains its current behavior with the existing flags:

```bash
cargo cyberfabric docs [-p <path>] [--registry <registry>] [--verbose] [--libs] [--version <version>] [--clean] [<query>]
```

### Behavior

- **Query required** unless `--clean` is used alone.
- **Local resolution first**: tries workspace metadata before network.
- **Cache-first registry fallback**: reuses cached sources before downloading.
- **Recursive re-export resolution**: follows re-exports across crate boundaries.
- **Library mapping mode** (`--libs`): prints `library_name -> package_name` mappings.
- **Cache cleaning** (`--clean`): removes the selected registry cache.

### Supported Queries

```text
cf-modkit
tokio::sync
cf-modkit::gts::plugin::BaseModkitPluginV1
serde::de::Deserialize
```

## Output Formats

All list and inspection commands support:

```text
--format table|json|yaml|toml
```

### Default Selection

- **Interactive terminal** (stdout is a TTY): `table`.
- **Non-interactive** (piped or redirected): `json`.

This can be overridden by the `CF_CLI_FORMAT` environment variable or the explicit `--format` flag. The flag always
takes highest precedence.

### JSON Contract

JSON output uses explicit field names, stable ordering, and no human-only formatting:

```json
{
  "environment": "dev",
  "app": "app1",
  "config": "config/dev-app1.yml",
  "modules": [
    {
      "name": "background-worker",
      "package": "background-worker",
      "source": "local",
      "version": "0.1.0",
      "status": "enabled",
      "capabilities": ["db"]
    }
  ]
}
```

JSON output schema is versioned alongside the CLI. Breaking changes to JSON output structure require a major CLI
version bump.

### Table Contract

Table output is human-friendly but not a stable contract. Column widths, alignment, and truncation may change
between releases. Automation should always use `--format json`.
