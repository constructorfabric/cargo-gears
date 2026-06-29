# Dylint Lints

Custom [dylint](https://github.com/trailofbits/dylint) linters enforcing architectural patterns, layer separation, and REST API conventions.

These rules are compiled by `cargo-gears-core`'s build script when the CLI is built with `dylint-rules`. During local development, the build uses the sibling `crates/cargo-gears-lints` path; in installed builds, it resolves this package from the Cargo registry at the CLI version. The resulting Dylint library is embedded into the CLI.

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
│   │   ├── de0101_....rs
│   │   └── ...
│   ├── de02_api_layer/
│   └── ...
├── docs/                     # Per-lint documentation, grouped by category
│   ├── README.md             # Index linking to each lint README
│   ├── de01_domain_layer/
│   │   ├── de0101_.../
│   │   │   └── README.md
│   │   └── ...
│   └── ...
├── tests/
│   └── ui/
│       ├── de0101_.../       # UI test fixtures (.rs + .stderr)
│       └── ...
├── Cargo.toml                # Publishable package manifest
└── rust-toolchain.toml       # Nightly channel for dylint
```

Each lint implementation lives in `src/<category>/<lint>.rs`. Per-lint documentation lives in `docs/<category>/<lint>/README.md`, with [docs/README.md](docs/README.md) as the index. UI fixtures live in `tests/ui/<lint>/`: `.rs` files contain code that should trigger (or not) the lint, and `.stderr` files contain the expected compiler diagnostics.

## Usage

```bash
cargo gears lint --dylint
cargo gears lint --all
```

See [docs/README.md](docs/README.md) for links to each lint's detailed documentation.

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
