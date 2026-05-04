# 12. Versioning and Compatibility

## Table of Contents

1. [Purpose](#purpose)
2. [CLI Versioning](#cli-versioning)
3. [Manifest Schema Versioning](#manifest-schema-versioning)
4. [JSON Output Versioning](#json-output-versioning)
5. [Template Versioning](#template-versioning)
6. [Deprecation Policy](#deprecation-policy)
7. [Migration Paths](#migration-paths)
8. [Compatibility Matrix](#compatibility-matrix)

## Purpose

Versioning ensures that developers, CI pipelines, and LLMs can depend on stable behavior across CLI releases. Breaking
changes are explicit, announced, and accompanied by migration tooling.

## CLI Versioning

The CLI follows Semantic Versioning (SemVer):

```text
MAJOR.MINOR.PATCH
```

| Component | Incremented When |
|---|---|
| MAJOR | Breaking changes to commands, flags, exit codes, or JSON output schema |
| MINOR | New commands, new flags, new JSON fields, new manifest schema features |
| PATCH | Bug fixes, documentation updates, performance improvements |

### Pre-1.0 Convention

While the CLI is pre-1.0 (`0.x.y`):

- MINOR bumps may include breaking changes (SemVer pre-1.0 convention).
- PATCH bumps are always non-breaking.
- Breaking changes are documented in the changelog.

### Version Display

```bash
cargo cyberfabric --version
# cyberfabric 0.1.0 (abc1234 2025-05-01)
```

## Manifest Schema Versioning

The manifest `schema_version` is a positive integer:

```toml
schema_version = 1
```

### Schema Evolution Rules

| Change Type | Schema Version Bump? | Example |
|---|---|---|
| Add optional field | No | Adding `extra_features` to run policy |
| Add required field | Yes | Adding a new mandatory section |
| Rename field | Yes | `crate` -> `package` |
| Remove field | Yes | Removing deprecated `source = "remote"` |
| Change field type | Yes | Changing `outputs` from string to array |
| Change enum variants | Yes | Removing a variant from `source` |
| Add enum variant | No | Adding `private-registry` to `source` |

### Forward Compatibility

- The CLI refuses to load a manifest with a `schema_version` higher than it supports.
- Error message includes the maximum supported version and suggests upgrading the CLI.

### Backward Compatibility

- The CLI can load manifests with older `schema_version` values.
- Internal migration logic normalizes older schemas to the current representation.
- `manifest validate` warns about deprecated fields in older schemas.

### Migration Between Schema Versions

When a new schema version is released:

```bash
cargo cyberfabric manifest migrate [--from <version>] [--to <version>] [--manifest <path>] [--dry-run]
```

This command:

1. Reads the manifest with the old schema.
2. Applies migration rules.
3. Writes the updated manifest.
4. Prints a diff of changes.

`--dry-run` shows the diff without writing.

## JSON Output Versioning

JSON output schema is implicitly versioned by the CLI version:

- **Within a major version**: fields are never removed or renamed; types are never changed.
- **New fields may be added** in any release (consumers must tolerate unknown fields).
- **Array ordering is deterministic** (sorted by name/key).

There is no explicit JSON schema version field in output. The CLI version is the version.

For CI pipelines that need strict schema validation, the CLI version should be pinned.

## Template Versioning

Templates are versioned alongside the CLI. The default Git template repository uses branches and tags:

| Branch/Tag | Purpose |
|---|---|
| `main` | Latest templates for the current CLI release |
| `v1` | Templates for manifest schema version 1 |
| `cli-0.1.0` | Templates pinned to CLI version 0.1.0 |

### Template-CLI Compatibility

When the CLI is installed from a release, it pins the default template branch to the matching CLI version range:

```text
CLI 0.1.x -> templates main (pre-1.0, moving target)
CLI 1.0.x -> templates v1
CLI 1.1.x -> templates v1 (minor releases use same major template branch)
CLI 2.0.x -> templates v2
```

Template version mismatches are detected during generation:

```text
error[E010]: template requires schema_version 2, but CLI supports up to 1
  = help: upgrade the CLI with 'cargo install --git https://github.com/cyberfabric/cf-cli'
```

## Deprecation Policy

### Deprecation Lifecycle

1. **Announce**: deprecated feature is documented in changelog and release notes.
2. **Warn**: the CLI prints a warning when the deprecated feature is used.
3. **Remove**: the feature is removed in the next major version.

### Minimum Warning Period

- Deprecated flags: warned for at least one minor release cycle.
- Deprecated manifest fields: warned for at least one schema version.
- Deprecated commands: warned for at least two minor release cycles.

### Warning Format

```text
warning: --config is deprecated for this command; use --env and --app instead
  = note: --config will be removed in v2.0.0
  = help: run 'cargo cyberfabric generate manifest --from-config config/quickstart.yml' to migrate
```

## Migration Paths

### Config-Centric to Manifest-First

```bash
# Generate manifest from existing config
cargo cyberfabric generate manifest --from-config config/quickstart.yml --env dev --app quickstart

# Verify the migration
cargo cyberfabric manifest validate

# Strip metadata from config (optional)
cargo cyberfabric generate manifest --from-config config/quickstart.yml --strip-metadata

# Run with manifest instead of --config
cargo cyberfabric run --env dev --app quickstart
```

### CLI Version Upgrade

When upgrading across major versions:

1. Read the changelog for breaking changes.
2. Run `manifest validate` to check manifest compatibility.
3. Run `manifest migrate` if schema version changed.
4. Run `lint` and `test` to verify workspace health.
5. Update CI pipeline if JSON output schema changed.

## Compatibility Matrix

| CLI Version | Schema Version | Template Branch | JSON Output |
|---|---|---|---|
| 0.0.x | N/A (config-only) | main | Unstable |
| 0.1.x | 1 | main | Unstable (pre-1.0) |
| 1.0.x | 1 | v1 | Stable v1 |
| 1.x.x | 1 | v1 | Stable v1 (additive only) |
| 2.0.x | 2 | v2 | Stable v2 |
