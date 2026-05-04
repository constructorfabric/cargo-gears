# 10. Security

## Table of Contents

1. [Purpose](#purpose)
2. [Secret Handling](#secret-handling)
3. [Environment Variable Expansion](#environment-variable-expansion)
4. [Generated File Safety](#generated-file-safety)
5. [Docker Image Safety](#docker-image-safety)
6. [Network Safety](#network-safety)
7. [Filesystem Safety](#filesystem-safety)

## Purpose

The CLI enforces secure-by-default behavior so that developers do not accidentally embed secrets in generated files,
commit credentials to version control, or expose sensitive data through build artifacts. Security is a design
constraint, not an afterthought.

## Secret Handling

### Principle

Secrets never appear as literal values in:

- Generated Rust source files.
- Generated `Cargo.toml` files.
- Manifest (`Cyberfabric.toml`).
- CLI output (stdout/stderr).
- Docker build context (unless explicitly referenced in config).

### Where Secrets Live

| Secret Type | Location | Mechanism |
|---|---|---|
| Database passwords | Runtime config YAML | `${DB_PASSWORD}` env-var expansion |
| API keys | Runtime config YAML | `${API_KEY}` env-var expansion |
| Registry credentials | Environment | Standard Docker/Helm credential flows |
| Config path | Generated server | `CF_CLI_CONFIG` env var at runtime |

### Output Redaction

The CLI redacts potentially sensitive values in two contexts:

1. **`manifest render --verbose`**: when verbose mode includes runtime config preview alongside the generation model,
   fields named `password`, `secret`, `token`, `key`, or `api_key` are masked as `***REDACTED***`. Without `--verbose`,
   `manifest render` only outputs manifest-derived data (module list, features, policies) which does not contain secrets.
2. **Diagnostic output**: error messages and suggestions never include runtime config values that could contain secrets.

Values using `${VAR_NAME}` syntax are always shown as-is (not expanded) because they are placeholder references, not
actual secrets.

The `--show-secrets` flag disables redaction for debugging. It produces a warning when used.

> **Scope note:** The manifest (`Cyberfabric.toml`) itself does not contain secret values by design. Redaction applies
> when the CLI reads or previews runtime config files, not to the manifest schema.

## Environment Variable Expansion

Runtime config files support `${VAR_NAME}` syntax for environment variable references:

```yaml
database:
  servers:
    primary:
      password: "${DB_PASSWORD}"
      dsn: "postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:5432/appdb"
```

### Expansion Rules

- Expansion happens at **runtime** in the generated server, not at CLI generation time.
- The CLI does not expand `${...}` tokens in config files. It passes them through as-is.
- The CLI validates that `${...}` syntax is well-formed during `manifest validate`.
- Missing environment variables at runtime cause the server to fail with a clear error.

### Why Not CLI-Time Expansion

Expanding secrets at CLI time would embed them in generated files, which are committed to `.cyberfabric/` (even though
it is gitignored, it is still on disk). Runtime expansion keeps secrets in memory only.

## Generated File Safety

### `.cyberfabric/` Directory

- Listed in `.gitignore` by `init` and `generate workspace`.
- Contains only derived output: `Cargo.toml`, `config.toml`, `main.rs`.
- Does not contain config files, secrets, or credentials.
- Safe to delete and regenerate.

### Generated `main.rs`

- Does not embed config paths, secrets, or environment-specific values.
- Reads `CF_CLI_CONFIG` from the environment at runtime.
- Uses only stable, auditable code patterns.

### Generated `Cargo.toml`

- Contains dependency metadata only (package names, versions, paths, features).
- Does not contain registry credentials or private URLs.

## Docker Image Safety

### Build Context

- The CLI ensures the Docker build context is the workspace root.
- Only files inside the workspace root can be copied into the image.
- `.env` files, `.git/`, and `target/` are excluded by the generated `.dockerignore`.

### Generated `.dockerignore`

```text
target/
.cyberfabric/
.git/
.env
.env.*
*.pem
*.key
```

### Config in Image

The runtime config is copied into the image at a known path. The config file should use `${...}` expansion for
secrets so that the image does not contain literal credentials. The developer is responsible for providing environment
variables at container runtime.

### No Secret Build Args

The CLI does not pass secrets as Docker build args. Build args are logged in Docker's build output, making them
unsuitable for secrets. The Dockerfile should use multi-stage builds and runtime environment variables instead.

## Network Safety

### Registry Access

- The CLI accesses `crates.io` for module resolution and docs cache.
- The CLI accesses the configured Git template repository for scaffolding.
- No other network access occurs without explicit user action.
- Network access can be disabled by providing `--local-path` for templates and using only local modules.

### TLS

All network access uses HTTPS. HTTP fallback is not supported.

### Cache Security

- Registry cache is stored in the OS temp directory (`cyberfabric-docs-cache/<registry>/`).
- Cache contents are crate source code, not secrets.
- `docs --clean` removes the cache.

## Filesystem Safety

### Write Scope

The CLI only writes to:

- The workspace root (when generating workspace).
- `modules/` (when generating modules).
- `config/` (when generating config).
- `.cyberfabric/` (when generating server projects).
- `Cyberfabric.toml` (when managing the manifest).
- The workspace `Cargo.toml` (when updating workspace members/dependencies).
- The OS temp directory (for docs cache).

The CLI never writes outside the workspace root except for the docs cache and tool installation (`tools` command).

### Permission Model

The CLI runs with the user's permissions. It does not require elevated privileges. The `tools` command may invoke
`rustup` which installs to the user's home directory.

### Overwrite Protection

- `generate workspace` fails if the target is a non-empty directory (unless `--force`).
- `generate module` fails if the module directory already exists.
- `generate manifest` fails if `Cyberfabric.toml` already exists (unless `--force`).
- `config mod add` uses upsert semantics (safe merge, not destructive overwrite).
