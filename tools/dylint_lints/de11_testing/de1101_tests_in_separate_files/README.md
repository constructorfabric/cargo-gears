<!-- Created: 2026-04-07 by Constructor Tech -->
# DE1101: Tests Must Be In Separate Files

## What it does

This lint forbids inline test code inside production Rust files. It scans production `.rs` files (not in `tests/`, not `*_tests.rs`) and reports violations.

## Why

Keeping tests in separate files makes it easier to:

- filter test files out when counting lines of code
- navigate the codebase for both humans and LLMs because files stay smaller
- keep production logic and test code separated by file type

## Triggers (error)

### Inline `#[test]` in a production file

```rust
// handler.rs
fn handle() {}

#[test]  // ❌ DE1101: test code must be moved to a separate test file
fn test_handle() {}
```

### Inline `#[cfg(test)] mod ... { }` block

```rust
// handler.rs
#[cfg(test)]  // ❌ DE1101: test code must be moved to a separate test file
mod tests {
    #[test]
    fn test_handle() {}
}
```

### `#[path]` pointing to wrong file

```rust
// handler.rs
#[cfg(test)]
#[path = "dto_tests.rs"]  // ❌ DE1101: must reference handler_tests.rs
mod tests;
```

## Does not trigger (ok)

### Out-of-line mod — any name, no `#[path]`

```rust
// handler.rs
#[cfg(test)]
mod handler_tests;  // ✅ ok
```

```rust
// handler.rs
#[cfg(test)]
mod tests;  // ✅ ok — without #[path], any module name is accepted
```

### `#[path]` pointing to correct file

```rust
// handler.rs
#[cfg(test)]
#[path = "handler_tests.rs"]
mod tests;  // ✅ ok
```

### Test files — not scanned

```rust
// handler_tests.rs — test file, lint does not scan it
#[test]
fn test_handle() {}  // ✅ ok — *_tests.rs files are skipped
```

```rust
// tests/integration.rs — integration test, lint does not scan it
#[test]
fn e2e() {}  // ✅ ok — files in tests/ are skipped
```

### `#[cfg(test)]` on items (not modules)

```rust
// handler.rs
#[cfg(test)]
impl Handler {  // ✅ does not trigger (this is not #[test] and not an inline mod)
    fn test_helper() {}
}
```

## Not scanned at all

- Files ending with `_tests.rs`
- Files under `tests/` directories
- Modules listed in `excluded_paths` in `dylint.toml`

## Configuration

Exclusions are configured in `dylint.toml` at the workspace root:

```toml
[de1101_tests_in_separate_files]
excluded_paths = [
    "libs/modkit",
    "modules/mini-chat",
    # ...
]
```

Each entry is a module path prefix (e.g. `libs/modkit`, `modules/system/oagw`). Remove entries one by one as modules are migrated.

## Relation to Rust Book Guidance

The Rust Book recommends keeping `#[cfg(test)] mod tests { ... }` inline in the same file as the production code ([ch11-03](https://doc.rust-lang.org/book/ch11-03-test-organization.html)). That is valid and idiomatic Rust.

This lint intentionally adopts a stricter repository-level policy. The cost is deviation from the most common Rust convention. The benefit is stronger separation between production and test code, easier LOC filtering, smaller production files, and simpler navigation for both humans and LLMs.
