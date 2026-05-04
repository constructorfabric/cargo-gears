# 02. Architecture

## Table of Contents

1. [Overview](#overview)
2. [Crate Layout](#crate-layout)
3. [Command Dispatch](#command-dispatch)
4. [Core Abstractions](#core-abstractions)
5. [Manifest Pipeline](#manifest-pipeline)
6. [Template Engine Integration](#template-engine-integration)
7. [Tool Orchestration](#tool-orchestration)
8. [Error Model](#error-model)
9. [Output Model](#output-model)
10. [Extension Points](#extension-points)

## Overview

The CLI is structured as a Cargo workspace with focused crates. The architecture separates command parsing, domain
logic, and external tool orchestration so that each layer can be tested independently and extended without coupling.

## Crate Layout

Current workspace members:

```text
crates/
├── cli/                  # Binary crate: clap parsing, command dispatch, CLI entry point
└── module-parser/        # Library crate: Rust source parsing for module discovery
```

Proposed evolution:

```text
crates/
├── cli/                  # Binary: clap parsing, command dispatch, output formatting
├── core/                 # Library: manifest model, config model, validation, resolution
├── codegen/              # Library: server project generation (Cargo.toml, main.rs)
├── templates/            # Library: template registry, cargo-generate orchestration
├── module-parser/        # Library: Rust source parsing for module discovery (existing)
└── tools/                # Library: external tool orchestration (cargo, docker, helm)
```

### Rationale

- **`core`** owns the manifest and config domain types, validation logic, and module resolution. Extracting this from
  `cli` enables unit testing without clap and enables reuse from a potential library API.
- **`codegen`** owns the generation of `.cyberfabric/<name>/` projects. Isolating generation makes it testable with
  snapshot assertions and decouples it from command parsing.
- **`templates`** owns the template registry and `cargo-generate` integration. Separating this from `cli` allows
  template logic to evolve independently.
- **`tools`** owns subprocess invocation for `cargo`, `docker`, `helm`, `rustup`. This is where retry logic,
  environment setup, and tool version detection live.

Migration path: Extract crates incrementally behind internal `pub(crate)` boundaries first, then promote to workspace
members when the API stabilizes.

## Command Dispatch

The CLI uses `clap` derive macros for parsing. The dispatch pattern remains the current match-based approach:

```rust
impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Init(args) => args.run(),
            Commands::Generate(args) => args.run(),
            Commands::Manifest(args) => args.run(),
            // ...
        }
    }
}
```

Each command module owns its `Args` struct and `run()` implementation. Commands that share behavior (e.g., `run` and
`build` both generate server projects) call shared functions from `core` or `codegen` rather than duplicating logic.

### Shared Argument Patterns

Common flag groups are extracted into reusable clap structs:

```rust
/// Flags shared by commands that operate on a workspace.
#[derive(clap::Args)]
pub struct WorkspaceArgs {
    /// Workspace root directory. Defaults to current directory.
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

/// Flags shared by commands that require a manifest.
#[derive(clap::Args)]
pub struct ManifestArgs {
    /// Path to Cyberfabric.toml. Discovered automatically if omitted.
    #[arg(long)]
    pub manifest: Option<PathBuf>,

    /// Target environment.
    #[arg(long)]
    pub env: Option<String>,

    /// Target app within the environment.
    #[arg(long)]
    pub app: Option<String>,
}

/// Flags shared by commands that support structured output.
#[derive(clap::Args)]
pub struct OutputArgs {
    /// Output format.
    #[arg(long, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Toml,
}
```

## Core Abstractions

### Manifest Model

The manifest is a strongly typed Rust struct deserialized from `Cyberfabric.toml`:

```rust
pub struct Manifest {
    pub schema_version: SchemaVersion,
    pub workspace: WorkspaceConfig,
    pub templates: Option<TemplateRegistry>,
    pub environments: BTreeMap<String, BTreeMap<String, AppConfig>>,
}

pub struct AppConfig {
    pub config: PathBuf,
    pub modules: Vec<ModuleRef>,
    pub run: Option<RunPolicy>,
    pub build: Option<BuildPolicy>,
    pub lint: Option<LintPolicy>,
    pub test: Option<TestPolicy>,
}
```

`BTreeMap` is used instead of `HashMap` for deterministic iteration order.

### Module Resolution

Module resolution follows a defined precedence:

1. Manifest module reference (name, source, version, package).
2. Local workspace discovery via `cargo_metadata` and `module-parser`.
3. Remote registry lookup (crates.io).

Resolution produces a `ResolvedModule` that contains everything needed for code generation:

```rust
pub struct ResolvedModule {
    pub name: String,
    pub package: String,
    pub version: semver::Version,
    pub source: ModuleSource,
    pub features: Vec<String>,
    pub default_features: bool,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<Capability>,
}
```

### Generation Model

The generation model is the fully resolved input to code generation. It is the output of `manifest render` and the
input to `codegen`:

```rust
pub struct GenerationModel {
    pub environment: String,
    pub app: String,
    pub config_path: PathBuf,
    pub generated_project: PathBuf,
    pub modules: Vec<ResolvedModule>,
    pub features: Vec<String>,
    pub profile: Profile,
}
```

This struct is serializable to JSON for `manifest render --format json` and for debugging.

## Manifest Pipeline

Every command that builds, runs, or deploys follows this pipeline:

```text
1. Discover manifest (Cyberfabric.toml or --manifest)
2. Select environment and app (--env, --app, or defaults)
3. Validate manifest structure and references
4. Resolve modules (local discovery + registry lookup)
5. Build GenerationModel
6. Generate server project (.cyberfabric/<env>-<app>/)
7. Execute operation (cargo build, cargo run, docker build, etc.)
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
    Docker,
    Helm,
    Rustup,
}
```

The runner handles:

- **Tool detection**: verifies the tool is installed and reports the version.
- **Argument construction**: builds the complete argument list from CLI flags and manifest policy.
- **Environment setup**: sets `CF_CLI_CONFIG`, `RUSTFLAGS`, and other required variables.
- **Output capture**: captures stdout/stderr for structured reporting.
- **Error wrapping**: wraps tool failures with CLI context (which command, which arguments, which manifest app).

## Error Model

All errors use a structured enum hierarchy:

```rust
pub enum CliError {
    Manifest(ManifestError),
    Config(ConfigError),
    Resolution(ResolutionError),
    Generation(GenerationError),
    Tool(ToolError),
    Validation(Vec<Diagnostic>),
    Io(std::io::Error),
}

pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}
```

See [09-developer-experience.md](./09-developer-experience.md) for the full error handling design.

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

- **Template registry**: custom templates via `--local-path`, manifest-defined sources, or the default Git repository.
- **Lint rules**: custom Dylint rules compiled into the CLI binary.
- **Test sets**: manifest-defined named test sets with custom arguments.
- **Build outputs**: the `--output` enum is extensible with new variants.
- **Module sources**: the `ModuleSource` enum can be extended for private registries.

A formal plugin system (e.g., WASM-based or subprocess-based) is a future consideration. The current extension model
is sufficient for the CyberFabric team's needs and avoids the complexity of a plugin API before usage patterns are
established.
