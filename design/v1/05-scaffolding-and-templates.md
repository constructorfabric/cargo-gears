# 05. Template Generation

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Commands](#proposed-commands)
4. [Template Registry](#template-registry)
5. [Initial Scaffolding](#initial-scaffolding)
6. [Module Scaffolding](#module-scaffolding)
7. [Configuration Scaffolding](#configuration-scaffolding)
8. [Manifest Scaffolding](#manifest-scaffolding)
9. [Build Templates](#build-templates)
10. [Agent and Skill Templates](#agent-and-skill-templates)
11. [CI Template Scaffolding](#ci-template-scaffolding)

## Purpose

Template generation should become a first-class CLI surface instead of being
limited to `init` and `mod add`. The goal is to let users scaffold a full
CyberFabric workspace, individual modules, runtime config, CLI manifest, build
artifacts, and automation guidance files.

## Current Behavior

Current generation behavior:

- `init` uses `cargo-generate` against `cf-template-rust`, subfolder `Init`.
- `mod add` uses `cargo-generate` against subfolder `Modules/<template>`.
- Available module templates are `background-worker`, `api-db-handler`, and `rest-gateway`.

## Proposed Commands

Keep existing commands and add a normalized generation namespace:

```text
cargo cyberfabric generate workspace <path> [flags]
cargo cyberfabric generate module --template <template> [--name <name>] [flags]
cargo cyberfabric generate config <kind> [flags]
cargo cyberfabric generate manifest [flags]
cargo cyberfabric generate build <kind> [flags]
cargo cyberfabric generate ci <provider> [flags]
cargo cyberfabric generate agents [flags]
cargo cyberfabric generate skill [flags]
```

Aliases:

- `new` will remain as an alias for `generate workspace`.

## Template Registry

We'll provide a set of templates to be used in the generation commands.
However, the developer can add their own templates by:

1. Manifest-defined template registry
2. Embedded fallback templates for small files

Catalog shape:

```toml
[[templates.module]]
name = "background-worker"
source = "git"
subfolder = "Modules/background-worker"

[[templates.build]]
name = "docker"
source = "embedded"

[[templates.config]]
name = "grpc-service"
source = "local"
path = "templates/grpc-service"

[[templates.agents]]
name = "custom-agent"
source = "git"
url = "ssh://github.com/cyberfabric/cf-template-rust.git"
branch = "main"
subfolder = "Agents/custom-agent"
```

This catalog is optional and can be placed in `Cyberfabric-templates.toml` or by its own section in the manifest.

## Initial Scaffolding

Command:

```text
cargo cyberfabric generate workspace ./my-app [--name my-app]
cargo cyberfabric init ./my-app [--name my-app]
```

Should create:

- `Cargo.toml` workspace
- `modules/` (with a simple module)
- `config/` (with a simple default runtime config with that previous module)
- `Cyberfabric.toml` (manifest to run the previous module with the config)
- `.cyberfabric/` ignored by Git in .gitignore
- optional `AGENTS.md`, `CLAUDE.md`
- optional `SKILL.md` in `.agents/skills/cyberfabric` or `.claude/skills/cyberfabric`
- optional Docker/Helm build templates in `deploy/`

Recommended flags:

```text
--profile minimal|service|platform
--with docker,helm,agents,skill
```

## Module Scaffolding

Command:

```text
cargo cyberfabric generate module --template background-worker --name jobs
```

Initial module template set:

- `background-worker`
- `api-db-handler`
- `rest-gateway`
- `grpc-service`
- `oop-module`

Module generation should:

- create the module crate
- add it to workspace members
- normalize dependencies to workspace dependencies
- inherit workspace lints
- optionally add the module to a manifest app
- optionally add default module runtime config

Recommended flags:

```text
--add-to-manifest env.<app>.dev,env.<app>.prod
--add-to-config config/dev-app.yml,config/prod-app.yml
```

## Configuration Scaffolding

Command:

```text
cargo cyberfabric generate config --env dev --app app1 --name dev-app2
```

Config templates:

- `prod`: hardened config for production
- `dev`: easy to use config for development
- `db`: configuration with different database types

Output examples:

```text
config/dev-app1.yml
config/fragments/database.yml
config/modules/<module>.yml
```

The generated config should not contain dependency metadata. It should only
contain runtime values.

## Manifest Scaffolding

Command:

```text
cargo cyberfabric generate manifest
```

Should create `Cyberfabric.toml` and optionally infer:

- known configs in `config/`
- local modules from workspace metadata
- default `dev` environment
- one app from the workspace/package name

Recommended flags:

```text
--from-config <path>
--env <env>
--app <app>
--include-local-modules
```

## Build Templates

Command:

```text
cargo cyberfabric generate build docker
cargo cyberfabric generate build compose
```

Build template kinds:

- `docker`: Dockerfile and `.dockerignore`
- `compose`: local development dependencies
- `ci-github`: GitHub Actions workflow

## Agent and Skill Templates

Commands:

```text
cargo cyberfabric generate agents
cargo cyberfabric generate skill
```

`AGENTS.md` should include:

- project commands
- formatting/lint/test expectations
- module conventions
- manifest/config split
- known generated paths

`SKILL.md` should include:

- CLI command reference for LLMs
- common workflows
- schema snippets
- examples that prefer manifest-first commands
- warning that generated `.cyberfabric/` is derived output

These templates are useful for humans, Codex-like coding agents, and CI bots.

## CI Template Scaffolding

CI workflow templates are separate from build artifact templates because they serve a different purpose: configuring
continuous integration pipelines rather than producing deployable artifacts.

```bash
cargo cyberfabric generate ci <provider> [-p <path>]
```
