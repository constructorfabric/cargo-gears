# 04. Manifest and Configuration

## Table of Contents

1. [Purpose](#purpose)
2. [Proposed Manifest](#proposed-manifest)
3. [Module References](#module-references)
4. [Commands](#commands)
5. [Validation](#validation)
6. [Generated Server Flow](#generated-server-flow)

## Purpose

The manifest detaches the **orchestration layer** from the **configuration layer**. Today the runtime config contains
module dependency metadata that the CLI uses to shape generated `Cargo.toml` and `src/main.rs`. The manifest makes this
explicit:

- **Manifest** (`Cyberfabric.toml`): what the CLI builds, how it builds it, and which policies apply.
- **Runtime config** (`config/*.yml`): what the generated server reads at runtime.

This separation eliminates metadata duplication, reduces config-file complexity, and makes the CLI's behavior
inspectable without reading YAML runtime values.

The manifest will be required for the `run`, `test`, `lint` and `build` commands. These commands require the 
orchestration information that the manifest provides.

For the rest of proposed commands, the manifest will not be required, as `generate`, `help`, `list`, `config`, `tools`

## Proposed Manifest

Default file name:

```text
Cyberfabric.toml
```

TOML is recommended because the current ecosystem in rust for handling `toml` with `toml_edit` crate allows us to
do in-place edits from the CLI without breaking comments.

Other formats were considered like YAML or JSON. JSON doesn't allow comments, which would be a nice feature to have
in the manifest. YAML is very close to TOML, but during the investigation phase, there was no crate that allowed
in-place edits without breaking comments. This feature is required as we are going to allow to do certain changes
in the manifest from the CLI.

Find the schema to be used in [manifest.rs](./manifest.rs).

Example:

```toml
[workspace]
version = 1
root = "." # default
config-dir = "config" # default
generated-dir = ".cyberfabric" # default

[env.app1.dev]
config = "app1-dev.yml" # relative from config_dir
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
    watch = { enabled = true }, # this is the default
    fips = false,
    otel = true
}
build = {
    name = "app1", # override env name
    profile = "debug"
}

[env.app1.lint]
dylint = {
   enabled = true,
   skip = ["rule-name"],
}
clippy = true # by default
fmt = true # by default
feature-set-test = true # inherits the feature set to test


[env.app1.test.default]
runner = "nextest"
config = "app1-test.yml"
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
config = "app1-prod.yml"
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
    watch = { enabled = false },
    fips = true,
    otel = true
}
build = {
    name = "app1", # override env name
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

Recommendation: Use always the `package` name to be precise.

## Commands

```text
cargo cyberfabric manifest [--manifest Cyberfabric.toml] <CMD>
cargo cyberfabric manifest validate 
cargo cyberfabric manifest ls [flags]
cargo cyberfabric manifest add <app> <env> [<module-ref1> <module-ref2> ...]
cargo cyberfabric manifest edit <app> <env> [flags]
cargo cyberfabric manifest rm <app> <env> <module-ref>
cargo cyberfabric manifest render <app> <env>
```

`manifest render` should produce the resolved generation model:

- selected runtime config path
- module dependency metadata
- generated Cargo dependencies
- generated features
- generated `.cyberfabric` inputs

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
- FIPS is requested with modules or features that cannot support it

Validation output should be deterministic and machine-readable with `--format json`.

## Generated Server Flow

New flow:

1. Load manifest.
2. Select `app` and `env`.
3. Load runtime config path declared for that app, unless overridden by
   `--config`.
4. Resolve modules from manifest.
5. Merge discovered metadata with manifest constraints.
6. Generate `.cyberfabric/<app>-<env>` or configured generated name.
7. Provide the relative path in the first argument to the config file.
8. Run the selected operation.
