---
title: LLM & CI workflow
description: Using cargo-gears for LLM-assisted development and CI automation — source inspection, help topics, and structured output.
sidebar:
  label: LLM & CI workflow
  order: 5
---

The `cargo gears` CLI is designed to serve as a deterministic interface for LLMs,
CI systems, and scripts. Several commands produce structured output or
inline documentation that automation can consume without parsing free-form text.

## Source inspection: `src`

The `src` command resolves Rust source code for any crate, module, or item from
the workspace, local cache, or crates.io. This is the fastest way for an LLM or
CI job to read framework code without cloning repositories.

```bash
cargo gears src cf-gears-toolkit
cargo gears src cf-gears-toolkit::gts::plugin::BaseGearsPluginV1
cargo gears src --verbose tokio::sync
```

Resolution order:

1. **Local workspace metadata** — `cargo metadata --no-deps` in the workspace
2. **Local source cache** — cached crate sources under the OS temp directory
3. **crates.io** — downloads and extracts the crate source

Use `--version` to pin a specific crate version for the registry fallback:

```bash
cargo gears src --version 1.0.217 serde::de::Deserialize
```

### Library name mappings

`--libs` prints `library_name -> package_name` mappings, including renamed
dependencies:

```bash
cargo gears src --libs cf-gears-toolkit
```

This is useful when an LLM needs to map Rust crate names (e.g.
`gears_toolkit_macros`) to Cargo package names (e.g. `cf-gears-toolkit-macros`).

### Cache management

```bash
cargo gears src --clean
cargo gears src --clean -p /tmp/cf-demo tokio::sync
```

## Inline documentation: `help topic`

The `help topic` command prints operational documentation for framework topics.
This is self-contained reference text that an LLM can use as context without
reading source code.

```bash
cargo gears help topic architecture
cargo gears help topic gear-layout
cargo gears help topic generated-server
cargo gears help topic otel
```

Available topics:

| Topic | Scope |
|---|---|
| `architecture` | Framework architecture, three-tier hierarchy, principles |
| `cli` | CLI reference, guidelines, command overview |
| `clienthub` | Typed ClientHub, plugins, GTS |
| `database` | SecureConn, transactions, migrations, repository pattern |
| `errors` | RFC-9457 Problem error handling |
| `fips` | FIPS mode activation and usage |
| `gear-layout` | Gear directory structure and SDK pattern |
| `gear-refs` | How local and remote gears are referenced |
| `gears-catalog` | Gear categories and dependency rules |
| `generated-server` | How the ephemeral generated server project works |
| `lifecycle` | Gear lifecycle, cancellation, background tasks |
| `manifest` | Overview of `Gears.toml` and manifest-driven workflows |
| `otel` | OpenTelemetry activation and runtime configuration |
| `rest-api` | OperationBuilder, OpenAPI, SSE, and OData |
| `security` | AuthN, AuthZ, SecureConn, AccessScope |

## Schema introspection: `help schema`

Print the schema for manifest, config, or module formats. Useful for validating
that a `Gears.toml` or runtime config conforms to the expected structure.

```bash
cargo gears help schema manifest
cargo gears help schema config --section database
cargo gears help schema module
```

Manifest sections: `workspace`, `apps`, `templates`.
Config sections: `server`, `database`, `logging`, `opentelemetry`, `modules`.

## Structured output

Several commands support `--format json` for machine-readable output:

```bash
cargo gears manifest validate --format json
cargo gears manifest ls --format json
cargo gears ls modules --format json
```

`build --dry-run` and `run --dry-run` print JSON with the generated project
directory and each generated file path and contents — useful for inspecting what
the CLI would generate without building or running:

```bash
cargo gears run --app app1 --env dev --dry-run
```

## LLM development flow

A typical LLM-assisted development loop using the CLI:

1. **Scaffold** a workspace and module:

   ```bash
   cargo gears new /tmp/cf-demo
   cargo gears generate module --template api-db-handler -p /tmp/cf-demo
   ```

2. **Read framework source** for the APIs you need to use:

   ```bash
   cargo gears src cf-gears-toolkit::gts::plugin::BaseGearsPluginV1
   cargo gears help topic database
   ```

3. **Wire the module** into config:

   ```bash
   cargo gears config mod add api-db-handler -p /tmp/cf-demo -c /tmp/cf-demo/config/app1-dev.yml
   cargo gears config db add primary -c /tmp/cf-demo/config/app1-dev.yml \
     --engine postgres --host localhost --port 5432 --user app --password '${DB_PASSWORD}' --dbname appdb
   cargo gears config mod db add api-db-handler -c /tmp/cf-demo/config/app1-dev.yml --server primary
   ```

4. **Run and iterate**:

   ```bash
   cargo gears run -p /tmp/cf-demo --app app1 --env dev --watch
   ```

5. **Lint** before committing:

   ```bash
   cargo gears lint --app app1 --env dev
   ```

## See also

- [Command reference](/cli/commands/) — full `src`, `help`, and `ls` documentation
- [Gears.toml manifest](/cli/manifest/) — manifest schema for `help schema manifest`
