# 07. Documentation and LLM Helpers

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
cargo cyberware help schema <manifest|config|module>
cargo cyberware help docs <rust-path>
cargo cyberware help topic <topic>
cargo cyberware docs <rust-path>
```

Keep `docs` for compatibility and add `help docs` as an alias for discoverable
help grouping.

## Schema Help

Command:

```text
cargo cyberware help schema manifest
cargo cyberware help schema config [-m <module>]
```

Should print:

- schema version
- supported fields
- defaults
- enum values
- examples
- compatibility notes

Examples:

```text
cargo cyberware help schema manifest --section apps.app.dev.test
cargo cyberware help schema config
```

Schema help should be generated from the same Rust types used by parsing where
possible, so documentation does not drift.

## LLM-Oriented Docs

Command:

```text
cargo cyberware help docs cf-modkit::bootstrap::run_server
```

## Other Information

`help topic` should expose short operational docs:

```text
cargo cyberware help topic manifest
cargo cyberware help topic module-refs
cargo cyberware help topic generated-server
cargo cyberware help topic fips
cargo cyberware help topic otel
```

The list is not exhaustive and can be expanded as needed.
