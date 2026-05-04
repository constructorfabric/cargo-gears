# 09. Developer Experience

## Table of Contents

1. [Purpose](#purpose)
2. [Exit Codes](#exit-codes)
3. [Error Messages](#error-messages)
4. [Output Formatting](#output-formatting)
5. [Progress Indicators](#progress-indicators)
6. [Color and Styling](#color-and-styling)
7. [Interactive Prompts](#interactive-prompts)
8. [Environment Variables](#environment-variables)
9. [Diagnostics and Debugging](#diagnostics-and-debugging)

## Purpose

Every interaction with the CLI should be predictable, informative, and actionable. Errors tell the developer exactly
what went wrong and how to fix it. Output is structured for both human reading and machine parsing. The CLI respects
terminal capabilities and non-interactive environments.

## Exit Codes

The CLI uses a defined set of exit codes so that scripts, CI, and automation can branch on failure type without
parsing stderr.

| Code | Meaning | Examples |
|---|---|---|
| `0` | Success | Command completed without errors |
| `1` | General error | Unclassified runtime failure |
| `2` | Usage error | Invalid arguments, missing required flags (matches clap's default) |
| `3` | Validation error | Manifest validation failed, config schema error |
| `4` | Resolution error | Module not found, registry unreachable |
| `5` | Generation error | Template failure, code generation error |
| `6` | Tool error | Cargo/Docker/Helm invocation failed |
| `7` | I/O error | File not found, permission denied |
| `130` | Interrupted | User pressed Ctrl+C (SIGINT) |

> **Note on clap errors:** Exit code 2 is clap's default for parse failures. Because clap errors bypass the CLI's own
> error handling, they will **not** produce the structured JSON error format below. Custom error handling wrapping clap
> failures is required to achieve full JSON coverage.

### JSON Error Wrapping

In `--format json` mode, errors are wrapped in a structured object:

```json
{
  "error": {
    "code": "E003",
    "exit_code": 4,
    "message": "Local module 'payments' not found in workspace",
    "location": {
      "file": "Cyberfabric.toml",
      "line": 15,
      "field": "env.dev.app1.modules[1].name"
    },
    "suggestion": "Run 'cargo cyberfabric generate module api-db-handler --name payments' to create it"
  }
}
```

## Error Messages

Every error message follows this structure:

```text
error[E003]: local module 'payments' not found in workspace
  --> Cyberfabric.toml:15
   |
15 | name = "payments"
   |        ^^^^^^^^^^ module not found in workspace.members
   |
  = help: run 'cargo cyberfabric generate module api-db-handler --name payments' to create it
  = note: available local modules: background-worker, api-db-handler
```

### Error Code Namespace

Diagnostic codes use a single global namespace with a letter prefix and numeric suffix:

| Prefix | Domain | Range |
|---|---|---|
| `E0xx` | Manifest and config validation | `E001`-`E099` |
| `E1xx` | Module resolution | `E100`-`E199` |
| `E2xx` | Code generation and templates | `E200`-`E299` |
| `E3xx` | Tool orchestration (cargo, docker, helm) | `E300`-`E399` |
| `E4xx` | CLI usage and argument errors | `E400`-`E499` |
| `W0xx` | Manifest and config warnings | `W001`-`W099` |

Codes are stable within a major version. New codes may be added in any release.

### Error Message Rules

1. **Start with the error category** (`error`, `warning`, `note`).
2. **Include the diagnostic code** (`E003`, `W001`).
3. **State what is wrong** in plain language.
4. **Point to the source** (file, line, field) when applicable.
5. **Suggest a fix** with a runnable command when possible.
6. **List available alternatives** when the error is a misspelling or unknown value.

### Warning Messages

Warnings follow the same format but do not cause a non-zero exit code unless `--strict` is active:

```text
warning[W001]: production environment uses watch = true
  --> Cyberfabric.toml:25
   |
25 | watch = true
   |         ^^^^ watch mode is not recommended for production
   |
  = help: set watch = false for production environments
```

## Output Formatting

### Table Output

Table output is the default for interactive terminals. Tables use:

- Fixed-width columns with dynamic sizing based on terminal width.
- Header row with bold styling (when color is enabled).
- Aligned columns with consistent padding.
- Truncation with `...` for values exceeding column width.

### JSON Output

JSON output is the stable contract. It uses:

- `serde_json::to_string_pretty` for readability.
- Stable field ordering (struct field order, `BTreeMap` for maps).
- No trailing commas or comments.
- UTF-8 encoding.
- Newline-terminated.

### Streaming Output

Commands that produce incremental output (e.g., `run` with subprocess stdout) stream in real time to stderr.
Structured results go to stdout. This separation lets `command 2>/dev/null` suppress progress while keeping the
machine-readable result.

## Progress Indicators

Long-running operations show progress:

| Operation | Indicator |
|---|---|
| Template cloning | Spinner with "Cloning template..." |
| Cargo build | Pass-through of Cargo's own progress |
| Docker build | Pass-through of Docker's own progress |
| Registry fetch | Spinner with "Fetching module metadata..." |
| Module resolution | Spinner with "Resolving modules..." |

Progress indicators write to stderr so they do not interfere with stdout-based structured output.

In non-interactive mode (no TTY on stderr), progress indicators are replaced with simple line-based status messages:

```text
:: Cloning template from https://github.com/cyberfabric/cf-template-rust...
:: Resolving 3 modules...
:: Generating .cyberfabric/dev-app1/...
:: Running cargo build...
```

## Color and Styling

The CLI respects the `NO_COLOR` environment variable (https://no-color.org/) and the `CLICOLOR` / `CLICOLOR_FORCE`
conventions:

| Condition | Color Behavior |
|---|---|
| stdout is a TTY, `NO_COLOR` not set | Colors enabled |
| stdout is not a TTY | Colors disabled |
| `NO_COLOR` is set (any value) | Colors disabled |
| `CLICOLOR=0` | Colors disabled |
| `CLICOLOR_FORCE=1` | Colors enabled even without TTY |

Color palette:

| Element | Color |
|---|---|
| Error prefix and code | Red, bold |
| Warning prefix and code | Yellow, bold |
| Success messages | Green |
| File paths and field names | Cyan |
| Suggested commands | Bold |
| Section headers | Bold |

## Interactive Prompts

Some commands prompt for user input in interactive mode:

- `tools` prompts before installing/upgrading (skipped with `--yes` / `-y`).
- Ambiguous manifest selection (multiple envs/apps) prompts for selection.
- `generate workspace` in a non-empty directory prompts for confirmation (skipped with `--force`).

### Non-Interactive Fallback

When stdin is not a TTY, `CI=true` is set, or `--no-interactive` is passed, prompts are replaced with errors:

```text
error[E011]: ambiguous manifest selection, multiple apps found
  = help: specify --env and --app explicitly, or set CF_CLI_ENV and CF_CLI_APP
```

This ensures the CLI never hangs waiting for input in CI.

## Environment Variables

The CLI reads these environment variables:

| Variable | Purpose |
|---|---|
| `CF_CLI_CONFIG` | Runtime config path for the generated server |
| `CF_CLI_MANIFEST` | Override manifest path (equivalent to `--manifest`) |
| `CF_CLI_ENV` | Default environment (equivalent to `--env`) |
| `CF_CLI_APP` | Default app (equivalent to `--app`) |
| `CF_CLI_FORMAT` | Default output format (equivalent to `--format`) |
| `CF_CLI_LOG` | Log level for CLI diagnostics (`error`, `warn`, `info`, `debug`, `trace`) |
| `NO_COLOR` | Disable colored output |
| `CLICOLOR` / `CLICOLOR_FORCE` | Color control |

CLI flags always take precedence over environment variables. Environment variables take precedence over manifest
defaults.

## Diagnostics and Debugging

### Verbose Mode

`-v/--verbose` increases output detail across all commands:

- Prints resolved paths, discovered modules, and selected policies.
- Shows subprocess invocations with full argument lists.
- Streams subprocess stdout/stderr in real time.

### Debug Logging

`CF_CLI_LOG=debug` enables internal debug logging. This is intended for CLI developers, not end users. Debug output
goes to stderr and includes:

- Manifest parsing steps.
- Module resolution chain.
- Template variable expansion.
- File I/O operations.

### `manifest render`

The canonical debugging tool for "why did the CLI generate this?" is `manifest render`. It produces the fully resolved
generation model as structured output, showing exactly what the CLI would generate without executing anything.

### Version Information

```bash
cargo cyberfabric --version
```

Output includes:

- CLI version.
- Git commit hash (when built from source).
- Rust toolchain version.
- Feature flags enabled (e.g., `dylint-rules`).

In `--format json` mode:

```json
{
  "version": "0.1.0",
  "commit": "abc1234",
  "rustc": "1.85.0",
  "features": ["dylint-rules"]
}
```
