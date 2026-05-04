# Glossary

Precise definitions for all domain terms used in the CyberFabric CLI design documents.

## A

### App

A named application configuration within an environment. An app specifies a module set, runtime config, and policies
for run, build, lint, and test. Defined in the manifest as `[env.<environment>.<app>]`.

## C

### Capability

A functional trait that a module exposes (e.g., `db`, `grpc`, `http`). Capabilities are discovered from module
metadata and used for validation (e.g., FIPS compatibility checks).

### Config (Runtime Config)

A YAML file containing runtime values consumed by the generated server and modules. Located in the workspace's
`config/` directory. Contains server settings, database connections, module runtime parameters, and feature
configuration. Does **not** contain module dependency metadata (that belongs in the manifest).

### Config-Centric Mode

The legacy operating mode where the CLI reads module dependency metadata from the runtime config file instead of a
manifest. Supported for backward compatibility via `--config`.

### CyberFabric

The Rust framework for building modular applications. The CLI is the canonical tooling interface for this framework.

### `Cyberfabric.toml`

See [Manifest](#manifest).

## D

### Diagnostic

A structured error, warning, or informational message produced by validation. Contains severity, code, message,
location, and suggestion fields. Serializable to JSON.

### Dry Run

A mode (`--dry-run`) that prints the resolved execution plan without performing side effects. No files are generated,
no subprocesses are invoked, no network calls are made.

## E

### Environment

A named deployment context (e.g., `dev`, `prod`, `staging`). Each environment contains one or more apps. Defined in
the manifest as `[env.<environment>]`.

### Exit Code

A numeric code returned by the CLI process to indicate success or failure type. Defined in
[09-developer-experience.md](./09-developer-experience.md#exit-codes).

## G

### Generated Server Project

The Cargo project created under `.cyberfabric/<name>/` by the CLI. Contains a generated `Cargo.toml`, `.cargo/config.toml`,
and `src/main.rs`. This is derived output that can be regenerated at any time. Listed in `.gitignore`.

### Generation Model

The fully resolved input to code generation. Contains the selected environment, app, config path, module list with
resolved versions, features, and profile. Produced by `manifest render` and consumed by the code generator.

## L

### Local Module

A CyberFabric module that exists as a crate in the workspace's `modules/` directory. Discovered via `cargo_metadata`
and `src/module.rs` parsing.

## M

### Manifest

The `Cyberfabric.toml` file at the workspace root. The single source of truth for what the CLI generates and
orchestrates: apps, environments, modules, features, and policies. See
[04-manifest-and-configuration.md](./04-manifest-and-configuration.md).

### Manifest-First Mode

The recommended operating mode where the CLI reads module composition, policies, and config paths from the manifest.
Activated by the presence of `Cyberfabric.toml` or explicit `--env`/`--app` flags.

### Modkit

The CyberFabric library set (`cf-modkit`, `cf-modkit-macros`, etc.) that provides the module development API. The
CLI generates code that uses Modkit types.

### Module

A self-contained CyberFabric component that provides specific functionality within an application. Modules are Rust
crates that implement the CyberFabric module interface (`src/module.rs`). They can be local (workspace) or remote
(registry).

### Module Reference

A manifest entry that identifies a module by name, source, and optionally version and package. Resolved to a
`ResolvedModule` during the generation pipeline.

## P

### Policy

Configuration in the manifest that defines how a command behaves for a specific app. Policies exist for `run`, `build`,
`lint`, and `test`. CLI flags override policy settings.

### Profile

A build profile that determines optimization level and debug information. Values: `debug`, `release`. Maps to Cargo's
`--release` flag.

## R

### Registry Module

A CyberFabric module published to a package registry (default: crates.io). Requires explicit `version` and `package`
in the manifest.

### Resolution

The process of converting a module reference (name, source) into a fully resolved module (package, version, features,
capabilities, path). Resolution tries local workspace discovery first, then registry lookup.

## S

### Schema Version

A positive integer in `Cyberfabric.toml` that identifies the manifest schema version. The CLI refuses to load
manifests with a schema version higher than it supports. See
[12-versioning-and-compatibility.md](./12-versioning-and-compatibility.md#manifest-schema-versioning).

### System Module

A CyberFabric module maintained by the framework team and published to crates.io. The CLI maintains a built-in
registry of known system modules. Examples: `credstore`, `api-gateway`, `authn-resolver`.

## T

### Template

A scaffold source used by `generate` commands. Templates can come from Git repositories, local directories, or
embedded resources in the CLI binary. See [05-scaffolding-and-templates.md](./05-scaffolding-and-templates.md).

### Template Registry

The configuration in the manifest (`[[templates.*]]`) that defines available templates and their sources. Overrides
the default Git repository.

## W

### Workspace

A Cargo workspace that contains CyberFabric modules, runtime configs, the manifest, and generated server projects.
Created by `init` or `generate workspace`. The workspace root contains `Cargo.toml`, `Cyberfabric.toml`, `modules/`,
`config/`, and `.cyberfabric/`.

### Watch Mode

A development mode (`--watch`) that monitors file changes and automatically restarts the generated server. Observes
module sources, runtime config, manifest, and workspace metadata.
