# 02. Architecture

## Table of Contents

1. [Overview](#overview)
2. [Crate Layout](#crate-layout)
3. [Manifest Pipeline](#manifest-pipeline)
4. [Template Engine Integration](#template-engine-integration)
5. [Tool Orchestration](#tool-orchestration)
6. [Output Model](#output-model)
7. [Extension Points](#extension-points)

## Overview

The CLI is structured as a Cargo workspace with focused crates. The architecture separates command parsing, domain
logic, and external tool orchestration so that each layer can be tested independently and extended without coupling.

## Crate Layout

```text
crates/
├── cli/                  # Binary: clap parsing, command dispatch, output formatting
├── core/                 # Library: manifest model, config model, validation, resolution
├── codegen/              # Library: server project generation (Cargo.toml, main.rs)
├── templates/            # Library: template registry, cargo-generate orchestration
├── module-parser/        # Library: Rust source parsing for module discovery (existing)
└── tools/                # Library: external tool orchestration (cargo helpers, system dependencies, etc)
```

### Rationale

- **`core`** owns the manifest and config domain types, validation logic, and module resolution. Extracting this from
  `cli` enables unit testing without clap and enables reuse from a potential library API.
- **`codegen`** owns the generation of `.cyberfabric/<name>/` projects. Isolating generation makes it testable with
  snapshot assertions and decouples it from command parsing.
- **`templates`** owns the template registry and `cargo-generate` integration. Separating this from `cli` allows
  template logic to evolve independently.
- **`tools`** owns subprocess invocation for `cargo ...`, `rustup`. This is where retry logic,
  environment setup, and tool version detection live.

Migration path: Extract crates incrementally behind internal `pub(crate)` boundaries first, then promote to workspace
members when the API stabilizes.

## Manifest Pipeline

Every command that builds or runs follows this pipeline:

```text
1. Discover manifest (Cyberfabric.toml or --manifest)
2. Select environment and app (--env, --app, or defaults)
3. Validate manifest structure and references
4. Resolve modules (local discovery + registry lookup)
5. Generate server project (.cyberfabric/<app>-<env>/)
6. Execute operation (cargo build, cargo run, etc.)
```

Commands that only inspect (e.g., `list`, `manifest validate`) stop at step 3-5. Commands that mutate (e.g.,
`manifest add`) operate on the manifest file directly.

The pipeline is implemented as composable functions, not a monolithic run method. Each step is independently testable.

## Template Engine Integration

Template generation uses `cargo-generate` as the engine. The CLI controls:

- Template source resolution (Git URL, local path, embedded fallback).
- Template variables (project name, module name, features).
- Post-generation hooks (workspace member wiring, dependency promotion, lint inheritance).

The template registry is defined in the manifest or falls back to the default Git template repository. See
[05-scaffolding-and-templates.md](./05-scaffolding-and-templates.md) for the registry schema.

## Tool Orchestration

External tools are invoked through a `ToolRunner` abstraction:

```rust
pub struct ToolRunner {
    pub tool: Tool,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
}

pub enum Tool {
    Cargo,
    CargoGenerate,
    CargoNextest,
    CargoLLVMCoverage,
    CargoDeny,
    Rustup,
}
```

The runner handles:

- **Tool detection**: verifies the tool is installed and reports the version.
- **Argument construction**: builds the complete argument list from CLI flags and manifest policy.
- **Environment setup**: sets `CF_CLI_CONFIG`, `RUSTFLAGS`, and other required variables.

## Output Model

All commands that produce output use a shared output pipeline:

```text
Command logic -> domain result -> OutputFormatter -> stdout
```

The `OutputFormatter` selects the rendering strategy based on `--format`:

- **Table**: human-readable columnar output using a lightweight table formatter.
- **JSON**: `serde_json::to_string_pretty` with stable field ordering.
- **YAML**: `serde_yaml` serialization.
- **TOML**: `toml::to_string_pretty` serialization.

Domain result types implement `Serialize` so that all format modes share the same data source.

## Extension Points

The architecture provides these extension points without requiring a plugin system in the first iteration:

- **Template registry**: manifest-defined sources, or the default Git repository.
- **Lint rules**: custom Dylint rules compiled into the CLI binary.
- **Test sets**: manifest-defined named test sets with custom arguments.
- **Build outputs**: the `--output` enum is extensible with new variants.
- **Module sources**: the `ModuleSource` enum can be extended for private registries.

A formal plugin system (e.g., WASM-based or subprocess-based) is a future consideration. The current extension model
is sufficient for the CyberFabric team's needs and avoids the complexity of a plugin API before usage patterns are
established.
