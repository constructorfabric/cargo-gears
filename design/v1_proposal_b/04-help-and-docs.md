# 04. Help and Docs

## Table of Contents

1. [Purpose](#purpose)
2. [Current Behavior](#current-behavior)
3. [Help Surfaces](#help-surfaces)
4. [Schema Help](#schema-help)
5. [LLM-Oriented Docs](#llm-oriented-docs)
6. [Other Information](#other-information)

## Purpose

Help should serve two audiences:

- developers working at the terminal
- LLMs and automation that need compact, precise, source-backed context

## Current Behavior

`docs` already resolves Rust source from:

- local workspace metadata
- cached registry packages
- `crates.io`

It can print source directly, list library mappings, select versions, and clean
cache.

## Help Surfaces

Proposed commands:

```text
cargo cyberfabric help schema <manifest|config|module>
cargo cyberfabric help docs <rust-path>
cargo cyberfabric help topic <topic>
cargo cyberfabric docs <rust-path>
```

Keep `docs` for compatibility and add `help docs` as an alias for discoverable
help grouping.

## Schema Help

Command:

```text
cargo cyberfabric help schema manifest
cargo cyberfabric help schema config
cargo cyberfabric help schema config module
```

Should print:

- schema version
- supported fields
- defaults
- enum values
- examples
- compatibility notes

Recommended output flags:

```text
--format markdown|json|yaml
--section <path>
```

Examples:

```text
cargo cyberfabric help schema manifest --section env.app.test
cargo cyberfabric help schema config --format json
```

Schema help should be generated from the same Rust types used by parsing where
possible, so documentation does not drift.

## LLM-Oriented Docs

Command:

```text
cargo cyberfabric help docs cf-modkit::bootstrap::run_server
```

## Other Information

`help topic` should expose short operational docs:

```text
cargo cyberfabric help topic manifest
cargo cyberfabric help topic module-refs
cargo cyberfabric help topic generated-server
cargo cyberfabric help topic fips
cargo cyberfabric help topic otel
cargo cyberfabric help topic docker
cargo cyberfabric help topic helm
```

These topics should be concise and should link to generated schema help where
possible.

