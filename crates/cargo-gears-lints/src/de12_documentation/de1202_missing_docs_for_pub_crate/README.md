# DE1202 — Missing Documentation for `pub(crate)` APIs

## Rule

APIs declared with `pub(crate)`, or whose public visibility is effectively
limited to a `pub(crate)` API, must have non-empty Rust documentation. Together
with rustc's `missing_docs` lint, this gives documentation coverage for both
externally exported `pub` APIs and crate-public APIs.

DE1202 checks:

- `pub(crate)` modules, functions, constants, statics, type aliases, macros,
  structs, enums, unions, traits, and trait aliases;
- `pub` APIs nested under a `pub(crate)` module, whose effective visibility is
  crate-public and which rustc's `missing_docs` lint does not cover;
- `pub(crate)` fields, public fields of `pub(crate)` types, and all named
  enum-variant fields of a `pub(crate)` enum;
- variants and associated items declared by a `pub(crate)` trait or enum;
- `pub(crate)` associated items in inherent implementations and public
  associated items whose self type is crate-public;
- `pub(crate)` foreign functions and statics.

## Rationale

`pub(crate)` APIs form the internal contract between modules in a crate. They
need the same intent, invariants, and usage documentation as externally
exported APIs, but rustc's built-in `missing_docs` lint intentionally does not
check them. DE1202 fills only that gap rather than duplicating `missing_docs` for
exported `pub` items.

## Allowed Exceptions

The lint skips:

- test-target compilations, files under `tests/`, and companion `*_tests.rs`
  files;
- declarative- and procedural-macro-generated items and automatically derived
  implementations;
- associated items in trait implementations, whose documentation belongs on
  the trait declaration;
- items and subtrees hidden with `#[doc(hidden)]`;
- crates temporarily listed in `de1202_excluded_crates`.

`pub(in crate)` is treated as equivalent to `pub(crate)`. Other `pub(in ...)`
visibilities, `pub(super)`, and private items are outside DE1202's scope. Enable
rustc's `missing_docs` lint separately for exported `pub` APIs.

## Migrating Existing Crates

Use a crate exclusion only for pre-existing documentation debt:

```toml
[cargo-gears-lints]
de1202_excluded_crates = [
    "legacy-crate",
]
```

Entries may use the Cargo package name (`legacy-crate`) or rustc crate name
(`legacy_crate`); DE1202 normalizes hyphens to underscores before matching.
An exclusion disables DE1202 for every target whose Cargo package name or rustc
crate name matches. New crates remain denied by default because they are not on
the list.

Remove one entry after documenting that crate's crate-public API. Do not add new
crates merely to get CI green; each exclusion should represent tracked,
temporary migration debt.

## Forbidden

```rust,ignore
pub(crate) struct RetryPolicy {
    pub(crate) attempts: u32,
}

impl RetryPolicy {
    pub(crate) fn should_retry(&self) -> bool {
        self.attempts > 0
    }
}
```

## Allowed

```rust,ignore
/// Controls retry behavior inside this crate.
pub(crate) struct RetryPolicy {
    /// Maximum number of attempts.
    pub(crate) attempts: u32,
}

impl RetryPolicy {
    /// Returns whether another attempt is permitted.
    pub(crate) fn should_retry(&self) -> bool {
        self.attempts > 0
    }
}
```

## Guidance

Add a non-empty `///` comment or `#[doc = ...]` attribute that describes the
API's purpose and any important invariants. Documentation expressions such as
`include_str!` and `concat!` are checked after expansion, so an expression that
expands to an empty string does not count. Do not duplicate implementation
details that are already obvious from the code.

For complete coverage, configure the target workspace with:

```toml
[workspace.lints.rust]
missing_docs = "deny"
```
