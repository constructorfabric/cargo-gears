// Created: 2026-04-22 by Constructor Tech
// Updated: 2026-04-22 by Constructor Tech
#![allow(dead_code)]

use std::fmt;

// Negative case: the tightened receiver check only flags `.to_string()` when
// the receiver is the source parameter type itself (or the `TryFrom::Error`
// assoc type). Stringifying an *unrelated* error inside a From body — e.g.
// for logging or to include a sibling error's message in a constructed
// variant — is intentionally not flagged.

#[derive(Debug)]
struct DatabaseError(String);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DatabaseError {}

#[derive(Debug)]
struct OtherError(String);

impl fmt::Display for OtherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for OtherError {}

#[derive(Debug)]
struct AppError {
    source: DatabaseError,
    context: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

// The From body stringifies `other`, which is an Error but NOT the source
// type (`DatabaseError`). The real source `e` is preserved in `source`, so
// the chain is intact. The tightened lint does not flag this.
fn build_other() -> OtherError {
    OtherError("sibling".into())
}

impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        let other = build_other();
        AppError {
            context: other.to_string(), // Should not trigger DE1302 - to_string
            source: e,
        }
    }
}

fn main() {}
