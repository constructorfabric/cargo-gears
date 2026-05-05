# 06. Test

## Table of Contents

1. [Purpose](#purpose)
2. [Proposed Commands](#proposed-commands)
3. [Test Sets](#test-sets)
4. [Runner Configuration](#runner-configuration)
5. [Coverage](#coverage)
6. [Manifest Integration](#manifest-integration)

## Purpose

Testing should become a real orchestrated command that can run the appropriate
test set for a workspace, module, app, or environment.

The main idea is to leverage the manifest to define the test sets and the test
runners, so we can have an automation about what specific test sets succeed and which needs revision.

## Proposed Commands

```text
cargo cyberfabric test
cargo cyberfabric test --module <module>
cargo cyberfabric test --set unit
cargo cyberfabric test --set integration
cargo cyberfabric test --set e2e
cargo cyberfabric test --coverage
cargo cyberfabric test --runner cargo-test
cargo cyberfabric test --runner nextest
cargo cyberfabric test --env dev --app app1
```

Compatibility:

- `--e2e` remains an alias for `--set e2e`.

## Test Sets

Initial enum:

- `unit`: crate-local unit tests.
- `integration`: workspace integration tests.
- `e2e`: end-to-end tests requiring generated app/config.
- `module`: tests scoped to one CyberFabric module.
- `all`: all configured sets.

Suggested behavior:

- no flags: run manifest default or `unit + integration`.
- `--module`: limit package selection to the resolved local module package.
- `--env --app`: generate or validate the selected app before running e2e tests.
- `--set e2e`: require either manifest app selection or explicit config.

## Runner Configuration

Runner enum:

- `cargo-test`
- `nextest`

Cargo test command:

```text
cargo test --workspace --all-features
```

Nextest command:

```text
cargo nextest run --workspace --all-features
```

Manifest:

```toml
[env.dev.app1.test]
runner = "nextest"
sets = ["unit", "integration"]
coverage = false

[env.prod.app1.test]
runner = "cargo-test"
sets = ["unit", "integration", "e2e"]
coverage = true
```

Runner resolution:

1. CLI `--runner`
2. app manifest runner
3. workspace default runner
4. `cargo-test`

If `nextest` is selected and missing, the CLI should print the exact install
command and fail unless `--install-missing-tools` is passed.

## Coverage

Coverage should be a mode, not a separate runner.

Preferred tool:

```text
cargo llvm-cov
```

Suggested commands:

```text
cargo cyberfabric test --coverage
cargo cyberfabric test --coverage --coverage-format lcov
cargo cyberfabric test --coverage --coverage-format html
```

Coverage format enum:

- `summary`
- `lcov`
- `html`
- `json`

Coverage should support module/app filtering where possible, but the first
implementation can run workspace coverage and label the limitation clearly.

## Manifest Integration

Manifest can define named sets:

```toml
[test-sets.unit]
runner = "cargo-test"
args = ["--workspace", "--all-features"]

[test-sets.e2e]
runner = "nextest"
args = ["run", "--workspace", "--features", "e2e"]
requires_generated_app = true
```

This keeps the CLI opinionated for common cases while allowing advanced teams to
make their test topology explicit.

