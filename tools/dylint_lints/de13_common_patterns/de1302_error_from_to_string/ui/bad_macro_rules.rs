// Created: 2026-04-22 by Constructor Tech
// Updated: 2026-04-22 by Constructor Tech
#![allow(dead_code)]

use std::fmt;

// Positive case: `macro_rules!` expansions that contain `.to_string()` on
// the source error are NOT silenced. A user-defined macro that stringifies
// an error is just as much a chain-loss pattern as hand-written code, so
// the lint still fires through the expansion.

#[derive(Debug)]
struct DatabaseError(String);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DatabaseError {}

#[derive(Debug)]
struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

macro_rules! render_err {
    ($e:expr) => {
        // Should trigger DE1302 - to_string
        $e.to_string()
    };
}

impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        AppError(render_err!(e))
    }
}

fn main() {}
