# DE1302 — No `.to_string()` in Error Conversion Impls

## TL;DR

Calling `.to_string()` on an error inside `impl From<E> for MyErr` (or
`TryFrom`) collapses a typed source into a string. After that, callers
can't `.source()` to the cause, can't `.downcast_ref::<E>()` to recover
the type, and tracing/alerting tools see only the message — not the
chain. DE1302 fires when the receiver of `.to_string()` is the source
parameter itself (or a `TryFrom::Error` assoc type that implements
`Error`). Fix it by storing the source in the variant — usually with
thiserror's `#[from]` / `#[source]` / `#[error(transparent)]`, or via
`Box<dyn Error + Send + Sync + 'static>` for opaque buckets. See
**[Fixing a flagged site](#fixing-a-flagged-site)** for the full
decision flow.

## Rule

This lint flags `.to_string()` calls inside `fn from()` (or `fn try_from()`)
bodies when they appear in:

- `impl From<X> for Y`
- `impl TryFrom<X> for Y`

where the source type `X`, the target type `Y`, or the `TryFrom::Error`
associated type implements `std::error::Error`. Both syntactic forms are
caught:

- Method-call form: `e.to_string()`
- UFCS form: `ToString::to_string(&e)` and `<E as ToString>::to_string(&e)`

Closure bodies are also walked (e.g. `.map(|e| e.to_string())` inside a From
body).

## Rationale

`From` impls on error types exist primarily to power the `?` operator: when
a function returns `Result<T, AppError>` and the caller writes
`db_query()?`, Rust desugars that to `db_query().map_err(AppError::from)`.
Whatever your `From<DatabaseError> for AppError` does is what every `?` in
the codebase will do.

Calling `e.to_string()` on an error inside that conversion converts the
error to a `String` and **discards the original**. The result is a new error
that:

- Has no `.source()` — the chain is broken; callers can't follow back to the
  root cause.
- Cannot be `.downcast_ref::<ConcreteErr>()` to recover the underlying type.
- Loses structured metadata (error codes, fields, retry hints, etc.).
- Is missing the information `tracing`, alerting, and bug-report tooling
  rely on.

For most conversions, you have better options that preserve the chain:

- **`thiserror`'s `#[from]`** auto-derives a `From` impl that stores the
  source error directly. The variant's first field becomes the source.
- **`#[error(transparent)]`** delegates `Display` / `source()` to a wrapped
  inner error — the variant disappears from messages but is still reachable
  via `.source()`.
- **`#[source]`** marks a field as the chain source without auto-generating
  the `From` impl. Use this when you want a custom variant constructor
  (`Internal { msg: String, #[source] source: SomeError }`) but still want
  `.source()` to work.
- **Box the source**: `Internal(Box<dyn std::error::Error + Send + Sync + 'static>)`
  with a manual `From` that calls `.into()` (no stringification). The
  `Send + Sync + 'static` bound is what async runtimes (tokio, async-std)
  and error-reporting libraries need to move errors across tasks.
- **Match-and-forward**: pattern-match the source variants and map them to
  shape-preserving target variants.

## Fixing a flagged site

When DE1302 fires, work through these in order — the first match is
almost always the right answer.

1. **Can the target variant hold the source directly (one field, exact
   type)?** Use thiserror's `#[from]` and delete the manual `From` impl.
   ```rust
   #[derive(thiserror::Error, Debug)]
   enum AppError {
       #[error("db: {0}")]
       Database(#[from] DatabaseError),
   }
   ```

2. **Is the variant a pure forward (no extra fields, Display delegates)?**
   Add `#[error(transparent)]` to make it invisible in messages while
   keeping the chain via `.source()`.
   ```rust
   #[derive(thiserror::Error, Debug)]
   enum AppError {
       #[error(transparent)]
       Database(#[from] DatabaseError),
   }
   ```

3. **Need a custom variant shape (extra fields like `msg`, `code`,
   `retry_after`) but still want `.source()` to work?** Use `#[source]`
   on the source field and write the `From` impl manually.
   ```rust
   #[derive(thiserror::Error, Debug)]
   enum AppError {
       #[error("db op '{op}' failed: {source}")]
       Database { op: String, #[source] source: DatabaseError },
   }
   ```

4. **One `Internal` bucket that needs to absorb many source types?**
   Replace `Internal(String)` with
   `Internal(Box<dyn Error + Send + Sync + 'static>)` and use `.into()`
   in each From impl — never `.to_string()`.
   ```rust
   #[derive(thiserror::Error, Debug)]
   enum AppError {
       #[error(transparent)]
       Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
   }

   impl From<anyhow::Error> for AppError {
       fn from(e: anyhow::Error) -> Self { AppError::Internal(e.into()) }
   }
   ```

5. **Is the source already an enum whose variants align with the target
   enum's variants?** Match-and-forward — explicit but preserves shape.
   ```rust
   impl From<DbError> for AppError {
       fn from(e: DbError) -> Self {
           match e {
               DbError::NotFound(id) => AppError::NotFound(id),
               DbError::Conflict(c)  => AppError::Conflict(c),
               other                 => AppError::Internal(Box::new(other)),
           }
       }
   }
   ```

6. **None of the above fit** (e.g. an SDK boundary that exposes only an
   opaque `Internal(String)` and changing the SDK is out of scope).
   Silence at the impl with a TODO so the debt is grep-able. See
   **[Configuration](#configuration)** for the exact pattern.

If the answer is "none of these fit because the source isn't actually
an `Error`-implementing type" — DE1302 won't fire. The receiver-tightening
gate skips non-Error sources. See `good_from_u32.rs` in the UI tests.

## Gating

The lint is type-driven, not name-based. It only walks a body when **at
least one of**:

- `source_ty` implements `std::error::Error`, **or**
- `target_ty` implements `std::error::Error`, **or**
- (TryFrom only) the `type Error` associated type implements
  `std::error::Error`.

Inside the body, `.to_string()` is only flagged when:

- The receiver type equals the source parameter type **and** the source type
  implements `Error`, **or**
- The receiver type equals the `TryFrom::Error` associated type.

Any other receiver — `&str`, `String`, `Uuid`, an unrelated error fetched for
logging — is left alone. This prevents false positives like `impl From<u32>
for MyErr` flagging `n.to_string()`.

## Examples

### Bad — chain destroyed

```rust
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError::Internal(e.to_string()) // chain lost
    }
}
```

```rust
impl TryFrom<DatabaseError> for AppError {
    type Error = ConversionError;

    fn try_from(e: DatabaseError) -> Result<Self, Self::Error> {
        Ok(AppError::Internal(e.to_string())) // chain lost
    }
}
```

```rust
// UFCS form — same problem.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError::Internal(ToString::to_string(&e))
    }
}
```

```rust
// Inside a closure inside a From body — also caught.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        let render = |x: &DatabaseError| x.to_string();
        AppError::Internal(render(&e))
    }
}
```

### Good — chain preserved

```rust
// thiserror #[from] — the cleanest path.
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("internal: {0}")]
    Internal(String),
}
```

```rust
// Manual From that stores the source directly.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError::Database(e) // chain preserved via #[error(transparent)] / source()
    }
}
```

```rust
// Custom variant shape with `#[source]` — when `#[from]` doesn't fit but you
// still want `.source()` to walk into the underlying error.
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("internal: {msg}")]
    Internal {
        msg: String,
        #[source]
        source: DatabaseError,
    },
}

impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError::Internal {
            msg: "database operation failed".into(),
            source: e,
        }
    }
}
```

```rust
// Boxed source variant — keeps a single Internal bucket while preserving
// `.source()`. anyhow::Error already implements Into<Box<dyn Error + Send + Sync>>.
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error(transparent)]
    Unexpected(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unexpected(e.into()) // no .to_string(), chain preserved
    }
}
```

### Not flagged (intentional)

```rust
// Source type is not an Error — stringifying a u32 has no chain to lose.
impl From<u32> for AppError {
    fn from(n: u32) -> Self {
        AppError::Internal(n.to_string()) // OK
    }
}
```

```rust
// Stringifying an unrelated error inside a From body for logging context.
// The returned error preserves the actual source.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        let other_err = build_some_unrelated_error();
        AppError {
            context: other_err.to_string(), // OK — recv is not the source type
            source: e,
        }
    }
}
```

```rust
// Format-arg macros (format!, write!, panic!, tracing::*) are NOT caught —
// they construct strings via Display::fmt, not ToString::to_string. See the
// "Known gaps" section.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError::Internal(format!("db: {e}")) // NOT flagged today
    }
}
```

## Macro behavior

| Source                           | Treatment                |
| -------------------------------- | ------------------------ |
| `macro_rules!` / bang proc-macro | **Checked.** A macro that expands to `.to_string()` on the source error is just as much a chain-loss pattern as inline code. |
| Attribute macros (`#[attr]`)     | Skipped — assumed to be third-party codegen the user can't easily change. |
| Derive macros                    | Skipped — same reason. |
| Compiler desugarings (`?`, etc.) | Skipped — synthetic, not user intent. |

## Known gaps

- **`format!("...{err}")` / `write!` / `panic!`** — these macros destroy the
  chain identically (they go through `Display::fmt` rather than
  `ToString::to_string`), but DE1302 doesn't see them. Catching this needs
  `format_args!`-level inspection; tracked as a follow-up.
- **Logging-only stringification** — `tracing::error!(error = %err)` inside a
  conversion body is not flagged; the receiver-type tightening rules out
  side-channel `.to_string()` calls.

## Configuration

The lint level is **deny** by default. To silence a known site, prefer fixing
the conversion to preserve the chain. When the underlying error type's shape
truly forbids that (e.g. an SDK boundary that exposes only opaque
`Internal(String)`), silence the impl explicitly.

### Preferred: `#[expect]` with `reason`

`#[expect(lint, reason = "...")]` (Rust 1.81+) is a stricter form of
`#[allow]`: if the underlying violation ever stops firing — for example,
because someone refactors `Internal(String)` into
`Internal(Box<dyn Error + Send + Sync>)` — the compiler **warns** that the
expectation didn't fire. The silence ages out automatically as soon as the
real fix lands.

```rust
#[expect(
    unknown_lints,
    de1302_error_from_to_string,
    reason = "Internal only carries a String; extend to hold \
              Box<dyn Error + Send + Sync + 'static> so .source() returns \
              the original error, then remove this expect."
)]
impl From<SomeError> for MyError {
    fn from(e: SomeError) -> Self {
        Self::Internal(e.to_string())
    }
}
```

If several adjacent conversions share the same rationale, group them in a
small inner module so the attribute appears once:

```rust
#![expect(
    unknown_lints,
    de1302_error_from_to_string,
    reason = "MyError::Internal collapses many sources into a String. \
              Extend to a boxed-source variant in a follow-up PR, then \
              drop this expect."
)]
mod error_froms {
    use super::MyError;

    impl From<A> for MyError { fn from(e: A) -> Self { Self::Internal(e.to_string()) } }
    impl From<B> for MyError { fn from(e: B) -> Self { Self::Internal(e.to_string()) } }
    impl From<C> for MyError { fn from(e: C) -> Self { Self::Internal(e.to_string()) } }
}
```

The `reason` field is also machine-readable — `cargo clippy --message-format=json`
exposes it, so a debt-audit script can list every silenced site with its
explanation in one pass.

### Acceptable: `#[allow]` with a `TODO(DE1302)` comment

For sites that pre-date the `#[expect]` recommendation, or when you
specifically don't want the auto-warn behaviour, the older pattern still
works:

```rust
// TODO(DE1302): `Internal` only carries a String; extend to hold a boxed
// source so `.source()` returns the original error, then remove this allow.
#[allow(unknown_lints, de1302_error_from_to_string)]
impl From<SomeError> for MyError {
    fn from(e: SomeError) -> Self {
        Self::Internal(e.to_string())
    }
}
```

The trade-off: an `#[allow]` doesn't notice when the underlying fix lands, so
it can rot in the codebase indefinitely. Migrate to `#[expect]` whenever you
touch a silenced site.

### Why `unknown_lints` is in the list

The dylint driver is only loaded by `make dylint`. Plain `cargo check` /
`cargo clippy` doesn't know about `de1302_error_from_to_string` and would
otherwise reject the attribute as an unknown lint name. Adding
`unknown_lints` to the same `#[expect]` / `#[allow]` makes the attribute
parse cleanly under both toolchains.

## UI Tests

This lint includes UI tests covering:

- Method-call positive case (`bad_from_to_string.rs`)
- UFCS positive case (`bad_ufcs_to_string.rs`)
- Closure body recursion (`bad_closure_to_string.rs`)
- `TryFrom` with Error source (`bad_tryfrom_to_string.rs`)
- `TryFrom` whose only Error is the assoc type (`bad_tryfrom_assoc_error.rs`)
- `macro_rules!` expansion still flagged (`bad_macro_rules.rs`)
- `From<u32>` with non-Error source — not flagged (`good_from_u32.rs`)
- Stringifying an unrelated error inside a From body — not flagged
  (`good_unrelated_error.rs`)
- `#[from]`-style and source-preserving conversions — not flagged
  (`good_from_preserve.rs`)

## See Also

- [thiserror](https://crates.io/crates/thiserror) — derive macro for typed
  errors with `#[from]` / `#[source]` / `#[error(transparent)]`.
- [anyhow](https://crates.io/crates/anyhow) — opaque error type that
  preserves chain via `.source()` and `Into<Box<dyn Error + Send + Sync>>`.
- [`std::error::Error::source`](https://doc.rust-lang.org/std/error/trait.Error.html#method.source) — the chain navigation method.
- [Error handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
