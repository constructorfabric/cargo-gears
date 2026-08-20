# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5](https://github.com/constructorfabric/cargo-gears/compare/cargo-gears-v0.0.4...cargo-gears-v0.0.5) - 2026-08-20

### Added

- *(tools)* add check-version subcommand for any tool (by @Artifizer)
- add --check-version top-level option (by @Artifizer) - #107
- *(ls)* add packages subcommand for directory-based crate discovery (by @Artifizer)
- *(ls)* add features and deps subcommands for Cargo.toml inspection (by @Artifizer)
- *(ls gears)* add --include-rdeps flag (by @Artifizer)
- *(ls gears)* add --scope-dirs directory filter (by @Artifizer)
- *(ls gears)* add --filter regex pattern (by @Artifizer)

### Fixed

- use take() to avoid partial move in ToolsArgs::run (by @Artifizer) - #107
- parse version output from both stdout and stderr (by @Artifizer) - #107
- enforce timeout for tool --version subprocess (by @Artifizer) - #107
- resolve clippy warnings and fix --include-rdeps guard in ls packages (by @Artifizer) - #107

### Other

- resolve conflict with main (Deploy now manifest-based) (by @Artifizer) - #107
- address PR #107 review feedback (by @Artifizer) - #107
- add regression tests for scope validation and check-version fixes (by @Artifizer) - #107
- use let...else for spawn error handling in tools.rs (by @Artifizer) - #107
- Merge branch 'main' into feat/ls-gears-enhancements (by @Artifizer) - #107
- add coverage for new ls subcommands, filter/rdeps, and tools check-version (by @Artifizer) - #107

### Contributors

* @Artifizer

## [0.0.4](https://github.com/constructorfabric/cargo-gears/compare/cargo-gears-v0.0.3...cargo-gears-v0.0.4) - 2026-08-19

### Other

- Align deploy command to match build and run manifest-based behaviour (by @maurolacy) - #105
- Add --locked flag to lint, cov and test commands (by @maurolacy) - #99
- Add --locked flag to build and run commands (by @maurolacy) - #99

### Contributors

* @maurolacy

## [0.0.3](https://github.com/constructorfabric/cargo-gears/compare/cargo-gears-v0.0.2...cargo-gears-v0.0.3) - 2026-08-14

### Other

- Add --list param to cargo gears lint (by @maurolacy) - #75

### Contributors

* @maurolacy
