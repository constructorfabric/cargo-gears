// simulated_dir=/workspace/tests/integration.rs
// compile-flags: --crate-type=lib

pub(crate) fn test_helper() {}

#[cfg(test)]
mod tests {
    pub(crate) fn nested_helper() {}
}

#[test]
pub(crate) fn test_case() {}
