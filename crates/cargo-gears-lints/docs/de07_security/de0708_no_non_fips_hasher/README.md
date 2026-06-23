# DE0708: No Non-FIPS Hasher Imports

## What it does

Prohibits imports of non-FIPS-validated hash crates (`sha2`, `sha1`, `md5`) outside an explicit allow-list.

## Why is this bad?

These crates use pure-Rust RustCrypto implementations that are not FIPS-validated. They may be present in the dependency graph via transitives, but new *direct* usage should not creep in without review.

## Allow-list (configurable)

Exceptions are centralized in the consuming workspace's `dylint.toml` under the `[cargo-gears-lints]` table. This fits a phased FIPS rollout: paths are permitted now and removed as milestones land.

```toml
[cargo-gears-lints]
hasher_allowed_paths = ["libs/legacy-checksum/", "gears/foo/src/etag.rs"]
```

Entries are substring-matched against forward-slash-normalized file paths. The list is empty by default (deny everywhere). For a one-off reviewed usage you can alternatively add `#[allow(de0708_no_non_fips_hasher)]` at the import site.

## Example

### Bad

```rust
// Triggers DE0708
use sha2::{Digest, Sha256};
```

### Good

```rust
// Route through the validated crypto provider,
// or add to the allow-list for a documented non-cryptographic use.
use std::hash::{DefaultHasher, Hasher};
```

## Configuration

This lint is configured to **deny** by default.

## References

- Your project's FIPS dependency policy
- `deny-fips.toml` (or equivalent) dependency bans
