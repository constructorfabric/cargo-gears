## General

Always prefer `cargo clippy` over cargo check. Only run it with `--all-features` if something related with lints have been touched.

Always format the code with `cargo fmt` and run the test suite with `cargo test` before finalizing if any rust code was
touched.

Always prefer `cargo add` over manually editing `Cargo.toml`.

Always prefer enums over strings when there's a clear set of valid values.

When updating the behaviour of any flag or option, update the [SKILL.md](./SKILL.md) file.
