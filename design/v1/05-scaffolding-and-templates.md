# 05. Template Generation

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Commands](#proposed-commands)
4. [Template Registry](#template-registry)
5. [Template Structure and Organization](#template-structure-and-organization)
6. [Initial Scaffolding](#initial-scaffolding)
7. [Module Scaffolding](#module-scaffolding)
8. [Configuration Scaffolding](#configuration-scaffolding)
9. [Manifest Scaffolding](#manifest-scaffolding)
10. [Agent and Skill Templates](#agent-and-skill-templates)

## Purpose

Template generation should become a first-class CLI surface instead of being
limited to `init` and `mod add`. The goal is to let users scaffold a full
Gears workspace, individual modules, runtime config, CLI manifest, build
artifacts, and automation guidance files.

### Current Behavior

Current generation behavior:

- `init` uses `cargo-generate` against `cf-template-rust`, subfolder `Init`.
- `mod add` uses `cargo-generate` against subfolder `Modules/<template>`.
- Available module templates are `background-worker`, `api-db-handler`, and `api-gateway`.

## Proposed Commands

We replace the current `init` and `mod add` commands with a more structured one:

```text
cargo gears generate workspace <path> [flags]
cargo gears generate module --template <template> [--name <name>] [flags]
cargo gears generate config <kind> [flags]
cargo gears generate manifest [flags]
cargo gears generate build <kind> [flags]
cargo gears generate agents [flags]
cargo gears generate skill [flags]
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
revision = "eb5a0f5d"

[templates.config.grpc-service]
source = "local"
path = "templates/grpc-service"

[templates.agents.custom-agent]
source = "git"
url = "ssh://github.com/constructorfabric/cf-template-rust.git"
tag = "0.2"
subfolder = "Agents/custom-agent"
```

This catalog is optional and can be placed along `Gears.toml` or by its own section in the manifest.

## Template Structure and Organization

The target repository layout for the default templates will be organized by subfolders, one for each section:

```text
cf-template-rust/
|-- Workspace/
|   `-- <workspace-template>/
|-- Modules/
|   `-- <module-template>/
|-- Config/
|   `-- <config-template>/
|-- Manifest/
|   `-- default/
|-- Build/
|   `-- <build-template>/
|-- Agents/
|   `-- <agent-template>/
`-- Skills/
    `-- <skill-template>/
```

Each concrete template directory is a standalone `cargo-generate` template. It
should contain its own `cargo-generate.toml`, a `README.md` explaining generated
architecture and expected inputs, and only the files needed for that scaffold.
Family directories such as `Modules/` may also define `sub_templates` so the CLI
can discover the built-in names without hardcoding them.

Module templates should preserve the Gears module layout described in
[02_module_layout_and_sdk_pattern.md](https://github.com/constructorfabric/gears-rust/blob/main/docs/modkit_unified_system/02_module_layout_and_sdk_pattern.md).
The generated module lives under `modules/<name>/` and may contain both an
implementation crate and an SDK crate:

```text
modules/<name>/
|-- Cargo.toml                    # module implementation crate
|-- sdk/                          # public contract crate, if exposed
|   |-- Cargo.toml
|   `-- src/
|       |-- lib.rs
|       |-- client.rs
|       |-- models.rs
|       `-- errors.rs
`-- src/
    |-- lib.rs
    |-- module.rs                 # Gears registration and lifecycle
    |-- config.rs                 # typed runtime config
    |-- api/                      # HTTP/gRPC/interface adapters
    |-- domain/                   # business logic and ports
    `-- infra/                    # DB, IO, system adapters
```

The template code should keep layer boundaries explicit:

- `sdk/` contains transport-agnostic public contracts and is safe for other modules to depend on.
- `src/api/` contains wire-level adapters only and should translate into domain
  services instead of carrying business logic.
- `src/domain/` contains internal business logic, ports, and local client adapters.
- `src/infra/` contains concrete persistence, HTTP, system, and other adapter implementations.
- The Gears module annotation (`#[module(...)]` / `#[gears_toolkit::module(...)]`)
  can live in any `src/*.rs` file. It is the Gears wiring point: module
  attributes, lifecycle methods, capability registration, and `ClientHub`
  registration.

Generated names should use kebab-case for module folders and crate names. Rust
identifiers derived from those names should be converted by the generator rather
than being stored as duplicate template variants. When the generated module has
an SDK crate, both the module crate and the SDK crate must be added to workspace
members, and the implementation crate should depend on the SDK through a local
path dependency before dependency normalization runs.

Validation should happen at the template boundary, not only after templates are
generated by the CLI.

## Initial Scaffolding

Command:

```text
cargo gears generate workspace ./my-app [--name my-app]
cargo gears init ./my-app [--name my-app]
```

Should create:

- `Cargo.toml` workspace
- `modules/` (with a simple module)
- `config/` (with a simple default runtime config with that previous module)
- `Gears.toml` (manifest to run the previous module with the config)
- `.gears/` ignored by Git in .gitignore
- optional `AGENTS.md`, `CLAUDE.md`
- optional `SKILL.md` in `.agents/skills/gears` or `.claude/skills/gears`
- optional Docker build templates in `deploy/`

Recommended flags:

```text
--profile minimal|service|platform
--with docker,agents,skill
```

## Module Scaffolding

Command:

```text
cargo gears generate module --template background-worker --name jobs
```

Initial module template set:

- `background-worker`
- `api-db-handler`
- `api-gateway`
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
cargo gears generate config --app app1 --env dev --name custom_name
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
cargo gears generate manifest
```

Should create `Gears.toml` and optionally infer:

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
cargo gears generate ai --agents
cargo gears generate ai --skill --provider <provider>
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
- warning that generated `.gears/` is derived output

These templates are useful for humans, Codex-like coding agents, and CI bots.
