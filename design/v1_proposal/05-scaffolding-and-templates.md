# 05. Scaffolding and Templates

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Generation Commands](#generation-commands)
4. [Template Registry](#template-registry)
5. [Workspace Scaffolding](#workspace-scaffolding)
6. [Module Scaffolding](#module-scaffolding)
7. [Configuration Scaffolding](#configuration-scaffolding)
8. [Manifest Scaffolding](#manifest-scaffolding)
9. [Build Template Scaffolding](#build-template-scaffolding)
10. [CI Template Scaffolding](#ci-template-scaffolding)
11. [Agent and Skill Scaffolding](#agent-and-skill-scaffolding)
12. [Post-Generation Hooks](#post-generation-hooks)
13. [Template Versioning](#template-versioning)

## Purpose

Template generation is a first-class CLI surface. Every standard artifact in a CyberFabric workspace can be scaffolded
through a deterministic, repeatable generation command. This ensures that new workspaces, modules, configs, build
files, and automation guides all follow approved patterns from day one.

## Current Behavior

- `init` uses `cargo-generate` against `cf-template-rust`, subfolder `Init`.
- `mod add` uses `cargo-generate` against `cf-template-rust`, subfolder `Modules/<template>`.
- Available module templates: `background-worker`, `api-db-handler`, `rest-gateway`.
- Docker support exists in `deploy` via an embedded Dockerfile.

## Generation Commands

```text
cargo cyberfabric generate workspace <path> [flags]
cargo cyberfabric generate module <template> [--name <name>] [flags]
cargo cyberfabric generate config <kind> [flags]
cargo cyberfabric generate manifest [flags]
cargo cyberfabric generate build <kind> [flags]
cargo cyberfabric generate ci <provider> [flags]
cargo cyberfabric generate agents [flags]
cargo cyberfabric generate skill [flags]
```

Aliases:

- `init <path>` is a permanent alias for `generate workspace <path>`.
- `mod add <template>` is a permanent alias for `generate module <template>`.

## Template Registry

The CLI resolves templates through a priority chain:

1. **`--local-path`**: explicit local directory, highest priority.
2. **Manifest-defined registry**: `[[templates.*]]` entries in `Cyberfabric.toml`.
3. **Default Git repository**: `https://github.com/cyberfabric/cf-template-rust`.
4. **Embedded fallbacks**: small files (Dockerfile, .dockerignore, AGENTS.md) bundled in the CLI binary.

### Manifest Registry Schema

```toml
[[templates.module]]
name = "background-worker"
source = "git"
url = "https://github.com/cyberfabric/cf-template-rust"
subfolder = "Modules/background-worker"
branch = "main"

[[templates.module]]
name = "grpc-service"
source = "local"
path = "templates/grpc-service"

[[templates.build]]
name = "docker"
source = "embedded"

[[templates.build]]
name = "helm"
source = "git"
url = "https://github.com/cyberfabric/cf-template-rust"
subfolder = "Build/helm"
branch = "main"
```

### Source Types

| Source | Description |
|---|---|
| `git` | Cloned from a Git URL with optional branch and subfolder |
| `local` | Read from a local filesystem path |
| `embedded` | Bundled in the CLI binary as a compile-time resource |

## Workspace Scaffolding

```bash
cargo cyberfabric generate workspace ./my-app [--name my-app] [--profile <profile>] [--with <extras>]
cargo cyberfabric init ./my-app [--name my-app] [--profile <profile>] [--with <extras>]
```

### Profiles

The `--profile` flag selects a workspace template tier:

| Profile | Description |
|---|---|
| `minimal` | Workspace skeleton with one hello-world module. Default. |
| `service` | Adds a REST gateway module, config with database, and Docker template. |
| `platform` | Full platform setup: multiple modules, Helm chart, CI workflow, AGENTS.md, SKILL.md. |

### Extras

The `--with` flag enables optional additions (comma-separated):

| Extra | What It Generates |
|---|---|
| `docker` | `Dockerfile` and `.dockerignore` |
| `helm` | `charts/<app>/` Helm chart skeleton |
| `ci-github` | `.github/workflows/ci.yml` (equivalent to `generate ci github`) |
| `agents` | `AGENTS.md` |
| `skill` | `SKILL.md` |

### Generated Layout

For the default `minimal` profile:

```text
my-app/
├── Cargo.toml                   # Workspace manifest
├── Cyberfabric.toml             # CLI manifest
├── .gitignore                   # Includes .cyberfabric/
├── config/
│   └── quickstart.yml           # Default runtime config
└── modules/
    └── hello-world/             # Starter module
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            └── module.rs
```

### Behavior

- Creates target directory if it does not exist.
- Fails if path exists and is a file.
- Fails if path exists as a non-empty directory (use `--force` to override).
- Infers project name from the final path segment unless `--name` is provided.
- Initializes a Git repository.
- Runs `cargo check` to verify the generated workspace compiles (can be skipped with `--skip-check`).

## Module Scaffolding

```bash
cargo cyberfabric generate module <template> [--name <name>] [-p <path>] [flags]
cargo cyberfabric mod add <template> [-p <path>] [flags]
```

### Available Templates

| Template | Status | Description |
|---|---|---|
| `background-worker` | Existing | Periodic or event-driven background task module |
| `api-db-handler` | Existing | REST API module with database integration |
| `rest-gateway` | Existing | HTTP gateway and routing module |
| `grpc-service` | Proposed | gRPC server module |
| `oop-module` | Proposed | Out of process style module |

### Flags

| Flag | Description |
|---|---|
| `--name <name>` | Override the generated module name (defaults to template name) |
| `--add-to-manifest <env.app,...>` | Add the module to specified manifest apps |
| `--add-to-config <config,...>` | Add default runtime config for the module |
| `--local-path <path>` | Use a local template directory |
| `--git <url>` | Override the template Git URL |
| `--subfolder <name>` | Override the template subfolder |
| `--branch <name>` | Override the Git branch |

### Behavior

- Requires `modules/` directory to exist in the workspace.
- Creates `modules/<name>/` with the template content.
- Fails if the module directory already exists.
- Adds the module to `workspace.members` in the root `Cargo.toml`.
- Promotes module dependencies to `workspace.dependencies`.
- Rewrites module dependency declarations to `workspace = true`.
- Adds `lints.workspace = true` to the generated module.
- If the module template includes `sdk/`, adds it as a workspace member.
- Optionally wires the module into manifest apps and runtime config.

## Configuration Scaffolding

```bash
cargo cyberfabric generate config <kind> [--env <env>] [--app <app>] [--name <filename>] [-p <path>]
```

### Config Kinds

| Kind | Description |
|---|---|
| `dev` | Development config with relaxed settings, local endpoints |
| `prod` | Production config with hardened settings, placeholder secrets |
| `test` | Test config with in-memory or ephemeral resources |

### Behavior

- Generates a YAML file in `<workspace>/config/`.
- Filename defaults to `<kind>-<app>.yml` or uses `--name`.
- Config contains only runtime values, no module metadata.
- If `--app` is provided and a manifest exists, pre-populates module runtime sections for configured modules.
- Prints a summary of generated files.

## Manifest Scaffolding

```bash
cargo cyberfabric generate manifest [--from-config <path>] [--env <env>] [--app <app>] [--include-local-modules] [-p <path>]
```

### Behavior

- Creates `Cyberfabric.toml` in the workspace root.
- If `--from-config` is provided, extracts module metadata from the config and generates manifest entries (see
  [04-manifest-and-configuration.md](./04-manifest-and-configuration.md#migration-from-config-centric-model)).
- If `--include-local-modules` is provided, discovers all local modules and adds them.
- Infers a default `dev` environment if `--env` is not provided.
- Infers an app name from the workspace package name if `--app` is not provided.
- Fails if `Cyberfabric.toml` already exists (use `--force` to overwrite).

## Build Template Scaffolding

```bash
cargo cyberfabric generate build <kind> [-p <path>]
```

### Build Kinds

| Kind | Description |
|---|---|
| `docker` | `Dockerfile` and `.dockerignore` |
| `compose` | `docker-compose.yml` for local development dependencies |
| `helm` | Helm chart skeleton in `charts/<app>/` |

### Docker Template

The generated Dockerfile:

- Uses a multi-stage build (builder + runtime).
- Uses `cargo-chef` for dependency caching.
- Accepts build args: `BUILDER_MANIFEST`, `BUILD_MODE`, `ARTIFACT_NAME`, `LOCAL_CONFIG_PATH`, `CONFIG_EXT`.
- Copies the runtime config into the image.
- Sets `CF_CLI_CONFIG` as the default entrypoint environment variable.
- Does not copy `.env` or secret files.

### Helm Template

The generated chart includes:

- `Chart.yaml` with app version and chart version.
- `values.yaml` with image repository/tag, config mount, env vars, ports, probes, resources.
- `templates/deployment.yaml`, `templates/service.yaml`, `templates/configmap.yaml`.

## CI Template Scaffolding

CI workflow templates are separate from build artifact templates because they serve a different purpose: configuring
continuous integration pipelines rather than producing deployable artifacts.

```bash
cargo cyberfabric generate ci <provider> [-p <path>]
```

### CI Providers

| Provider | What It Generates |
|---|---|
| `github` | `.github/workflows/ci.yml` with validate, lint, test, and build steps |

The generated workflow uses manifest-driven commands (see
[11-ci-and-automation.md](./11-ci-and-automation.md#ci-pipeline-patterns) for the recommended pattern). Additional
providers (e.g., `gitlab`, `azure`) can be added as templates become available.

## Agent and Skill Scaffolding

```bash
cargo cyberfabric generate agents [-p <path>]
cargo cyberfabric generate skill [-p <path>]
```

### AGENTS.md

Generated `AGENTS.md` includes:

- Project commands (init, build, run, test, lint, deploy).
- Formatting and lint expectations.
- Module conventions and naming rules.
- Manifest/config split explanation.
- Known generated paths (`.cyberfabric/`).
- Testing requirements.

### SKILL.md

Generated `SKILL.md` includes:

- CLI command reference optimized for LLM consumption.
- Common workflows with copy-pasteable commands.
- Schema snippets for manifest and config.
- Examples that prefer manifest-first commands.
- Warning that `.cyberfabric/` contents are derived output and should not be manually edited.

Both files are generated from maintained templates so they stay in sync with CLI changes.

## Post-Generation Hooks

After any generation command, the CLI runs post-generation hooks:

1. **Format generated Rust files** with `rustfmt` if available.
2. **Validate generated TOML/YAML** for syntax correctness.
3. **Print a summary** of created/modified files.
4. **Suggest next steps** (e.g., "Run `cargo cyberfabric run` to start the server").

Hooks are deterministic and do not make network calls. They can be skipped with `--no-hooks`.

## Template Versioning

Templates are versioned alongside the CLI. The default Git template repository uses branches or tags that correspond
to CLI version ranges:

- `main` branch: latest templates compatible with the current CLI release.
- `v1` branch: templates for schema version 1.
- Tags: `cli-0.1.0`, `cli-0.2.0`, etc.

When the CLI is installed from a release, it pins the default template branch to the matching version. Development
builds use `main`.

Template version mismatches (e.g., template requires a newer schema version than the CLI supports) are detected
during generation and reported as errors with an upgrade suggestion.
