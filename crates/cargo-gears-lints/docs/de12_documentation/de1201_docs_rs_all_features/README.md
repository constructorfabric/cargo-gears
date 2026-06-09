# DE1201 — Docs.rs All Features

Publishable crates must enable docs.rs all-features builds.

## What it checks

Every crate where `publish` is not `false` must have:

```toml
[package.metadata.docs.rs]
all-features = true
```

## Why

docs.rs builds each crate with a constrained feature set unless configured otherwise.
Enabling all features catches documentation failures for optional feature combinations
before publishing and keeps public API docs complete.

## Exclusions

Crates can be excluded from this check via:

- **`dylint.toml`** — add the crate name to `[de1201_docs_rs_all_features].excluded_crates`
- **Environment variable** — set `DE1201_DOCS_RS_ALL_FEATURES_EXCLUDED_CRATES` (comma- or whitespace-separated list)

## Examples

### Bad

```toml
[package]
name = "my-crate"
version = "0.1.0"
```

Missing `[package.metadata.docs.rs]` section entirely.

### Good

```toml
[package]
name = "my-crate"
version = "0.1.0"

[package.metadata.docs.rs]
all-features = true
```

### Skipped

```toml
[package]
name = "internal-tool"
version = "0.1.0"
publish = false
```

Crates with `publish = false` are not checked.
