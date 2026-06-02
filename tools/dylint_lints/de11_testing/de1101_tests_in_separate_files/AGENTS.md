<!-- Created: 2026-04-07 by Constructor Tech -->
# DE1101: Enabling the lint for a new module

## Quick start

Three steps to enable DE1101 for a module that's currently excluded:

### 1. Remove the module from `dylint.toml` exclusions

Open `dylint.toml` in the workspace root and delete the module's line from `[de1101_tests_in_separate_files].excluded_paths`:

```toml
[de1101_tests_in_separate_files]
excluded_paths = [
    # ...
    # "modules/my-module",   ← delete this line
    # ...
]
```

### 2. Extract inline tests

Run the extraction script from the workspace root:

```sh
python3 dylint_lints/de11_testing/de1101_tests_in_separate_files/extract_tests.py .
```

The script will:
- Find all `#[cfg(test)] mod tests { ... }` inline blocks in `.rs` files
- Extract each test body into a `<stem>_tests.rs` file next to the source
- Replace the inline block with `#[cfg(test)] #[path = "<stem>_tests.rs"] mod <stem>_tests;`
- Print `WARN` for `#[cfg(test)]` on individual items (structs, impls) — fix those manually
- Skip `tests/`, `ui/`, `target/`, `.git/` directories automatically

### 3. Format and verify

```sh
cargo fmt --all
make check
```

`make check` runs fmt, clippy, dylint, and all tests. If it passes, you're done.

## Common issues after extraction

| Problem | Cause | Fix |
|---------|-------|-----|
| `unused import: super::*` | Test doesn't use parent module items | Remove `use super::*;` line |
| Clippy lint fires in test file | Test code was hidden behind `#[cfg(test)]`, now visible to clippy | Add `#![allow(clippy::the_lint)]` at top of test file |
| `#[cfg(test)]` on individual items | Script only extracts `mod` blocks, not standalone items | Move manually or leave as-is (these don't trigger DE1101) |

## What the lint enforces

For any `.rs` file in scope:
- No inline `#[test]` or `#[cfg(test)] mod ... { }` blocks
- Test module name must be `{source_stem}_tests` (e.g. `handler.rs` → `mod handler_tests;`)
- If `#[path = "..."]` is used, it must point to `{source_stem}_tests.rs`
