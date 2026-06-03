# Dylint Lints

Custom [dylint](https://github.com/trailofbits/dylint) linters enforcing architectural patterns, layer separation, and REST API conventions.

These rules are compiled at build time (behind the `dylint-rules` feature of `gears-cli-core`) and embedded into the CLI binary. At runtime, `cargo gears lint --dylint` extracts and runs them against the target workspace.

## Available Lints

### DE01xx — Domain Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0101 | No Serde in Domain | Structs/enums in `/domain/` must not derive `Serialize`/`Deserialize` |
| DE0102 | No ToSchema in Domain | Structs/enums in `/domain/` must not derive `utoipa::ToSchema` |
| DE0104 | No API DTO in Domain | Structs/enums in `/domain/` must not use the `api_dto` macro |

### DE02xx — API Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0201 | DTOs Only in API Rest | Types with `*Dto` suffix must live in `*/api/rest/*.rs` |
| DE0202 | DTOs Not Outside API | Contract, domain, and infra modules must not import DTO types |
| DE0203 | DTOs Must Use `api_dto` | DTO types in `api/rest` must use `#[modkit_macros::api_dto(...)]` |
| DE0204 | DTOs Must Have ToSchema | DTO types must derive `utoipa::ToSchema` for OpenAPI docs |

### DE03xx — Domain Layer (infra / HTTP boundaries)

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0301 | No Infra in Domain | Domain modules must not import infra crates (`modkit_db`, `sea_orm`, etc.) |
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

### DE08xx — REST API Conventions

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0801 | API Endpoint Version | Endpoints must follow `/{service}/v{N}/{resource}` (kebab-case) |
| DE0802 | Use OData Ext | OData query params must use `OperationBuilderODataExt` methods |
| DE0803 | API Snake Case | API DTOs must use `snake_case` in serde `rename_all`/`rename` attrs |

### DE09xx — GTS Layer

| Rule | Name | What it enforces |
|------|------|------------------|
| DE0901 | GTS String Pattern | *(disabled — hardcoded vendor allowlist, needs configuration)* |
| DE0902 | No `schema_for!` on GTS | GTS structs must use `gts_schema_with_refs_as_string()`, not `schema_for!` |

### DE11xx — Testing

| Rule | Name | What it enforces |
|------|------|------------------|
| DE1101 | Tests in Separate Files | Inline test blocks must be extracted to `*_tests.rs` companion files |

### DE13xx — Common Patterns

| Rule | Name | What it enforces |
|------|------|------------------|
| DE1301 | No Print Macros | `println!`/`eprintln!`/`dbg!` forbidden in production code |
| DE1302 | No `.to_string()` in From | Error `From` impls must not call `.to_string()` (use `.into()`) |
| DE1303 | No Primitive Type Alias | `pub type X = Uuid` etc. must be newtypes for type safety |

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
