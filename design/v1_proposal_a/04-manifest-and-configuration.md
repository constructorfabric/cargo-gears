# 04. Manifest and Configuration

## Table of Contents

1. [Purpose](#purpose)
2. [Manifest Schema](#manifest-schema)
3. [Module References](#module-references)
4. [Runtime Configuration](#runtime-configuration)
5. [Manifest Commands](#manifest-commands)
6. [Validation Rules](#validation-rules)
7. [Discovery and Defaults](#discovery-and-defaults)
8. [Schema Versioning](#schema-versioning)
9. [Migration from Config-Centric Model](#migration-from-config-centric-model)
10. [Generated Server Flow](#generated-server-flow)

## Purpose

The manifest detaches the **orchestration layer** from the **configuration layer**. Today the runtime config contains
module dependency metadata that the CLI uses to shape generated `Cargo.toml` and `src/main.rs`. The manifest makes this
explicit:

- **Manifest** (`Cyberfabric.toml`): what the CLI builds, how it builds it, and which policies apply.
- **Runtime config** (`config/*.yml`): what the generated server reads at runtime.

This separation eliminates metadata duplication, reduces config-file complexity, and makes the CLI's behavior
inspectable without reading YAML runtime values.

## Manifest Schema

File name: `Cyberfabric.toml`

Format: TOML. Chosen because `toml_edit` supports in-place edits that preserve comments, which is critical for a
CLI that mutates the file programmatically.

```toml
schema_version = 1

[workspace]
root = "."                       # Workspace root, default "."
config_dir = "config"            # Directory for runtime configs, default "config"
generated_dir = ".cyberfabric"   # Directory for generated projects, default ".cyberfabric"

# Template registry (optional, overrides defaults)
[[templates.module]]
name = "background-worker"
source = "git"
url = "https://github.com/cyberfabric/cf-template-rust"
subfolder = "Modules/background-worker"
branch = "main"

[[templates.build]]
name = "docker"
source = "embedded"

# Environment / App definitions
[env.dev.app1]
config = "dev-app1.yml"          # Relative to config_dir

[[env.dev.app1.modules]]
name = "background-worker"
source = "local"

[[env.dev.app1.modules]]
name = "credstore"
source = "registry"
version = "0.4.2"
package = "cf-credstore"

[env.dev.app1.run]
watch = true
fips = false
otel = true

[env.dev.app1.build]
name = "app1"                    # Override generated project name
profile = "debug"
outputs = ["binary"]

[env.dev.app1.lint]
clippy = true
fmt = true
skip_dylint = false

[env.dev.app1.test]
runner = "nextest"
sets = ["unit", "integration"]
coverage = false

# Production environment
[env.prod.app1]
config = "prod-app1.yml"

[[env.prod.app1.modules]]
name = "background-worker"
source = "local"

[[env.prod.app1.modules]]
name = "credstore"
source = "registry"
version = "0.4.2"
package = "cf-credstore"

[env.prod.app1.run]
watch = false
fips = true
otel = true

[env.prod.app1.build]
name = "app1"
profile = "release"
outputs = ["binary", "docker"]

[env.prod.app1.build.docker]
image = "registry.example.com/app1"
tag = "1.2.3"

[env.prod.app1.test]
runner = "cargo-test"
sets = ["unit", "integration", "e2e"]
coverage = true
```

### Schema Design Decisions

- **`BTreeMap` for environments and apps** ensures deterministic ordering in serialization and iteration.
- **Environment-first hierarchy** (`[env.<environment>.<app>]`) groups all apps under their environment. This inverts
  the v1 design (`[env.<app>.<environment>]`) to match the mental model of "select an environment, then an app within
  it" which aligns with `--env <env> --app <app>` flag ordering.
- **Modules are arrays of tables** (`[[env.dev.app1.modules]]`) rather than inline tables for readability with many
  modules.
- **Policy sections** (`run`, `build`, `lint`, `test`) are optional; the CLI applies convention defaults when omitted.
- **`schema_version`** is a top-level integer for forward compatibility. See [Schema Versioning](#schema-versioning).
- **No `remote` source type.** The v1 design had `local`, `remote`, and `registry`. The `remote` source is removed
  because its semantics (Git-based dependency) overlap with `registry` for published crates and with `local` path
  dependencies. If Git-based dependencies are needed in the future, they should be added as a new explicit source type
  (e.g., `git`) with clear semantics rather than the ambiguous `remote`.

## Module References

Each module reference has:

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Module name used in the CyberFabric runtime |
| `source` | Yes | One of `local`, `registry` (see [note on removed `remote` source](#schema-design-decisions)) |
| `version` | When `registry` | Exact semver version |
| `package` | When `registry` | Cargo package name (crate name) |
| `features` | No | List of Cargo features to enable |
| `default_features` | No | Whether to use default features (default: `true`) |
| `dependencies` | No | Module-level runtime dependency names |

### Source Types

- **`local`**: Discovered from the workspace via `cargo_metadata`. The CLI resolves the module name to a package name
  and version from workspace metadata and `src/module.rs`.
- **`registry`**: Resolved from the configured registry (default: crates.io). Requires explicit `version` and
  `package`.

### Resolution Rules

1. Manifest module reference is the primary input.
2. For `local` modules, the CLI discovers package name, version, and capabilities from the workspace.
3. For `registry` modules, the CLI validates the version exists on the registry.
4. If a local module and a registry module resolve to the same package name, validation fails.
5. Resolution produces a `ResolvedModule` struct used by code generation.

## Runtime Configuration

Runtime config files remain YAML and contain only runtime values:

```yaml
# config/dev-app1.yml
server:
  host: "0.0.0.0"
  port: 8080

opentelemetry:
  tracing:
    endpoint: "http://localhost:4317"

database:
  servers:
    primary:
      engine: postgres
      host: localhost
      port: 5432
      user: app
      password: "${DB_PASSWORD}"
      dbname: appdb

modules:
  background-worker:
    interval_secs: 10
  credstore:
    vault_url: "http://localhost:8200"
```

### What Moves Out of Runtime Config

The following fields currently in runtime config move to the manifest:

| Field | Old Location (config) | New Location (manifest) |
|---|---|---|
| Module package name | `modules.<name>.metadata.package` | `env.<env>.<app>.modules[].package` |
| Module version | `modules.<name>.metadata.version` | `env.<env>.<app>.modules[].version` |
| Module source | `modules.<name>.metadata.source` | `env.<env>.<app>.modules[].source` |
| Module features | `modules.<name>.metadata.features` | `env.<env>.<app>.modules[].features` |
| Module default_features | `modules.<name>.metadata.default_features` | `env.<env>.<app>.modules[].default_features` |

Runtime config retains module-specific runtime settings (e.g., `interval_secs`, `vault_url`) under `modules.<name>`.

## Manifest Commands

### `manifest validate`

```bash
cargo cyberfabric manifest validate [--manifest <PATH>] [--format table|json]
```

Validates:

- TOML syntax and schema conformance.
- All module references resolve (local discovery + registry check).
- No duplicate package names across module lists.
- Config files referenced by apps exist on disk.
- Policy sections use valid enum values.
- Cross-environment consistency (e.g., prod modules are a subset of dev modules, or explicitly divergent).

Output: list of diagnostics with severity, code, message, location, and suggestion.

### `manifest render`

```bash
cargo cyberfabric manifest render --env <env> --app <app> [--manifest <PATH>] [--format json|yaml|toml|table]
```

Produces the fully resolved generation model without executing any build:

- Resolved config path.
- Resolved module list with package names, versions, features.
- Generated Cargo dependencies.
- Generated features.
- Generated `main.rs` inputs.
- Selected lint/test/run/build policies.

This command is critical for debugging and for LLM-driven automation because it exposes the exact input to code
generation.

### `manifest add`

```bash
cargo cyberfabric manifest add <env> <app> [<module-ref>...] [--config <config>] [--manifest <PATH>]
```

Adds a new environment/app entry or appends modules to an existing entry.

### `manifest edit`

```bash
cargo cyberfabric manifest edit <env> <app> [flags] [--manifest <PATH>]
```

Modifies policy settings on an existing app entry.

### `manifest rm`

```bash
cargo cyberfabric manifest rm <env> <app> [<module-ref>] [--manifest <PATH>]
```

Removes an environment/app entry or a specific module from an existing entry.

## Validation Rules

Validation fails early when:

| Rule | Severity | Code |
|---|---|---|
| Environment/app does not exist | Error | `E001` |
| Module reference has unknown source type | Error | `E002` |
| Local module cannot be discovered in workspace | Error | `E003` |
| Registry module version does not exist | Error | `E004` |
| Duplicate package name across modules | Error | `E005` |
| Referenced config file does not exist on disk | Error | `E006` |
| Run/build/test policy uses unsupported enum value | Error | `E007` |
| Production environment uses `watch = true` | Warning | `W001` |
| FIPS requested with potentially incompatible modules | Warning | `W002` |
| Docker output selected without `build.docker.image` | Error | `E008` |
| Helm output selected without chart directory | Error | `E009` |
| Schema version newer than CLI supports | Error | `E010` |
| Deprecated field used | Warning | `W003` |

Validation output is deterministic. In `--format json` mode, diagnostics are an array of objects with `severity`,
`code`, `message`, `location`, and `suggestion` fields.

## Discovery and Defaults

### Manifest Discovery

When `--manifest` is not provided, the CLI searches for `Cyberfabric.toml` by walking up from the workspace root
(or current directory). This matches Cargo's `Cargo.toml` discovery behavior.

### Default Environment and App

When `--env` and `--app` are omitted:

1. If the manifest has exactly one environment with one app, it is selected automatically.
2. If the manifest has one environment with multiple apps, the CLI prompts (interactive) or fails (non-interactive).
3. If the manifest has multiple environments, the CLI prompts or fails.

This behavior ensures simple projects need no flags while complex projects require explicit selection.

### Fallback to Config-Only Mode

When no `Cyberfabric.toml` exists and `--config` is provided, the CLI falls back to the current config-centric
behavior. This ensures backward compatibility and supports simple one-off usage.

## Schema Versioning

The `schema_version` field is a positive integer at the top of `Cyberfabric.toml`:

```toml
schema_version = 1
```

Rules:

- The CLI refuses to load a manifest with a `schema_version` higher than it supports.
- Schema changes that add optional fields do not bump the version.
- Schema changes that rename, remove, or change the semantics of existing fields bump the version.
- The CLI can read older schema versions and apply internal migration logic.
- `manifest validate` warns about deprecated fields from older schemas.

## Manifest Migration

When a new schema version is released, the CLI provides a migration command:

```bash
cargo cyberfabric manifest migrate [--from <version>] [--to <version>] [--manifest <path>] [--dry-run]
```

This command:

1. Reads the manifest with the old schema.
2. Applies migration rules (field renames, restructuring, default injection).
3. Writes the updated manifest using `toml_edit` to preserve comments.
4. Prints a diff of changes.

`--dry-run` shows the diff without writing. See
[12-versioning-and-compatibility.md](./12-versioning-and-compatibility.md#migration-between-schema-versions) for the
full versioning policy.

## Migration from Config-Centric Model

The `generate manifest --from-config` command generates a `Cyberfabric.toml` from an existing config file:

```bash
cargo cyberfabric generate manifest --from-config config/quickstart.yml --env dev --app quickstart
```

Migration steps:

1. Parse the existing config file.
2. Extract module metadata (package, version, source, features).
3. Generate manifest entries with discovered metadata.
4. Write `Cyberfabric.toml`.
5. Optionally strip metadata from the config file (with `--strip-metadata`).
6. Print a diff summary showing what moved.

The migration is non-destructive by default: the config file is not modified unless `--strip-metadata` is passed.

## Generated Server Flow

With the manifest-first model, the generation pipeline becomes:

1. Load manifest (`Cyberfabric.toml`).
2. Select environment and app.
3. Load runtime config path from manifest, unless overridden by `--config`.
4. Resolve modules from manifest references.
5. Merge discovered metadata with manifest constraints.
6. Generate `.cyberfabric/<env>-<app>/` (or configured name).
7. Set `CF_CLI_CONFIG` to the runtime config path.
8. Execute the selected operation (`cargo run`, `cargo build`, `docker build`).

The generated `main.rs` remains unchanged: it reads `CF_CLI_CONFIG` at runtime and calls `run_server(config)`.
