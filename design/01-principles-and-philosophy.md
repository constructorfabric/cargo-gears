# 01. Principles and Philosophy

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Standards the CLI Enforces](#standards-the-cli-enforces)
4. [Tradeoff Guidelines](#tradeoff-guidelines)
5. [Non-Goals](#non-goals)

## Overview

The CyberFabric CLI is a **deterministic enforcement layer** for the CyberFabric framework. It exists because:

- Framework adoption has a high cognitive barrier regardless of developer background.
- LLM-generated code is fast but non-deterministic and can drift from approved conventions.
- Manual setup of workspaces, configs, and build pipelines is error-prone and inconsistent.
- Without a canonical tool, teams make divergent decisions that compound over time.

The CLI makes the correct path the easiest path. Every command, default, and validation rule should guide developers
toward CyberFabric-approved patterns without requiring them to read or memorize the framework's internal design.

## Core Principles

### 1. Convention Over Configuration

Every command has sensible defaults derived from CyberFabric conventions. Explicit overrides are available but never
required for the common case.

- `init` produces a workspace that compiles and runs without further configuration.
- `lint` runs all quality gates by default; individual suites are opt-in only when narrowing scope.
- `run` resolves config path, module set, and features from the manifest without requiring flags.
- Generated project paths follow a deterministic naming convention: `.cyberfabric/<env>-<app>`.

### 2. Manifest-First Orchestration

The manifest (`Cyberfabric.toml`) is the single source of truth for what the CLI generates and orchestrates. Runtime
configuration is the single source of truth for runtime values. This separation eliminates the ambiguity of the current
config-centric model where dependency metadata and runtime values coexist.

- Manifest answers: **what** to build (apps, environments, modules, features, policies).
- Runtime config answers: **how** to behave at runtime (endpoints, credentials, tuning).

### 3. Deterministic Outputs

Given the same manifest, config, and workspace state, the CLI must produce byte-identical generated artifacts. This
property is critical for:

- Reproducible builds in CI.
- Predictable behavior for LLM-driven automation.
- Debugging: if the output changes, the input changed.

Implementation rules:

- No timestamp injection into generated files.
- No random identifiers in generated code.
- Sorted dependency lists and module orderings.
- Canonical formatting of all generated TOML, YAML, and Rust files.

### 4. Fail Fast, Fail Clearly

Every command validates its inputs before performing side effects. Errors are structured, actionable, and
machine-readable.

- Validation runs before file generation, before network calls, before subprocess invocation.
- Error messages include the offending value, the constraint that was violated, and a suggested fix.
- Exit codes follow a defined taxonomy (see [09-developer-experience.md](./09-developer-experience.md)).
- `--format json` produces structured error objects, not human prose.

### 5. Orchestrate, Don't Replace

The CLI wraps `cargo`, `cargo-generate`, `cargo-clippy`, `cargo-fmt`, `cargo-nextest`, `cargo-llvm-cov`, `docker`,
and `helm`. It never reimplements their core functionality.

- `lint` invokes `cargo fmt` and `cargo clippy` with the right flags, not a custom formatter.
- `build` invokes `cargo build` inside a generated project, not a custom compiler.
- `deploy` invokes `docker build` with controlled arguments, not a custom image builder.

When a wrapped tool fails, the CLI surfaces the tool's stderr alongside its own context. The developer can always
reproduce the failure by running the underlying tool directly.

### 6. Machine-Readable by Default

Every command that produces output supports `--format json` for stable, schema-versioned structured output. Table
output is the default for humans; JSON is the stable contract for tools, CI, and LLMs.

- List commands: `--format table|json|yaml|toml`.
- Validation: `--format json` produces an array of diagnostic objects.
- Render/dry-run: `--format json` produces the resolved generation model.
- Errors: JSON mode wraps errors in `{"error": {"code": "...", "message": "...", "detail": ...}}`.

### 7. Secure by Default

The CLI never embeds secrets in generated files. Credentials flow through environment variable expansion
(`${VAR}` syntax) in config files and are resolved at runtime.

- Generated code reads config paths from `CF_CLI_CONFIG`, not hardcoded paths.
- Database passwords in config use `${DB_PASSWORD}` notation.
- `deploy` does not copy `.env` files into Docker images.
- `manifest render` redacts values that look like secrets.

### 8. Backward Compatible

Existing commands and flags continue to work across minor and patch releases. Breaking changes require a major version
bump and a documented migration path.

- The current `--config`-based flow remains supported alongside the manifest-first flow.
- Deprecated flags produce warnings for at least one minor release cycle before removal.
- `manifest init --from-config` provides a migration path from config-centric to manifest-first.

## Standards the CLI Enforces

The following standards are enforced across all generated code, configuration, and workflows:

| Standard | How the CLI Enforces It |
|---|---|
| Standardized project structure | `init` and `generate workspace` produce a canonical layout |
| Consistent naming conventions | Name validation regex `[a-zA-Z0-9_-]+` on modules, apps, environments |
| Approved architectural patterns | Module templates encode approved patterns; no freeform scaffolding |
| Reusable scaffolding templates | `generate` commands use a versioned template registry |
| Validated configuration | `manifest validate` and implicit validation before every build/run |
| Dependency and version consistency | Workspace dependency promotion; manifest pins module versions |
| Secure-by-default defaults | Env-var expansion for secrets; no embedded credentials |
| Consistent logging and observability | `--otel` feature flag; standardized tracing config schema |
| Built-in linting and guardrails | `lint` orchestrates fmt, clippy, and custom dylint rules |
| Testing conventions | `test` orchestrates runners, sets, and coverage with manifest policy |
| Documentation and discoverability | `help schema`, `help topic`, `list` commands |
| Backward compatibility | Versioned manifest schema; `--config` flow preserved |

## Tradeoff Guidelines

When making design decisions, apply these tradeoff rules in order:

1. **Correctness over convenience.** A command that silently produces wrong output is worse than one that fails with a
   clear error.

2. **Determinism over flexibility.** If a behavior can be deterministic, it must be. Non-deterministic behavior requires
   an explicit opt-in flag.

3. **Convention over configuration.** If a reasonable default exists, it should be the default. Configuration is for
   exceptions, not the common case.

4. **Explicit over implicit.** When the CLI takes a significant action (generating files, modifying config, invoking
   Docker), it should state what it is doing. Use `--dry-run` to preview.

5. **Machine-first, human-friendly.** Design output for programmatic consumption first, then add human-friendly
   formatting. Never produce output that is human-readable but unparseable.

6. **Minimal surface, maximum coverage.** Fewer commands with composable flags beat many specialized commands. But
   don't sacrifice discoverability for minimalism.

## Non-Goals

The CLI intentionally does not:

- **Replace Cargo.** Developers still use `cargo` directly for Rust-specific tasks the CLI does not orchestrate.
- **Manage infrastructure.** Deployment to Kubernetes, cloud providers, or other infrastructure is out of scope.
  The CLI produces artifacts (binaries, images, charts) that deployment tools consume.
- **Enforce at runtime.** The CLI enforces standards at development time. Runtime enforcement is the framework's
  responsibility.
- **Provide a GUI.** The CLI is a terminal tool. IDE integration should consume the CLI's JSON output, not replace it.
- **Support non-Rust languages.** CyberFabric is a Rust framework. The CLI is Rust-specific.
