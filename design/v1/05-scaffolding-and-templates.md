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
9. [Agent and Skill Templates](#agent-and-skill-templates)

## Purpose

Template generation should become a first-class CLI surface instead of being
limited to `init` and `mod add`. The goal is to let users scaffold a full
Cyberware workspace, individual modules, runtime config, CLI manifest, build
artifacts, and automation guidance files.

### Current Behavior

Current generation behavior:

- `init` uses `cargo-generate` against `cf-template-rust`, subfolder `Init`.
- `mod add` uses `cargo-generate` against subfolder `Modules/<template>`.
- Available module templates are `background-worker`, `api-db-handler`, and `rest-gateway`.

## Proposed Commands

We replace the current `init` and `mod add` commands with a more structured one:

```text
cargo cyberware generate workspace <path> [flags]
cargo cyberware generate module --template <template> [--name <name>] [flags]
cargo cyberware generate config <kind> [flags]
cargo cyberware generate manifest [flags]
cargo cyberware generate build <kind> [flags]
cargo cyberware generate agents [flags]
cargo cyberware generate skill [flags]
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
[templates.module.background-worker]
source = "git"
url = "ssh://github.com/username/repository.git"
rev = "eb5a0f5d"

[templates.config.grpc-service]
source = "local"
path = "templates/grpc-service"

[templates.agents.custom-agent]
source = "git"
url = "ssh://github.com/cyberware/cf-template-rust.git"
rev = "f1e6ad66"
subfolder = "Agents/custom-agent"
```

This catalog is optional and can be placed in `Cyberware-templates.toml` or by its own section in the manifest.

## Initial Scaffolding

Command:

```text
cargo cyberware generate workspace ./my-app [--name my-app]
cargo cyberware init ./my-app [--name my-app]
```

Should create:

- `Cargo.toml` workspace
- `modules/` (with a simple module)
- `config/` (with a simple default runtime config with that previous module)
- `Cyberware.toml` (manifest to run the previous module with the config)
- `.cyberware/` ignored by Git in .gitignore
- optional `AGENTS.md`, `CLAUDE.md`
- optional `SKILL.md` in `.agents/skills/cyberware` or `.claude/skills/cyberware`
- optional Docker build templates in `deploy/`

Recommended flags:

```text
--profile minimal|service|platform
--with docker,agents,skill
```

## Module Scaffolding

Command:

```text
cargo cyberware generate module --template background-worker --name jobs
```

Initial module template set:

- `background-worker`
- `api-db-handler`
- `rest-gateway`
- `grpc-service`
- `oop-module`

Module generation should:

- retrieve from the template(through git, local path, or embedded from the CLI)
- put in @/modules/<name> the module selected
- replace the placeholders with the expected values (this is done through cargo generate)
- add the module and its sdk, if any, to workspace members
- normalize dependencies to workspace dependencies
- inherit workspace lints
- optionally add the module to a manifest app
- optionally add default module runtime config

Recommended flags:

```text
--add-to-manifest apps.<app>.dev,apps.<app>.prod
--add-to-config config/app-dev.yml,config/app-prod.yml
```

## Configuration Scaffolding

Command:

```text
cargo cyberware generate config --app app1 --env dev --name custom_name
```

Config templates:

- `prod`: hardened config for production
- `dev`: easy to use config for development
- `db`: configuration with different database types

Output examples:

```text
config/app1-dev.yml
config/fragments/database.yml
config/modules/<module>.yml
```

The generated config should not contain dependency metadata. It should only
contain runtime values.

## Manifest Scaffolding

Command:

```text
cargo cyberware generate manifest
```

Should create `Cyberware.toml` and optionally infer:

- generate a default config in `config/` based on the app/environment pair
- local modules from workspace metadata
- default `dev` environment
- one app from the workspace/package name

Recommended flags:

```text
--env <env>
--app <app>
--include-local-modules
```

## Agent and Skill Templates

Commands:

```text
cargo cyberware generate ai --agents
cargo cyberware generate ai --skill --provider <provider>
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
- warning that generated `.cyberware/` is derived output

These templates are useful for humans, Codex-like coding agents, and CI bots.
