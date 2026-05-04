# 03. Command Surface

## Table of Contents

1. [Design Conventions](#design-conventions)
2. [Complete Command Tree](#complete-command-tree)
3. [Verb Taxonomy](#verb-taxonomy)
4. [Shared Argument Patterns](#shared-argument-patterns)
5. [Name Validation](#name-validation)
6. [Command Aliases and Migration](#command-aliases-and-migration)
7. [Discoverability](#discoverability)

## Design Conventions

All commands follow these conventions:

- **Verb-first structure.** Top-level commands are verbs (`init`, `run`, `build`, `lint`, `test`, `deploy`) or
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
├── init <path>                                     # Initialize workspace (alias: generate workspace)
│
├── generate
│   ├── workspace <path>                            # Full workspace scaffolding
│   ├── module <template> [--name <name>]           # Module scaffolding
│   ├── config <kind>                               # Runtime config scaffolding
│   ├── manifest                                    # Cyberfabric.toml scaffolding
│   ├── build <kind>                                # Build artifact templates (docker, helm, compose)
│   ├── ci <provider>                               # CI workflow templates (github)
│   ├── agents                                      # AGENTS.md generation
│   └── skill                                       # SKILL.md generation
│
├── manifest
│   ├── validate                                    # Validate manifest structure and references
│   ├── render [--env <env>] [--app <app>]          # Render resolved generation model
│   ├── add <env> <app> [<module-ref>...]           # Add environment/app/modules
│   ├── edit <env> <app> [flags]                    # Edit app settings
│   ├── rm <env> <app> [<module-ref>]               # Remove environment/app/module
│   └── migrate [--from <v>] [--to <v>]             # Migrate manifest schema version
│
├── list
│   ├── modules [--env <env>] [--app <app>]         # All modules (local + system + configured)
│   ├── local-modules                               # Workspace-discovered modules
│   ├── system-modules                              # Built-in system module registry
│   ├── configs                                     # Runtime config files
│   └── apps                                        # Manifest-defined apps and environments
│
├── help
│   ├── schema <manifest|config|module>             # Schema reference
│   ├── docs <rust-path>                            # Rust source resolution (alias: docs)
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
├── docs [<query>]                                  # Rust source resolution (kept for compat)
├── lint                                            # Orchestrated linting
├── test                                            # Orchestrated testing
├── tools                                           # Tool bootstrap (rustup, fmt, clippy)
├── run                                             # Generate and run server
├── build                                           # Generate and build server
├── deploy                                          # Build Docker image
├── completions --shell <shell>                      # Generate shell completions
└── man --output-dir <dir>                           # Generate man pages
```

## Verb Taxonomy

| Verb | Meaning | Side Effects |
|---|---|---|
| `init` | Create a new workspace from scratch | Creates directories and files |
| `generate` | Scaffold a specific artifact from a template | Creates files, may modify workspace Cargo.toml |
| `manifest` | Inspect or mutate `Cyberfabric.toml` | `validate`/`render` are read-only; `add`/`edit`/`rm` mutate |
| `list` | Read-only inspection of workspace state | None |
| `help` | Read-only documentation and schema output | None |
| `config` | Mutate runtime config YAML | Modifies config file |
| `lint` | Run quality checks | None (read-only analysis) |
| `test` | Run test suites | May generate app for e2e; test execution |
| `run` | Generate server and execute it | Generates project, runs process |
| `build` | Generate server and compile it | Generates project, compiles binary |
| `deploy` | Build a Docker image | Generates project, invokes Docker |
| `tools` | Install or upgrade toolchain components | May install system components |
| `docs` | Resolve and print Rust source | May download/cache crate sources |
| `completions` | Generate shell completion scripts | Writes to stdout or file |
| `man` | Generate man pages | Writes man page files |

## Shared Argument Patterns

### Workspace Path: `-p, --path <PATH>`

Available on: `config`, `lint`, `test`, `run`, `build`, `deploy`, `generate module`, `list`, `docs`.

Sets the workspace root. When provided, the CLI changes the working directory before resolving any other paths. When
omitted, the current directory is used.

### Runtime Config: `-c, --config <PATH>`

Available on: `config mod`, `config db`, `run`, `build`, `deploy`.

Path to the YAML runtime config file. Required when operating without a manifest, or to override the manifest-declared
config. Resolved relative to the workspace root after `-p/--path` is applied.

### Manifest Selection: `--manifest <PATH>`, `--env <ENV>`, `--app <APP>`

Available on: `run`, `build`, `deploy`, `lint`, `test`, `manifest`, `list modules`, `list apps`.

`--manifest` overrides automatic `Cyberfabric.toml` discovery. `--env` and `--app` select the target within the
manifest. When the manifest has exactly one environment and one app, they are selected automatically.

### Output Format: `--format <table|json|yaml|toml>`

Available on: `list`, `manifest validate`, `manifest render`, `help schema`.

Controls output serialization. When omitted, the CLI detects the terminal: `table` for interactive TTYs, `json` when
stdout is not a TTY. The `CF_CLI_FORMAT` environment variable can override the auto-detected default. The explicit
`--format` flag always takes highest precedence.

### Dry Run: `--dry-run`

Available on: `run`, `build`, `deploy`, `generate`.

Prints the resolved plan (generation model, command invocation, file list) without executing side effects. Combined
with `--format json`, this is the stable contract for automation.

### Verbose: `-v, --verbose`

Available on: all commands.

Increases output detail. For subprocesses, shows their stdout/stderr in real time instead of capturing it.

### Non-Interactive: `--no-interactive`

Available on: all commands.

Disables all interactive prompts. In this mode, ambiguous selections that would normally prompt the user instead
produce an error with a suggestion to provide explicit flags. Automatically implied when stdin is not a TTY or
`CI=true` is set. See [11-ci-and-automation.md](./11-ci-and-automation.md#non-interactive-mode).

### Install Missing Tools: `--install-missing-tools`

Available on: `test`, `lint`.

When a required tool (e.g., `cargo-nextest`, `cargo-llvm-cov`) is not installed, install it automatically instead of
failing with a suggestion. Not enabled by default to avoid surprising side effects.

## Name Validation

All user-provided names (module names, app names, environment names, DB server names) are validated against:

```text
^[a-zA-Z][a-zA-Z0-9_-]*$
```

Rules:

- Must start with a letter.
- May contain letters, digits, hyphens, and underscores.
- Must not be empty.
- Maximum length: 64 characters.
- Case-sensitive; lowercase-with-hyphens is the recommended convention.

Validation is enforced at the clap parsing layer using a custom `value_parser` so that invalid names never reach
command logic.

## Command Aliases and Migration

| Legacy Command | New Command | Status |
|---|---|---|
| `init <path>` | `generate workspace <path>` | `init` is a permanent alias |
| `mod add <template>` | `generate module <template>` | `mod add` is a permanent alias |
| `docs <query>` | `help docs <query>` | `docs` is a permanent alias |
| `deploy` | `build --output docker` | `deploy` remains for now; guided toward `build --output docker` |

Aliases are implemented at the clap level so that `--help` shows both forms and shell completions work for both.

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

### Man Pages

For systems that support them, man pages can be generated from clap metadata using `clap_mangen`:

```bash
cargo cyberfabric man --output-dir /usr/local/share/man/man1/
```

Man page generation is a build-time or install-time step, not a runtime dependency.
