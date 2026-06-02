// Created: 2026-04-22 by Constructor Tech
// Updated: 2026-04-22 by Constructor Tech
#![allow(dead_code)]

use std::fmt;

// Positive case for the TryFrom::Error gate + tightened receiver check:
// source and target are both plain non-error types, but the associated
// `type Error = MyErr` implements `std::error::Error`. The body takes a
// pre-existing `MyErr` whose `.source()` carries a real `ParseIntError`
// chain, then stringifies it while building a new `MyErr` — dropping the
// `ParseIntError` cause.

#[derive(Debug)]
struct MyErr {
    msg: String,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl fmt::Display for MyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for MyErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|s| s as _)
    }
}

fn parse_strict(s: &str) -> Result<u32, MyErr> {
    s.parse::<u32>().map_err(|e| MyErr {
        msg: format!("parse failed for {s:?}"),
        source: Some(Box::new(e)),
    })
}

struct PlainData(u32);
struct PlainOutput(u32);

impl TryFrom<PlainData> for PlainOutput {
    type Error = MyErr;

    fn try_from(value: PlainData) -> Result<Self, Self::Error> {
        if value.0 == 0 {
            // `inner` carries a real `.source()` chain (ParseIntError).
            let inner = parse_strict("not a number").unwrap_err();
            return Err(MyErr {
                // Should trigger DE1302 - to_string
                msg: inner.to_string(),
                source: None,
            });
        }
        Ok(PlainOutput(value.0))
    }
}

fn main() {}
