# 03. Command Surface

## Table of Contents

1. [Design Conventions](#design-conventions)
2. [Complete Command Tree](#complete-command-tree)
3. [Verb Taxonomy](#verb-taxonomy)
4. [Shared Argument Patterns](#shared-argument-patterns)
5. [Name Validation](#name-validation)
6. [Discoverability](#discoverability)

## Design Conventions

All commands follow these conventions:

- **Verb-first structure.** Top-level commands are verbs (`new`, `run`, `build`, `lint`, `test`) or
  noun-scoped managers (`config`, `manifest`, `list`, `help`).
- **Predictable flag names.** The same concept uses the same flag across all commands (e.g., `-p/--path` always means
  workspace root, `-c/--config` always means runtime config file).
- **No positional ambiguity.** When a command accepts both a positional argument and flags, the positional argument is
  always the primary subject (module name, path, query). Flags modify behavior.
- **Enum over string.** When the set of valid values is known at compile time, use a `clap::ValueEnum` rather than an
  open string. This enables tab-completion, validation, and documentation.
- **Short flags for frequent use.** Only the most commonly used flags get single-letter aliases. Infrequent flags use
  long-form only to keep the short namespace clean.

## Complete Command Tree

```text
cargo cyberfabric
│
├── new <path>                                        # Initialize workspace (alias: generate workspace)
│
├── generate
│   ├── workspace <path>                              # Full workspace scaffolding
│   ├── module --template <template> [--name <name>]  # Module scaffolding
│   ├── config <kind>                                 # Runtime config scaffolding
│   ├── manifest                                      # Cyberfabric.toml scaffolding
│   ├── build <kind>                                  # Build artifact templates (docker, compose)
│   └── ai [--skill --provider <p> --agents]          # SKILL.md generation
│
├── manifest
│   ├── validate                                    # Validate manifest structure and references
│   ├── render [--app <app>] [--env <env>]          # Render resolved generation model
│   ├── add <app> <env> [<module-ref>...]           # Add environment/app/modules
│   ├── edit <app> <env> [[--set config.ref]]       # Edit app settings
│   ├── rm <app> <env> [<module-ref>]               # Remove environment/app/module
│   └── migrate [--from <v>] [--to <v>]             # Migrate manifest schema version
│
├── list
│   ├── modules [--app <app>] [--env <env>]         # All modules (local + system + configured)
│   ├── local-modules                               # Workspace-discovered modules
│   ├── system-modules                              # Built-in system module registry
│   ├── configs                                     # Runtime config files
│   └── apps                                        # Manifest-defined apps and environments
│
├── help
│   ├── schema <manifest|config|module>             # Schema reference
│   ├── docs <rust-path>                            # Rust source resolution 
│   └── topic <topic>                               # Operational topic docs
│
├── config
│   ├── mod
│   │   ├── list -c <config>                        # List modules in config
│   │   ├── add <module> -c <config>                # Add module to config
│   │   ├── rm <module> -c <config>                 # Remove module from config
│   │   └── db
│   │       ├── add <module> -c <config>            # Add module DB config
│   │       ├── edit <module> -c <config>           # Edit module DB config
│   │       └── rm <module> -c <config>             # Remove module DB config
│   └── db
│       ├── add <name> -c <config>                  # Add global DB server
│       ├── edit <name> -c <config>                 # Edit global DB server
│       └── rm <name> -c <config>                   # Remove global DB server
│
├── docs [<query>]                                  # LLM helper for efficient Rust source retrieval (alias: help docs)
├── lint                                            # Orchestrated linting
├── test                                            # Orchestrated testing
├── tools                                           # Tool bootstrap (rustup, fmt, clippy)
├── run                                             # Generate and run server
├── build                                           # Generate and build server
├── ci                                              # Alias (manifest verify + lint + test + build) 
└── completions --shell <shell>                     # Generate shell completions
```

## Verb Taxonomy

| Verb          | Meaning                                      | Side Effects                                                |
|---------------|----------------------------------------------|-------------------------------------------------------------|
| `new`         | Create a new workspace from scratch          | Creates directories and files                               |
| `generate`    | Scaffold a specific artifact from a template | Creates files, may modify workspace Cargo.toml              |
| `manifest`    | Inspect or mutate `Cyberfabric.toml`         | `validate`/`render` are read-only; `add`/`edit`/`rm` mutate |
| `list`        | Read-only inspection of workspace state      | None                                                        |
| `help`        | Read-only documentation and schema output    | None                                                        |
| `config`      | Mutate runtime config YAML                   | Modifies config file                                        |
| `lint`        | Run quality checks                           | None (read-only analysis)                                   |
| `test`        | Run test suites                              | May generate app for e2e; test execution                    |
| `run`         | Generate server and execute it               | Generates project, runs process                             |
| `build`       | Generate server and compile it               | Generates project, compiles binary                          |
| `tools`       | Install or upgrade toolchain components      | May install system components                               |
| `docs`        | Resolve and print Rust source                | May download/cache crate sources                            |
| `completions` | Generate shell completion scripts            | Writes to stdout or file                                    |

## Shared Argument Patterns

### Workspace Path: `-p, --path <PATH>`

Available on: `config`, `lint`, `test`, `run`, `build`, `generate module`, `list`, `docs`.

Sets the workspace root. When provided, the CLI changes the working directory before resolving any other paths. When
omitted, the current directory is used.

### Runtime Config: `-c, --config <PATH>`

Available on: `config mod`, `config db`, `run`, `build`.

Path to the YAML runtime config file. Required when operating without a manifest, or to override the manifest-declared
config. Resolved relative to the workspace root after `-p/--path` is applied.

### Manifest Selection: `--manifest <PATH>`, `--env <ENV>`, `--app <APP>`

Available on: `run`, `build`, `lint`, `test`, `manifest`, `list modules`, `list apps`.

`--manifest` overrides automatic `Cyberfabric.toml` discovery. `--env` and `--app` select the target within the
manifest. When the manifest has exactly one environment and one app, they are selected automatically.

### Dry Run: `--dry-run`

Available on: `run`, `build`, `generate`.

Prints the resolved plan (generation model, command invocation, file list) without executing side effects. Combined
with `--format json`, this is the stable contract for automation.

### Verbose: `-v, --verbose`

Available on: all commands.

Increases output detail. For subprocesses, shows their stdout/stderr in real time instead of capturing it.

### Install Missing Tools: `--install-missing-tools`

Available on: `test`, `lint`.

When a required tool (e.g., `cargo-nextest`, `cargo-llvm-cov`) is not installed, install it automatically instead of
failing with a suggestion. Not enabled by default to avoid surprising side effects.

## Name Validation

All user-provided names (module names, app names, environment names, DB server names) are validated against a kebab-case regex:

```text
^[a-z](?:-[a-z0-9]+)+$
```

Rules:

- Must start with a letter.
- May contain letters, digits, hyphens, and underscores.
- Must not be empty.
- Maximum length: 64 characters.
- Case-sensitive; lowercase-with-hyphens is the recommended convention.

Validation is enforced at the clap parsing layer using a custom `value_parser` so that invalid names never reach
command logic.

## Discoverability

### Help Text Structure

Every command and subcommand has:

- A one-line `about` description shown in parent help.
- A `long_about` paragraph shown in its own `--help`.
- Grouped arguments with section headers (clap `help_heading`).
- Examples in `after_help`.

### Shell Completions

The CLI generates shell completion scripts for `bash`, `zsh`, `fish`, and `powershell`:

```bash
cargo cyberfabric completions --shell zsh > _cyberfabric
```

Completion scripts use clap's `clap_complete` crate and include `ValueEnum` variants for all typed arguments.
