# 08. Lint and Test

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Modes](#proposed-modes)
4. [Manifest Integration](#manifest-integration)
5. [Runner Configuration](#runner-configuration)
6. [Coverage](#coverage)

## Purpose

Linting should be an orchestrated quality gate that can run locally, in CI, and
through manifest-defined app policies.

Testing should become a real orchestrated command that can run the appropriate
test set for a workspace, module, app, or environment.

The main idea is to leverage the manifest to define the lint/test sets and test
runners, so we can have an automation about what specific test sets succeed and which needs revision.

## Linting

### Current Behavior

`lint` supports:

- default all available lint suites
- `--fmt`
- `--clippy`
- `--dylint`
- `--strict`
- `--workspace`

Clippy currently uses:

```text
cargo clippy --workspace --all-targets --all-features
```

Fmt currently uses:

```text
cargo fmt --check --all
```

Dylint runs embedded custom rules when the binary is built with `dylint-rules`.

### Proposed Modes

Add lint targets from the manifest:

```text
cargo gears lint
cargo gears lint --app app1
```

### Manifest Integration

Manifest example:

```toml
[apps.app1.dev.lint]
skip-dylint = [
    "rule-name"
]
# To skip all lint rules
# skip-dylint = true
clippy = true # by default
fmt = true # by default
feature-set-test = true # inherits the feature set to test
```

## Testing

### Runner Configuration

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

Runner resolution:

1. CLI `--runner`
2. app manifest runner
3. workspace default runner
4. `cargo-test`

If `nextest` is selected and missing, the CLI should print the exact install
command and fail unless `--install-missing-tools` is passed.

### Testing matrix

Testing matrix is a list of test sets that can be run with `cargo gears test --app app1 --env dev --matrix default`.
Test execution uses the runtime config declared by the selected environment, for example `apps.app1.dev.config`.

```toml
[apps.app1.dev.test.default]
runner = "nextest"
feature-set = {
    "module1" = [
        ["unit", "integration"],
        ["sqlite"],
        ["postgres"],
        ["fips"],
        false # disable all features
    ],
    "module2" = true, # enable all features
}
```

```toml
[apps.app1.dev.test.integration]
custom-command = "./scripts/integration-tests.sh"
# custom-command = "uv run e2e/python/test_cases.py"
```

### Coverage

Coverage is selected by the CLI and runs through the llvm-cov runner.

Preferred tool:

```text
cargo llvm-cov
```

Suggested commands:

```text
cargo gears test --coverage
cargo gears test --coverage --coverage-format lcov
cargo gears test --coverage --coverage-format html
```

Coverage format enum:

- `summary`
- `lcov`
- `html`
- `json`

Coverage should support module/app filtering where possible, but the first
implementation can run workspace coverage and label the limitation clearly.
