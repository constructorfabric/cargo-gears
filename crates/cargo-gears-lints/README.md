# Dylint Lints

Custom [dylint](https://github.com/trailofbits/dylint) linters enforcing architectural patterns, layer separation, and REST API conventions.

These rules are compiled by `cargo-gears-core`'s build script when the CLI is built with `dylint-rules`. During local development, the build uses the sibling `crates/cargo-gears-lints` path; in installed builds, it resolves this package from the Cargo registry at the version pinned by the `LINTS_PACKAGE_VERSION` constant in `crates/cargo-gears-core/build.rs`. The resulting Dylint library is embedded into the CLI.

## Available Lints

### DE01xx — Domain Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0101 | No Serde in Domain | Structs/enums in `/domain/` must not derive `Serialize`/`Deserialize` |
| DE0102 | No ToSchema in Domain | Structs/enums in `/domain/` must not derive `utoipa::ToSchema` |
| DE0104 | No API DTO in Domain | Structs/enums in `/domain/` must not use the `api_dto` macro |

### DE02xx — API Layer

| Rule | Name | What it enforces                                                            |
|------|------|-----------------------------------------------------------------------------|
| DE0201 | DTOs Only in API Rest | Types with `*Dto` suffix must live in `*/api/rest/*.rs`                     |
| DE0202 | DTOs Not Outside API | Contract, domain, and infra modules must not import DTO types               |
| DE0203 | DTOs Must Use `api_dto` | DTO types in `api/rest` must use `#[cf_gears_toolkit_macros::api_dto(...)]` |
| DE0204 | DTOs Must Have ToSchema | DTO types must derive `utoipa::ToSchema` for OpenAPI docs                   |

### DE03xx — Domain Layer (infra / HTTP boundaries)

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0301 | No Infra in Domain | Domain modules must not import infra crates (`cf_gears_toolkit_db`, `sea_orm`, etc.) |
| DE0308 | No HTTP in Domain | Domain modules must not reference `http`, `axum`, or `hyper` types |
| DE0309 | Must Have Domain Model | Externally-visible domain types must have `#[domain_model]` attribute |

### DE05xx — Client Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0503 | Plugin Client Suffix | Plugin client traits in `*-sdk` crates must use `*Client` suffix |
| DE0504 | Client Versioning | Client/PluginClient traits must have version suffixes (`V1`, `V2`, ...) |

### DE07xx — Security

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0706 | No Direct SQLx | Direct `sqlx` usage is forbidden; use Sea-ORM / SecORM abstractions |
| DE0707 | Drop Zeroize | Manual byte-zeroing in `Drop` impls must use `zeroize` or `secrecy` |
| DE0708 | No Non-FIPS Hasher | Direct `sha2`/`sha1`/`md5` imports are forbidden; allow-list configurable via `dylint.toml` |

### DE08xx — REST API Conventions

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0801 | API Endpoint Version | Endpoints must follow `/{service}/v{N}/{resource}` (kebab-case) |
| DE0802 | Use OData Ext | OData query params must use `OperationBuilderODataExt` methods |
| DE0803 | API Snake Case | API DTOs must use `snake_case` in serde `rename_all`/`rename` attrs |

### DE09xx — GTS Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0901 | GTS String Pattern | GTS identifiers must be valid; vendor allowlist configurable via `dylint.toml` |
| DE0902 | No `schema_for!` on GTS | GTS structs must use `gts_schema_with_refs_as_string()`, not `schema_for!` |
| DE0904 | No Hard-Coded GTS Prefix | GTS IDs must use `gts_id!("<suffix>")`, not a literal `gts.` prefix |

### DE12xx — Documentation

| Rule | Name | What it enforces |
|------|------|------------------|
| DE1201 | Docs.rs All Features | Publishable crates must set `package.metadata.docs.rs.all-features = true` |

### DE11xx — Testing

| Rule | Name | What it enforces |
|------|------|------------------|
| DE1101 | Tests in Separate Files | Inline test blocks must be extracted to `*_tests.rs` companion files |

### DE13xx — Common Patterns

| Rule | Name | What it enforces |
|------|------|------------------|
| DE1301 | No Print Macros | `println!`/`eprintln!`/`print!`/`eprint!` forbidden in production code |
| DE1302 | No `.to_string()` in From | Error `From` impls must not call `.to_string()` (use `.into()`) |
| DE1303 | No Primitive Type Alias | `pub type X = Uuid` etc. must be newtypes for type safety |

## Project Structure

```text
crates/cargo-gears-lints/
├── src/
│   ├── lib.rs                # Registers all lints in one dylint library
│   ├── lint_utils.rs         # Shared helpers
│   ├── de01_domain_layer/    # Lint implementations grouped by category
│   │   ├── de0101_no_serde_in_domain.rs
│   │   ├── de0101_no_serde_in_domain/
│   │   │   └── README.md     # Per-lint documentation, colocated with source
│   │   └── ...
│   ├── de02_api_layer/
│   └── ...
├── docs/
│   └── README.md             # Index linking to each lint README
├── tests/
│   └── ui/
│       ├── de0101_.../       # UI test fixtures (.rs + .stderr)
│       └── ...
├── Cargo.toml                # Publishable package manifest
└── rust-toolchain.toml       # Nightly channel for dylint
```

Each lint implementation lives in `src/<category>/<lint>.rs`. Per-lint documentation lives in `src/<category>/<lint>/README.md`, colocated with the source for discoverability. [docs/README.md](docs/README.md) serves as the index linking to each lint's README. UI fixtures live in `tests/ui/<lint>/`: `.rs` files contain code that should trigger (or not) the lint, and `.stderr` files contain the expected compiler diagnostics.

## Usage

```bash
cargo gears lint --dylint          # run only the architecture lints (this crate)
cargo gears lint --all             # run all lint stages: fmt + clippy + dylint
```

Individual stages can also be selected with `--fmt` or `--clippy`.

See [docs/README.md](docs/README.md) for links to each lint's detailed documentation.

## Adding a New Lint Rule

This section walks through the end-to-end process of authoring a new architecture lint, testing it locally, and getting it adopted in a target workspace (e.g. `gears-rust`).

### 1. Choose a rule ID and category

Rules follow the `DEccnn` numbering scheme, where `cc` is the category and `nn` is the rule number within that category. Pick the next available number in an existing category, or create a new category if none fits.

Existing categories:

| Prefix | Category | Source directory |
|--------|----------|-----------------|
| DE01xx | Domain layer (serde/schema) | `de01_domain_layer/` |
| DE02xx | API layer (DTOs) | `de02_api_layer/` |
| DE03xx | Domain layer (infra/HTTP) | `de03_domain_layer/` |
| DE05xx | Client layer | `de05_client_layer/` |
| DE07xx | Security | `de07_security/` |
| DE08xx | REST API conventions | `de08_rest_api_conventions/` |
| DE09xx | GTS layer | `de09_gts_layer/` |
| DE11xx | Testing | `de11_testing/` |
| DE12xx | Documentation | `de12_documentation/` |
| DE13xx | Common patterns | `de13_common_patterns/` |

Example: a new common-patterns rule would be **DE1304** in `de13_common_patterns/`.

### 2. Create the implementation file

Create `src/<category>/de<ccnn>_<snake_name>.rs`. The implementation uses the `rustc` lint infrastructure via `dylint_linting`. Choose the appropriate lint pass:

| Pass | When to use |
|------|-------------|
| `EarlyLintPass` (pre-expansion) | AST-level checks before macro expansion (derive attrs, macro calls) |
| `EarlyLintPass` | AST-level checks after macro expansion (use imports, struct fields) |
| `LateLintPass` | Type-resolved checks (trait impls, type information, cross-crate resolution) |

A minimal skeleton:

```rust
extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::Item;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_pre_expansion_lint! {
    /// DE1304: Short description of the rule
    ///
    /// Longer explanation of what it enforces and why.
    #[doc = include_str!("de1304_your_lint_name/README.md")]
    pub DE1304_YOUR_LINT_NAME,
    Deny,
    "short diagnostic message (DE1304)"
}

impl EarlyLintPass for De1304YourLintName {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Your lint logic here.
        // Use helpers from lint_utils.rs for path checks, filename extraction, etc.
    }
}
```

Use `Deny` severity for rules that should fail the build. Use `Warn` only if the rule is advisory.

The `include_str!` directive embeds the per-lint README into `rustc` documentation, so `cargo doc` includes it.

### 3. Register the lint in `lib.rs`

Three edits are needed in `src/lib.rs`:

**a)** Add the module declaration in the appropriate category block:

```rust
mod de13_common_patterns {
    // ... existing lints ...
    pub(crate) mod de1304_your_lint_name;
}
```

**b)** Register the lint constant in `register_lints`:

```rust
lint_store.register_lints(&[
    // ... existing lints ...
    de13_common_patterns::de1304_your_lint_name::DE1304_YOUR_LINT_NAME,
]);
```

**c)** Register the lint pass (match the pass type you chose in step 2):

```rust
// Pre-expansion:
lint_store.register_pre_expansion_pass(|| {
    Box::new(de13_common_patterns::de1304_your_lint_name::De1304YourLintName)
});

// Early pass:
lint_store.register_early_pass(|| {
    Box::new(de13_common_patterns::de1304_your_lint_name::De1304YourLintName)
});

// Late pass:
lint_store.register_late_pass(|_| {
    Box::new(de13_common_patterns::de1304_your_lint_name::De1304YourLintName)
});
```

### 4. Write per-lint documentation

Create `src/<category>/de<ccnn>_<snake_name>/README.md` — colocated with the implementation file for discoverability. Include:

- **Rule** — what the lint checks
- **Rationale** — why the rule exists
- **Allowed Exceptions** — any path/context exceptions
- **Examples** — forbidden and allowed code snippets
- **Guidance** — how to fix violations or suppress the lint

Then add a link in `docs/README.md` (the lint index):

```markdown
- [DE1304 - Your Lint Name](../src/de13_common_patterns/de1304_your_lint_name/README.md)
```

### 5. Add UI tests

Create a directory `tests/ui/de<ccnn>_<snake_name>/` with test fixtures. Each fixture is a pair:

- **`<case>.rs`** — Rust source that should trigger (or not trigger) the lint
- **`<case>.stderr`** — expected compiler diagnostics (empty for cases that should pass cleanly)

Naming conventions:
- `forbidden_<scenario>.rs` / `bad_<scenario>.rs` — code that should trigger the lint
- `allowed_<scenario>.rs` / `good_<scenario>.rs` — code that should pass

Example forbidden case (`forbidden_example.rs`):

```rust
// compile-flags: --crate-type=lib

pub fn example() {
    // Should trigger DE1304
    offending_code_here();
}
```

For lints that use file-path-based detection (e.g. checking if code is in `/domain/`), use a `// simulated_dir=` comment on the first line to simulate the path in UI tests:

```rust
// simulated_dir=modules/my-gear/src/domain/model.rs
// compile-flags: --crate-type=lib

pub struct Foo;
```

To generate the initial `.stderr` files, run the tests and let them fail — the test harness prints the actual compiler output, which you can capture into the `.stderr` file. Alternatively, run with `DYLINT_BLESS=1` to auto-update `.stderr` files.

### 6. Register examples in `Cargo.toml`

Each UI test fixture must be registered as a `[[example]]` in `Cargo.toml` so that `dylint_testing::ui_test_examples` discovers it:

```toml
[[example]]
name = "de1304_your_lint_name-forbidden_example"
path = "tests/ui/de1304_your_lint_name/forbidden_example.rs"

[[example]]
name = "de1304_your_lint_name-allowed_example"
path = "tests/ui/de1304_your_lint_name/allowed_example.rs"
```

The name format is `<lint_id>-<fixture_name>`. For lints that run in SDK crates, append `-sdk` to the name (the test harness uses the `-sdk` suffix to set `--crate-name` accordingly).

### 7. Run tests

```bash
# Run all lint UI tests
cd crates/cargo-gears-lints
cargo test

# Auto-update .stderr files after intentional diagnostic changes
DYLINT_BLESS=1 cargo test
```

### 8. Test against a target workspace

Before publishing, verify the new lint works against a real codebase:

```bash
# Build cargo-gears from source with the new lint
cargo run -p cargo-gears -- gears lint --dylint -p /path/to/gears-rust
```

This lets you validate that:
- The lint triggers on real violations (if any exist)
- Existing code that should pass is not flagged
- Path-based detection works with the target workspace's directory layout

### 9. Publish and adopt

`cargo-gears-lints` and `cargo-gears` release independently (each has its own
`release-plz` pipeline and toolchain - `release-plz` opens a release PR for
each automatically once commits land on `main`). Once the lint is merged and
`cargo-gears-lints` is released, `cargo-gears-core`'s `LINTS_PACKAGE_VERSION`
constant (`crates/cargo-gears-core/build.rs`) needs to be bumped to that new
version for standalone installs to pick it up - this requires its own
PR/release of `cargo-gears`.

Once that new version of `cargo-gears` is published:

1. In the **target workspace** (e.g. `gears-rust`), update `cargo-gears`:
   ```bash
   cargo install cargo-gears
   ```
2. Run `cargo gears lint --dylint` (or `make dylint`) to check for violations.
3. Fix violations, or temporarily add the rule to the `skip` list in `Gears.toml` and track clean-up:
   ```toml
   [apps.gears-rust.dev.lint.dylint]
   skip = ["de1304_your_lint_name"]
   ```
4. If the rule has configurable parameters (allow-lists, thresholds), configure them in the target workspace's `dylint.toml`.

### Checklist

- [ ] Implementation file in `src/<category>/`
- [ ] Lint and pass registered in `lib.rs`
- [ ] Per-lint README in `src/<category>/<lint>/`
- [ ] Link added to `docs/README.md`
- [ ] UI test fixtures in `tests/ui/<lint>/` (forbidden + allowed cases)
- [ ] Examples registered in `Cargo.toml`
- [ ] `cargo test` passes
- [ ] Tested against a real workspace
- [ ] Crate README table updated (this file)

## Troubleshooting

**Build fails for lint package** — Dylint rules require a specific nightly toolchain
(declared in `rust-toolchain.toml`). The build script installs it automatically via `rustup`.

**Lint not triggering** — Check that the file path matches the expected module pattern
(e.g., `*/api/rest/*`). See the per-lint README for details.

## Resources

- [Dylint documentation](https://github.com/trailofbits/dylint)
- [Clippy lint development guide](https://doc.rust-lang.org/nightly/clippy/development/index.html)

## License

Apache-2.0
