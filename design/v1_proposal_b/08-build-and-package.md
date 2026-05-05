# 08. Build and Package

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Build Outputs](#build-outputs)
4. [Binary Build](#binary-build)
5. [FIPS and OpenTelemetry](#fips-and-opentelemetry)
6. [Docker Image](#docker-image)
7. [Helm Chart](#helm-chart)
8. [Manifest Integration](#manifest-integration)

## Purpose

Build should orchestrate reproducible outputs from the manifest: generated
binary projects, Docker images, and eventually Helm chart packages.

## Current Behavior

`build` currently:

- requires `--config`
- generates `.cyberfabric/<config-name>`
- runs `cargo build`
- supports `--otel`, `--fips`, `--release`, `--name`, and `--clean`

`deploy` currently:

- builds a Docker image
- can generate a server project or accept `--manifest <Cargo.toml>`
- writes an embedded Dockerfile if none exists
- supports tag, debug mode, Dockerfile override, and build args

## Build Outputs

Output enum:

- `binary`
- `docker`
- `helm`

Command:

```text
cargo cyberfabric build --env prod --app app1 --output binary
cargo cyberfabric build --env prod --app app1 --output docker
cargo cyberfabric build --env prod --app app1 --output helm
cargo cyberfabric build --env prod --app app1 --output all
```

`deploy` can remain as a Docker-focused compatibility command, but new users
should be guided toward `build --output docker`.

## Binary Build

Binary build should generate the app and run:

```text
cargo build
```

or:

```text
cargo build --release
```

Manifest:

```toml
[env.prod.app1.build.binary]
profile = "release"
name = "app1"
```

The generated project location should be deterministic:

```text
.cyberfabric/<env>-<app>
```

unless explicitly overridden.

## FIPS and OpenTelemetry

Build feature selection should be shared with run:

```toml
[env.app1.prod.features]
fips = true
otel = true
```

Recommendation: use structured booleans for stable platform features and a
repeatable `extra_features` list for advanced features.

## Docker Image

Command:

```text
cargo cyberfabric build --env prod --app app1 --output docker --tag app1:latest
```

Recommendation: cargo-chef for the caching.

Manifest:

```toml
[env.prod.app1.build.docker]
image = "registry.example.com/app1"
tag = "1.2.3"
dockerfile = "Dockerfile"
build_args = { RUSTFLAGS = "-C target-cpu=x86-64-v3" }
```

The current `deploy` implementation can be refactored behind a `DockerBuilder`
service used by both:

- `deploy`
- `build --output docker`

The Dockerfile should be generated explicitly by `generate build docker` for
new projects. Writing an embedded Dockerfile during build can remain as a
compatibility fallback but should print a notice.

## Helm Chart

Command:

```text
cargo cyberfabric build --env prod --app app1 --output helm
```

Manifest:

```toml
[env.app1.prod.build.helm]
chart = "charts/app1"
version = "0.1.0"
app_version = "1.2.3"
values = "charts/app1/values.yaml"
```

Initial behavior:

- validate chart exists
- render values from manifest and runtime config reference
- package chart with `helm package`

Future behavior:

- `helm template` validation
- schema generation for values
- optional image tag injection from Docker build output

## Manifest Integration

Recommended build pipeline:

1. Validate manifest app.
2. Resolve modules.
3. Generate server project.
4. Build binary if requested or required by Docker.
5. Build Docker image if requested.
6. Package Helm chart if requested.
7. Print a build summary.

Example summary:

```text
Built app prod/app1
  binary: .cyberfabric/prod-app1/target/release/app1
  image: registry.example.com/app1:1.2.3
  helm: charts/app1-0.1.0.tgz
```

