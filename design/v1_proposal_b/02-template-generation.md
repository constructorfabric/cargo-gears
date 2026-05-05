# 02. Template Generation

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
- Docker support exists in `deploy` through an embedded Dockerfile.

## Proposed Commands

Keep existing commands and add a normalized generation namespace:

```text
cargo cyberfabric generate workspace <path>
cargo cyberfabric generate module <template> [--name <name>]
cargo cyberfabric generate config <kind>
cargo cyberfabric generate manifest
cargo cyberfabric generate build <kind>
cargo cyberfabric generate agents
cargo cyberfabric generate skill
```

Aliases:

- `init` will remain as an alias for `generate workspace`.

## Template Registry

We'll provide a set of templates to be used in the generation commands.
However, the developer can add their own templates by:

1. `--local-path`
2. manifest-defined template registry
3. default Git template repo
4. embedded fallback templates for small files

Catalog shape:

```toml
[[templates.module]]
name = "background-worker"
source = "git"
subfolder = "Modules/background-worker"

[[templates.build]]
name = "docker"
source = "embedded"

[[templates.agent]]
name = "custom-agent"
source = "git"
url = "ssh://github.com/cyberfabric/cf-template-rust.git"
branch = "main"
subfolder = "Agents/custom-agent"
```

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
cargo cyberfabric generate build helm
```

Build template kinds:

- `docker`: Dockerfile and `.dockerignore`
- `compose`: local development dependencies
- `helm`: chart skeleton
- `ci-github`: GitHub Actions workflow

Docker should align with current `deploy` behavior, but generation should make
the Dockerfile inspectable before the build.

Helm should be a packaging template, not a deployment command in the first
iteration. The first version should generate values for:

- image repository/tag
- config file mount
- environment variables
- service ports
- probes
- resource requests/limits

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

