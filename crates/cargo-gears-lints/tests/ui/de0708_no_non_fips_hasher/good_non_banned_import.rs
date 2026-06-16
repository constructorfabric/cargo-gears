#![allow(unused_imports, dead_code)]

// Should not trigger DE0708 - std::hash is not a banned crate
use std::hash::{DefaultHasher, Hasher};

fn main() {}
