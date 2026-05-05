# 01. Manifest and Configuration

## Table of Contents

1. [Purpose](#purpose)
2. [Proposed Manifest](#proposed-manifest)
3. [Module References](#module-references)
4. [Commands](#commands)
5. [Validation](#validation)
6. [Generated Server Flow](#generated-server-flow)

## Purpose

The manifest should detach the orchestration layer from the configuration layer.
Today the configuration contains enough module metadata for the CLI to shape the generated
`Cargo.toml` and `src/main.rs`. The proposed manifest makes that behavior explicit.

## Proposed Manifest

Default file name:

```text
Cyberfabric.toml
```

TOML is recommended because the current ecosystem in rust for handling `toml` with `toml_edit` crate allows us to
do in-place edits from the CLI without breaking comments.

Example:

```toml
[workspace]
root = "." # default
config-dir = "config" # default
generated-dir = ".cyberfabric" # default

[env.app1.dev]
config = "dev-app1.yml" # relative from config_dir
modules = [
    {
        name = "module1",
        source = "local", # local | remote | registry
        version = "1.2.0", # required for remote and registry
        package = "crate1",
    },
    {
        name = "module2",
        source = "remote",
        version = "0.1.0",
        package = "crate2",
    }
]
run = {
    watch = true,
    fips = false,
    otel = true
}
build = {
    name = "app1", # override env name
    outputs = ["binary", "docker"],
    image = "registry.example.com/app1",
    profile = "debug"
}

[env.app1.lint]
skip-dylint = [
    "rule-name"
]
# skip-dylint = true
clippy = true # by default
fmt = true # by default
feature-set-test = true # inherits the feature set to test


[env.app1.test]
runner = "nextest"
config = "test-app1.yml"
coverage = true
feature-set = {
    "module1" = [
        ["unit", "integration"],
        ["sqlite"],
        ["postgres"],
        ["fips"],
        false # disable all features
    ],
    "module2" = true, # enable all features
}

[env.app1.prod]
config = "prod-app1.yml"
modules = [
    {
        name = "module1",
        source = "local", # local | remote | registry
        version = "1.2.0", # required for remote and registry
        crate = "crate1",
    },
    {
        name = "module2",
        source = "remote",
        version = "0.1.0",
        crate = "crate2",
    }
]
run = {
    watch = false,
    fips = true,
    otel = true
}
build = {
    name = "app1", # override env name
    docker = {
        image = "registry.example.com/app1", # required for docker output
    },
    profile = "release"
}
```

The TOML structure makes environment and app explicit while leaving room for
test, lint, run, and build policy.

## Module References

Initial enum:

- `local`: discovered from current Cargo workspace metadata and `src/module.rs`.
- `remote`: resolved from the configured registry, defaulting to `crates.io`.
- `registry`: explicit registry-qualified remote module.

For the local, we can treat it as module name first, then package
name and library name as a fallback, and print the resolved package name in verbose output.

Recommendation: Use always the `crate` name to be precise.

## Commands

```text
cargo cyberfabric manifest [--manifest Cyberfabric.toml] <CMD>
cargo cyberfabric manifest validate 
cargo cyberfabric manifest ls [flags]
cargo cyberfabric manifest add <env> <app> [<module-ref1> <module-ref2> ...]
cargo cyberfabric manifest edit <env> <app> [flags]
cargo cyberfabric manifest rm <env> <app> <module-ref>
cargo cyberfabric manifest render <env> <app>
```

`manifest render` should produce the resolved generation model:

- selected runtime config path
- module dependency metadata
- generated Cargo dependencies
- generated features
- generated `main.rs` inputs
- selected lint/test/run/build policy

This command is useful for debugging and for LLMs because it exposes the exact
input to generation without executing Cargo.

## Validation

Validation should fail early when:

- an environment/app does not exist
- a module reference has an unknown kind
- a local module cannot be discovered
- a remote module cannot be resolved
- multiple module refs resolve to the same package name
- module dependency constraints conflict
- a run/build/test profile references unsupported modes
- production uses `watch = true`
- FIPS is requested with modules or features that cannot support it
- Docker/Helm output is selected without required metadata

Validation output should be deterministic and machine-readable with `--format json`.

## Generated Server Flow

New flow:

1. Load manifest.
2. Select `env` and `app`.
3. Load runtime config path declared for that app, unless overridden by
   `--config`.
4. Resolve modules from manifest.
5. Merge discovered metadata with manifest constraints.
6. Generate `.cyberfabric/<env>-<app>` or configured generated name.
7. Set `CF_CLI_CONFIG` to the runtime config.
8. Run the selected operation.

The generated `main.rs` can remain close to the current implementation: it
should still load `CF_CLI_CONFIG` at runtime and call `run_server(config)`.
