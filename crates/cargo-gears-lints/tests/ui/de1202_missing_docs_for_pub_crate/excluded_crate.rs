// simulated_dir=/workspace/src/lib.rs
// compile-flags: --crate-type=lib

// This undocumented API is allowed because the UI harness configures
// `excluded-crate`, which normalizes to this fixture's `excluded_crate` name.
pub(crate) fn legacy_undocumented_api() {}
