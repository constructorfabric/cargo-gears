# 05. Lint

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Modes](#proposed-modes)
4. [Manifest Integration](#manifest-integration)
5. [Dylint](#dylint)
6. [Clippy](#clippy)
7. [Fmt](#fmt)

## Purpose

Linting should be an orchestrated quality gate that can run locally, in CI, and
through manifest-defined app policies.

## Current Behavior

`lint` supports:

- default all available lint suites
- `--fmt`
- `--clippy`
- `--dylint`
- `--strict`
- `--path`

Clippy currently uses:

```text
cargo clippy --workspace --all-targets --all-features
```

Fmt currently uses:

```text
cargo fmt --check --all
```

Dylint runs embedded custom rules when the binary is built with `dylint-rules`.

## Proposed Modes

Add lint targets from the manifest:

```text
cargo cyberfabric lint
cargo cyberfabric lint --app app1
```

## Manifest Integration

Manifest example:

```toml
[env.app1.lint]
skip-dylint = [
    "rule-name"
]
# To skip all lint rules
# skip-dylint = true
clippy = true # by default
fmt = true # by default
feature-set-test = true # inherits the feature set to test
```
