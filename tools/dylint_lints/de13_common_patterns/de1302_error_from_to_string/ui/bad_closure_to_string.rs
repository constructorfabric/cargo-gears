// Created: 2026-04-21 by Constructor Tech
// Updated: 2026-04-21 by Constructor Tech
#![allow(dead_code)]

use std::fmt;

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

// Closure inside `fn from` body — the lint must descend into the closure
// using its own typeck context.
impl From<DatabaseError> for AppError {
    fn from(e: DatabaseError) -> Self {
        let render = |err: &DatabaseError| {
            // Should trigger DE1302 - to_string
            err.to_string()
        };
        AppError(render(&e))
    }
}

fn main() {}
