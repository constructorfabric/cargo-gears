// Created: 2026-04-22 by Constructor Tech
// Updated: 2026-04-22 by Constructor Tech
#![allow(dead_code)]

use std::fmt;

// Negative case: the tightened receiver check re-verifies that the source
// parameter type itself implements `Error`. The impl-level gate passes
// because the target `MyError` implements `Error`, but stringifying a plain
// `u32` source doesn't destroy any chain — nothing to flag.

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyError {}

impl From<u32> for MyError {
    fn from(n: u32) -> Self {
        // Should not trigger DE1302 - to_string
        MyError(n.to_string())
    }
}

fn main() {}
