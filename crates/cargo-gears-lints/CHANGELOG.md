# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5](https://github.com/constructorfabric/cargo-gears/compare/cargo-gears-lints-v0.0.4...cargo-gears-lints-v0.0.5) - 2026-08-20

### Added

- *(dylint)* require docs for crate-public APIs (by @fdlockgraf)

### Fixed

- *(dylint)* use effective visibility for DE1202 (by @fdlockgraf)
- *(dylint)* correct DE1202 visibility and scopes (by @fdlockgraf)

### Contributors

* @fdlockgraf

## [0.0.4](https://github.com/constructorfabric/cargo-gears/compare/cargo-gears-lints-v0.0.3...cargo-gears-lints-v0.0.4) - 2026-08-14

### Added

- *(lints)* Reject hard-coded GTS ID prefixes

### Other

- Set up release-plz for automated versioning and publishing (by @striped-zebra-dev) - #91
- Address review: require custom method to be terminal, tighten verb rules (by @striped-zebra-dev) - #90
- Allow AIP-136 custom-method colon suffix in DE0801 (by @striped-zebra-dev) - #90
- Fix failing DE0904 in passing (by @maurolacy) - #75
- Merge branch 'main' into dev (by @maurolacy) - #75
- make existing domain UI dylint tests compatible with DE0309 (by @maurolacy) - #83
- Port DE0309 must_have_domain_model lint from gears-rust (by @maurolacy) - #83
- Merge branch 'main' into dylint-no_hardcoded_gts_prefix (by @Artifizer) - #81

### Contributors

* @striped-zebra-dev
* @maurolacy
* @Artifizer
