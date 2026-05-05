# 03. List and Inspection

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Commands](#proposed-commands)
4. [System Modules](#system-modules)
5. [Local Modules](#local-modules)
6. [Configuration Overview](#configuration-overview)
7. [Output Formats](#output-formats)

## Purpose

The CLI should let users quickly inspect what modules exist, what modules are
enabled, and how manifest apps relate to runtime configuration.

## Current Behavior

`config mod list` currently prints:

- system modules when `--system` is passed
- workspace modules from Cargo metadata and `src/module.rs`
- modules enabled in a specific config
- verbose metadata when `--verbose` is passed

This is valuable but config-centric. The manifest-first design needs a
workspace/app overview as well.

## Proposed Commands

```text
cargo cyberfabric list modules
cargo cyberfabric list system-modules
cargo cyberfabric list local-modules
cargo cyberfabric list configs
cargo cyberfabric list apps
```

## System Modules

Command:

```text
cargo cyberfabric list system-modules [--verbose] [--registry crates.io]
```

Output should include:

- module name
- crate name
- latest version
- capabilities
- dependencies
- available features
- whether the module is already used by any manifest app

Initial system module registry can continue to use the current static list:

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

## Local Modules

Command:

```text
cargo cyberfabric list local-modules [--verbose]
```

Output should include:

- module name
- package name
- version
- path
- capabilities
- declared module deps
- apps/environments that reference it
- config files that still contain legacy metadata for it

This can reuse existing `get_module_name_from_crate()` discovery.

## Configuration Overview

Commands:

```text
cargo cyberfabric list configs
cargo cyberfabric list apps
cargo cyberfabric list modules --env dev --app app1
```

`list configs` should show:

- config path
- inferred environment/app when linked from manifest
- runtime sections present
- modules with runtime config
- modules with legacy metadata

`list apps` should show:

- environment
- app
- config path
- module count
- run mode summary
- test policy summary
- build outputs

Example:

```text
Environment  App   Config              Modules  Run             Build
dev          app1  config/dev-app1.yml 3        watch,otel      binary
prod         app1  config/prod-app1.yml 2       fips,otel       binary,docker
```

## Output Formats

All list commands should support:

```text
--format table|json|yaml|toml
```

Default is `table` for humans. `json` is the stable contract for tools and LLMs.

JSON output should use explicit fields rather than human labels:

```json
{
  "environment": "dev",
  "app": "app1",
  "config": "config/dev-app1.yml",
  "modules": [
    {
      "reference": "local:crate1",
      "module_name": "crate1",
      "package": "crate1",
      "source": "local",
      "capabilities": [
        "db"
      ]
    }
  ]
}
```

