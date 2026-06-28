---
title: Gears.toml manifest
description: The Gears.toml manifest schema — workspace defaults, apps, environments, modules, and policies.
sidebar:
  label: Gears.toml manifest
  order: 4
---

The `Gears.toml` file at the workspace root is the source of truth for what the
CLI generates and orchestrates. It declares workspace-level defaults and
per-app/environment overrides. Runtime values live in separate YAML config files.

## Top-level structure

```toml
[workspace]
# workspace-level defaults

[apps.<app>.<env>]
# per-app/environment configuration

[templates]
# optional template registry overrides
```

## `[workspace]`

Workspace-level defaults for paths and schema version.

| Field | Type | Default | Description |
|---|---|---|---|
| `version` | `u32` | `1` | Schema version (currently always 1) |
| `root` | `PathBuf` | _none_ | Workspace root override (relative to manifest directory) |
| `config-dir` | `PathBuf` | `config` | Directory containing config YAML files |
| `generated-dir` | `PathBuf` | `.gears` | Directory for generated server projects |
| `global_env` | `Environment` | _none_ | Global environment inherited by all apps (optional) |

Example:

```toml
[workspace]
config-dir = "config"
generated-dir = ".gears"
```

## `[apps.<app>.<env>]`

Each app is a map of environment names to environment configurations.

| Field | Type | Default | Description |
|---|---|---|---|
| `config` | `PathBuf` | _required_ | Config YAML path relative to `config-dir` |
| `modules` | `Vec<ModuleRef>` | `[]` | Modules to include in the generated server |
| `run` | `Option<RunPolicy>` | _none_ | Runtime policy overrides |
| `build` | `Option<BuildPolicy>` | _none_ | Build policy overrides |
| `lint` | `LintPolicy` | _default_ | Lint policy overrides |
| `test` | `TestPolicy` | _default_ | Test policy overrides |

Example:

```toml
[apps.app1.dev]
config = "app1-dev.yml"

[[apps.app1.dev.modules]]
source = "local"
name = "background-worker"

[apps.app1.dev.run]
fips = false
otel = true

[apps.app1.dev.build]
profile = "debug"

[apps.app1.dev.lint]
clippy = true
fmt = true
```

## Module references

Modules are declared with a `source` tag that distinguishes local workspace
modules from remote (published) modules.

### Local module

```toml
[[apps.app1.dev.modules]]
source = "local"
name = "background-worker"
# version = "0.1"    # optional, overrides discovered version
# package = "cf-background-worker"  # optional, overrides discovered package name
```

### Remote module

```toml
[[apps.app1.dev.modules]]
source = "remote"
name = "credstore"
package = "cf-credstore"
version = "0.4.2"
# registry = "crates.io"  # optional
```

## `[run]` — RunPolicy

Runtime policy for watch mode, FIPS, and OpenTelemetry.

| Field | Type | Default | Description |
|---|---|---|---|
| `watch` | `WatchPolicy` | _see below_ | Watch-mode settings |
| `fips` | `bool` | `false` | Enable FIPS mode |
| `otel` | `bool` | `false` | Enable OpenTelemetry |

### `[run.watch]` — WatchPolicy

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | `bool` | `true` | Enable file watching in run mode |
| `include` | `Vec<PathBuf>` | `[]` | Paths to watch (replaces default watch set) |
| `exclude` | `Vec<PathBuf>` | `[]` | Paths to exclude from the effective watch set |

```toml
[apps.app1.dev.run]
fips = false
otel = true

[apps.app1.dev.run.watch]
enabled = true
exclude = ["target/", ".gears/"]
```

## `[build]` — BuildPolicy

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `Option<String>` | _none_ | Override generated project name (default: `<app>-<env>`) |
| `profile` | `Option<BuildProfile>` | _none_ | Build profile: `debug`, `release`, or custom string |
| `clean` | `Option<bool>` | _none_ | Remove `Cargo.lock` before building |

```toml
[apps.app1.prod.build]
profile = "release"
clean = true
```

## `[lint]` — LintPolicy

| Field | Type | Default | Description |
|---|---|---|---|
| `ref` | `Option<String>` | _none_ | Reference another environment's lint policy |
| `clippy` | `bool` | `true` | Run Clippy |
| `fmt` | `bool` | `true` | Run `cargo fmt --check` |
| `feature-set-test` | `bool` | `true` | Enable feature-set lint testing |
| `dylint` | `Option<Dylint>` | _none_ | Dylint configuration |

### `[lint.dylint]`

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | `bool` | `true` | Enable Dylint rules |
| `skip` | `Vec<String>` | `[]` | Rule names to skip (passed as allowed rustc lints) |

```toml
[apps.app1.dev.lint]
clippy = true
fmt = true

[apps.app1.dev.lint.dylint]
enabled = true
skip = ["de0301_no_infra_in_domain"]
```

## `[test]` — TestPolicy

| Field | Type | Default | Description |
|---|---|---|---|
| `ref` | `Option<String>` | _none_ | Reference another environment's test policy |
| `runner` | `TestRunner` | `nextest` | Test runner: `cargo` or `nextest` |
| `feature-set` | `BTreeMap<String, ModuleFeatureSet>` | _empty_ | Per-module feature matrix |
| `custom-command` | `Option<String>` | _none_ | Custom test command override |

### Feature sets

Each module's feature set is a list of `FeatureSet` entries with a `mode` tag:

```toml
[apps.app1.dev.test]
runner = "nextest"

[apps.app1.dev.test.feature-set."background-worker"]
mode = "all-features"

[[apps.app1.dev.test.feature-set."api-db-handler"]]
mode = "features"
features = ["postgres", "metrics"]
```

Supported modes:

- **`default-features`** — use Cargo defaults
- **`all-features`** — pass `--all-features`
- **`no-default-features`** — pass `--no-default-features`
- **`features`** — pass `--no-default-features --features <LIST>`

## `[templates]` — TemplateRegistry

Optional registry of template sources for `generate` commands. Overrides the
built-in template defaults.

```toml
[templates.module."my-template"]
source = "git"
url = "https://github.com/my-org/my-template"
branch = "main"
subfolder = "Modules/my-template"

[templates.config."staging"]
source = "local"
path = "~/dev/my-config-templates/staging"
```

Template sources use a `source` tag:

- **`git`** — `url`, `revision`, `tag`, `branch`, `subfolder` (all optional except `url`)
- **`local`** — `path`
- **`embedded`** — use the built-in template

## Validation

Validate the manifest and all app/environment entries:

```bash
cargo gears manifest validate
```

List configured app/environment pairs with resolved config paths:

```bash
cargo gears manifest ls --format table
```

## See also

- [Command reference](/cli/commands/) — `build`, `run`, `lint`, `test`, `manifest`
- [Getting started](/cli/getting-started/) — end-to-end flow using the manifest
