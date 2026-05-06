# 10. CI and Automation

## Table of Contents

1. [Purpose](#purpose)
2. [CI Pipeline Patterns](#ci-pipeline-patterns)
3. [LLM Integration](#llm-integration)
4. [Idempotency](#idempotency)

## Purpose

The CLI is designed to be a first-class citizen in CI pipelines, LLM-driven automation, and scripted workflows.
Every command that takes user input can be fully driven by flags and environment variables.

## CI Pipeline Patterns

### Recommended GitHub Actions Workflow

```yaml
name: CI
on: [ push, pull_request ]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Install CyberFabric CLI
        run: cargo install --git https://github.com/cyberfabric/cf-cli
      - name: Validate manifest
        run: cargo cyberfabric manifest validate --format json
      - name: Lint
        run: cargo cyberfabric lint --env prod --app app1 --strict
      - name: Test
        run: cargo cyberfabric test --env prod --app app1 --coverage --coverage-format lcov
      - name: Build
        run: cargo cyberfabric build --env prod --app app1 --output binary --release
```

Or with alias:

```yaml
name: CI
on: [ push, pull_request ]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - name: Install CyberFabric CLI
        run: cargo install --git https://github.com/cyberfabric/cf-cli
      - name: Validate manifest
        run: cargo cyberfabric ci --env prod --app app1
```

### Key Patterns

- **Validate first.** `manifest validate` catches configuration errors before expensive build steps.
- **Use manifest policies.** `--env prod --app app1` reads lint/test/build policies from the manifest. CI does not
  duplicate flag logic.
- **Use `--strict` in CI.** Turns warnings into errors for lint checks.

### Pre-Flight Check

```bash
cargo cyberfabric build --env prod --app app1 --dry-run --format json | jq '.modules | length'
```

Dry-run produces the resolved generation model without executing anything. CI can validate the model before committing
to a full build.

## LLM Integration

The CLI is designed to be usable by LLM-driven coding agents (Codex, Copilot, Claude, etc.) with minimal ambiguity.

### Design Properties for LLMs

1. **Deterministic output.** Same inputs produce same outputs. LLMs can predict CLI behavior.
2. **Structured JSON.** LLMs can parse `--format json` output directly without regex.
3. **Explicit flags.** No implicit state or side effects. Every behavior is triggered by a flag or manifest entry.
4. **Error suggestions.** Error messages include runnable fix commands that LLMs can extract and execute.
5. **Dry-run.** LLMs can preview actions before executing them.
6. **Schema help.** `help schema` provides machine-readable schema documentation.
7. **SKILL.md.** The generated skill file provides a compact command reference optimized for LLM context windows.

### LLM Workflow Example

```text
1. LLM reads SKILL.md for command reference.
2. LLM runs: cargo cyberfabric list modules --format json
3. LLM parses the JSON to understand available modules.
4. LLM runs: cargo cyberfabric manifest render --env dev --app app1 --format json
5. LLM parses the generation model to understand the app structure.
6. LLM writes module code following the patterns in the existing modules.
7. LLM runs: cargo cyberfabric lint --format json
8. LLM parses lint results and fixes any issues.
9. LLM runs: cargo cyberfabric test --format json
10. LLM verifies tests pass.
```

### `manifest render` as LLM Context

`manifest render --format json` is the single most useful command for LLMs. It provides:

- The complete module list with versions and features.
- The resolved config path.
- The generated project path.
- The selected policies.

This is equivalent to a "project state snapshot" that gives the LLM complete context without reading multiple files.

## Idempotency

Most CLI commands are idempotent:

| Command             | Idempotent? | Notes                                                 |
|---------------------|-------------|-------------------------------------------------------|
| `manifest validate` | Yes         | Read-only                                             |
| `manifest render`   | Yes         | Read-only                                             |
| `list *`            | Yes         | Read-only                                             |
| `help *`            | Yes         | Read-only                                             |
| `lint`              | Yes         | Read-only analysis                                    |
| `run`               | No          | Starts a process                                      |
| `build`             | Yes         | Regenerates and rebuilds (same output for same input) |
| `deploy`            | Yes         | Regenerates and rebuilds Docker image                 |
| `generate *`        | Partially   | Fails if target exists; `--force` makes it idempotent |

CI pipelines can safely re-run validation, lint, and build steps without worrying about accumulated side effects.
