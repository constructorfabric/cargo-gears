# Dylint Lints

Custom [dylint](https://github.com/trailofbits/dylint) linters enforcing architectural patterns, layer separation, and REST API conventions.

These rules are compiled at build time (behind the `dylint-rules` feature of `gears-cli-core`) and embedded into the CLI binary. At runtime, `cargo gears lint --dylint` extracts and runs them against the target workspace.

## Available Lints

### Domain Layer (DE01xx)
- **DE0101** No Serde in Domain
- **DE0102** No ToSchema in Domain
- **DE0104** No API DTO in Domain

### API Layer (DE02xx)
- **DE0201** DTOs Only in API Rest Folder
- **DE0202** DTOs Not Referenced Outside API
- **DE0203** DTOs Must Use API DTO
- **DE0204** DTOs Must Have ToSchema Derive

### Domain Layer (DE03xx)
- **DE0301** No Infra in Domain
- **DE0308** No HTTP Types in Domain

### Client Layer (DE05xx)
- **DE0503** Plugin Client Suffix
- **DE0504** Client Versioning

### Security (DE07xx)
- **DE0706** No Direct SQLx
- **DE0707** Drop Zeroize (sensitive types)

### REST Conventions (DE08xx)
- **DE0801** API Endpoint Must Have Version
- **DE0802** Use OData Extension Methods
- **DE0803** API Snake Case

### GTS (DE09xx)
- **DE0901** GTS String Pattern Validator
- **DE0902** No `schema_for!` on GTS Structs

### Testing (DE11xx)
- **DE1101** Tests in Separate Files

### Common Patterns (DE13xx)
- **DE1301** No Print/Debug Macros
- **DE1302** No `.to_string()` in Error From impls
- **DE1303** No Primitive Type Aliases

## Project Structure

```text
tools/dylint_lints/
├── lint_utils/               # Shared helper crate
├── de01_domain_layer/        # One crate per lint rule
│   ├── de0101_.../
│   │   ├── src/lib.rs        # Lint implementation
│   │   ├── ui/               # UI test fixtures (.rs + .stderr)
│   │   └── Cargo.toml
│   └── ...
├── de02_api_layer/
├── ...
├── Cargo.toml                # Workspace manifest
└── rust-toolchain.toml       # Nightly channel for dylint
```

Each lint crate contains a `ui/` directory with test fixtures: `.rs` files with code that should trigger (or not) the lint, and `.stderr` files with the expected compiler diagnostics.

## Usage

```bash
cargo gears lint --dylint
cargo gears lint --all
```

See the per-lint `README.md` files for details on what each rule checks and examples.

## Troubleshooting

**Build fails for lint workspace** — Dylint rules require a specific nightly toolchain
(declared in `rust-toolchain.toml`). The build script installs it automatically via `rustup`.

**Lint not triggering** — Check that the file path matches the expected module pattern
(e.g., `*/api/rest/*`). See the per-lint README for details.

## Resources

- [Dylint documentation](https://github.com/trailofbits/dylint)
- [Clippy lint development guide](https://doc.rust-lang.org/nightly/clippy/development/index.html)

## License

Apache-2.0
