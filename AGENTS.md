## General

Always prefer `cargo clippy` over cargo check. Do not run with `--all-features` as we are in a process of migration and `dylint-rules` feature from `cyberware-cli-core` member is failing.

Always format the code with `cargo fmt` and run the test suite with `cargo test` before finalizing if any rust code was
touched.

Always prefer `cargo add` over manually editing `Cargo.toml`.

Always prefer enums over strings when there's a clear set of valid values.

When updating the behaviour of any flag or option, update the [SKILL.md](./SKILL.md) file.
