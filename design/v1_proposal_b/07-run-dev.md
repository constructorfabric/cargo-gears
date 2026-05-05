# 07. Run for Development

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Proposed Commands](#proposed-commands)
4. [FIPS](#fips)
5. [OpenTelemetry](#opentelemetry)
6. [Watch Mode](#watch-mode)
7. [Manifest Integration](#manifest-integration)

## Purpose

Development run mode should make it easy to run one manifest app with the right
config, module set, feature flags, and local feedback loop.

## Current Behavior

`run` currently:

- requires `--config`
- optionally takes `--path`
- generates `.cyberfabric/<config-name>`
- sets `CF_CLI_CONFIG`
- supports `--watch`
- supports `--otel`
- supports `--fips`
- supports `--release`
- supports `--name`
- supports `--clean`

## Proposed Commands

```text
cargo cyberfabric run --env dev --app app1
cargo cyberfabric run --manifest Cyberfabric.toml --env dev --app app1
cargo cyberfabric run --config config/quickstart.yml
cargo cyberfabric run --env dev --app app1 --watch
cargo cyberfabric run --env dev --app app1 --fips --otel
```

The current `--config` flow should stay supported for migration and simple
projects. When both manifest app and `--config` are provided, the explicit
config path should override the manifest config path but not the manifest module
selection.

## FIPS

FIPS should remain a Cargo feature on the generated project:

```text
-F fips
```

Manifest:

```toml
[env.prod.app1.run]
fips = true
```

Validation:

- warn or fail if `fips = true` is configured outside approved environments,
  depending on a `policy` setting.
- fail if selected modules declare a known incompatible capability or feature.

## OpenTelemetry

OpenTelemetry should remain a generated project feature:

```text
-F otel
```

Runtime values remain in config:

```yaml
opentelemetry:
  tracing:
    endpoint: http://localhost:4317
```

Manifest only decides whether the generated app enables the dependency feature.

## Watch Mode

Watch mode should observe:

- workspace module source
- runtime config file
- manifest file
- generated server inputs

Suggested flags:

```text
--watch
--watch-path <PATH>
--ignore <GLOB>
```

Manifest:

```toml
[env.dev.app1.run.watch]
enabled = true
paths = ["modules", "config/dev-app1.yml", "Cyberfabric.toml"]
ignore = ["target", ".cyberfabric"]
```

For a first implementation, keep the current watch behavior and add manifest
selection. Path-level customization can follow.

## Manifest Integration

Resolution order:

1. CLI flags
2. app manifest run settings
3. defaults

Example final resolved run model:

```json
{
  "environment": "dev",
  "app": "app1",
  "config": "config/dev-app1.yml",
  "generated_project": ".cyberfabric/dev-app1",
  "features": ["otel"],
  "watch": true,
  "profile": "debug"
}
```

Add:

```text
cargo cyberfabric run --dry-run --format json
```

to print this model without executing Cargo.

