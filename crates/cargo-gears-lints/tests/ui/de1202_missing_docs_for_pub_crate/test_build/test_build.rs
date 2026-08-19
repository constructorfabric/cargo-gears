// simulated_dir=/workspace/src/lib.rs

#[cfg(any(test, doctest))]
pub(crate) mod tests {
    pub(crate) fn nested_helper() {}
}

#[test]
pub(crate) fn test_case() {}
