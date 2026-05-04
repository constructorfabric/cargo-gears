# 07. Quality Gates

## Table of Contents

1. [Purpose](#purpose)
2. [Lint](#lint)
3. [Test](#test)
4. [Coverage](#coverage)
5. [Manifest Integration](#manifest-integration)

## Purpose

Lint and test are orchestrated quality gates that run locally and in CI. The CLI manages tool invocation, flag
construction, and policy enforcement so that every developer and every pipeline runs the same checks with the same
configuration. The manifest defines per-app policies; the CLI enforces them.

## Lint

### Current Behavior

`lint` supports `--fmt`, `--clippy`, `--dylint`, `--strict`, and `--path`. Clippy runs with
`--workspace --all-targets --all-features`. Fmt runs with `--check --all`. Dylint runs embedded custom rules.

### Proposed Commands

```bash
cargo cyberfabric lint [-p <path>] [--all] [--fmt] [--clippy] [--strict] [--dylint] [--env <env>] [--app <app>] [--format table|json]
```

### Behavior

| Condition | What Runs |
|---|---|
| No flags | All suites (fmt + clippy + dylint if available). Same as passing `--all`. |
| `--fmt` only | `cargo fmt --check --all` |
| `--clippy` only | `cargo clippy --workspace --all-targets --all-features` |
| `--dylint` only | Embedded Dylint rules |
| `--strict` | Turns Clippy warnings into errors. Only valid with `--clippy` or `--all`. |
| `--env` / `--app` | Reads lint policy from manifest. Applies skip rules, feature set overrides. |

### Manifest Lint Policy

```toml
[env.dev.app1.lint]
clippy = true          # default: true
fmt = true             # default: true
dylint = true          # default: true when available
skip_dylint_rules = ["rule-name"]  # skip specific rules (list of strings)
strict = false         # default: false
```

Policy resolution order:

1. CLI flags (highest precedence).
2. App manifest lint policy.
3. CLI defaults (all suites enabled).

### Lint Output

Default output shows pass/fail per suite with summary counts.

`--format json` output:

```json
{
  "suites": [
    {"name": "fmt", "status": "pass", "duration_ms": 1200},
    {"name": "clippy", "status": "fail", "warnings": 3, "errors": 1, "duration_ms": 15000},
    {"name": "dylint", "status": "skip", "reason": "not available"}
  ],
  "overall": "fail"
}
```

### Clippy Configuration

The workspace `Cargo.toml` already defines Clippy lint levels:

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
```

The CLI respects these settings. It does not override them unless `--strict` is passed, which adds `-D warnings` to
turn all Clippy warnings into errors.

### Dylint Rules

Dylint rules are compiled into the CLI binary when built with the `dylint-rules` feature. Before running Dylint, the
CLI ensures the required toolchains are installed.

If the feature is not enabled, `lint --dylint` prints a clear error explaining the limitation and suggesting how to
build the CLI with Dylint support.

## Test

### Current Status

`test` is declared but currently unimplemented (`unimplemented!("Not implemented yet")`).

### Proposed Commands

```bash
cargo cyberfabric test [-p <path>] [--module <name>] [--set <set>] [--runner <runner>] [--coverage] [--coverage-format <format>] [--env <env>] [--app <app>] [--format table|json]
```

### Test Sets

| Set | Description |
|---|---|
| `unit` | Crate-local unit tests |
| `integration` | Workspace integration tests |
| `e2e` | End-to-end tests requiring a generated app and config |
| `module` | Tests scoped to a single CyberFabric module |
| `all` | All configured sets |

Default behavior (no flags): runs `unit` + `integration`, or whatever the manifest defines.

### Test Runners

| Runner | Command |
|---|---|
| `cargo-test` | `cargo test --workspace --all-features` |
| `nextest` | `cargo nextest run --workspace --all-features` |

Runner resolution order:

1. `--runner` CLI flag.
2. App manifest test runner.
3. Workspace default.
4. `cargo-test`.

If `nextest` is selected and not installed, the CLI prints the install command and fails unless
`--install-missing-tools` is passed (see
[03-command-surface.md](./03-command-surface.md#install-missing-tools---install-missing-tools)).

### Test Scoping

| Flag | Effect |
|---|---|
| `--module <name>` | Limits to the resolved local module package: `--package <package>` |
| `--set unit` | Runs unit tests only |
| `--set integration` | Runs integration tests only |
| `--set e2e` | Requires manifest app selection or explicit `--config`; generates/validates the app first |
| `--env <env> --app <app>` | Reads test policy from manifest; generates app for e2e if needed |

### E2E Test Flow

When `--set e2e` is used:

1. Resolve the target app (from `--env`/`--app` or manifest default).
2. Generate or validate the generated server project.
3. Set `CF_CLI_CONFIG` and other required environment variables.
4. Run the test suite with the e2e test set.

### Test Output

Default output shows pass/fail per test with summary counts.

`--format json` output:

```json
{
  "runner": "nextest",
  "sets": ["unit", "integration"],
  "results": {
    "passed": 42,
    "failed": 1,
    "skipped": 3,
    "duration_ms": 8500
  },
  "overall": "fail"
}
```

## Coverage

Coverage is a mode on the `test` command, not a separate runner.

```bash
cargo cyberfabric test --coverage [--coverage-format <format>]
```

### Coverage Tool

Preferred tool: `cargo-llvm-cov`.

If not installed, the CLI prints the install command and fails unless `--install-missing-tools` is passed (see
[03-command-surface.md](./03-command-surface.md#install-missing-tools---install-missing-tools)).

### Coverage Formats

| Format | Description |
|---|---|
| `summary` | Terminal summary (default) |
| `lcov` | LCOV format for CI integration |
| `html` | HTML report |
| `json` | JSON coverage data |

### Coverage Scoping

Coverage supports the same scoping flags as `test` (`--module`, `--set`, `--env`/`--app`). The first implementation
runs workspace-level coverage. Module-level coverage filtering is a future enhancement; the limitation is documented
clearly.

## Manifest Integration

### Manifest Test Policy

```toml
[env.dev.app1.test]
runner = "nextest"
sets = ["unit", "integration"]
coverage = false

[env.prod.app1.test]
runner = "cargo-test"
sets = ["unit", "integration", "e2e"]
coverage = true
coverage_format = "lcov"
```

### Named Test Sets

Advanced teams can define custom test sets **per-app** in the manifest, alongside the other policy sections:

```toml
[env.dev.app1.test]
runner = "nextest"
sets = ["unit", "integration"]
coverage = false

[[env.dev.app1.test.custom_sets]]
name = "unit"
runner = "cargo-test"
args = ["--workspace", "--all-features"]

[[env.dev.app1.test.custom_sets]]
name = "e2e"
runner = "nextest"
args = ["run", "--workspace", "--features", "e2e"]
requires_generated_app = true
```

Custom test sets live under the app's test policy (`env.<env>.<app>.test.custom_sets`) to keep them
scoped to the app they apply to. The built-in set names (`unit`, `integration`, `e2e`, `module`) work
without custom definitions; `custom_sets` is only needed to override arguments or runner for a specific set.

This keeps the CLI opinionated for common cases while allowing explicit test topology for advanced needs.

### Combined Quality Gate

For CI, a single command can run all quality gates:

```bash
cargo cyberfabric lint --env prod --app app1 && cargo cyberfabric test --env prod --app app1 --coverage
```

Both commands read their policies from the manifest and apply the correct flags automatically. CI pipelines do not
need to duplicate flag logic.
