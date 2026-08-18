// simulated_dir=/workspace/src/lib.rs
// compile-flags: --crate-type=lib

/// Internal APIs.
pub(crate) mod internal {
    /// Retry count.
    pub(crate) const RETRIES: usize = 3;

    /// Retries the operation.
    pub(crate) fn retry() {}

    /// Exposes an operation through this crate-public module.
    pub fn visible_through_crate_module() {}

    /// Retry policy.
    pub(crate) struct Policy {
        /// Number of attempts.
        pub(crate) attempts: usize,
        private: usize,
    }

    /// Execution state.
    pub(crate) enum State {
        /// Ready to execute.
        Ready,
        /// Execution failed.
        Failed {
            /// Failure code.
            code: u16,
        },
    }

    /// Executes a policy.
    pub(crate) trait Execute {
        /// Executes the operation.
        fn execute(&self);
    }

    impl Policy {
        /// Returns the attempt count.
        pub(crate) fn attempts(&self) -> usize {
            self.attempts
        }
    }

    impl Execute for Policy {
        fn execute(&self) {}
    }
}

// Externally public APIs are checked by rustc's missing_docs, not DE1202.
pub fn exported() {}
pub struct Exported;
pub struct MixedVisibility {
    /// A crate-public field on an exported type.
    pub(crate) internal: usize,
}

// Other restricted and private visibilities are outside DE1202's scope.
mod private_module {
    pub(super) fn parent_visible() {}
}
fn private() {}

#[doc(hidden)]
pub(crate) fn deliberately_hidden() {}

#[doc(hidden)]
pub(crate) mod deliberately_hidden_module {
    pub fn hidden_child() {}
}

#[allow(de1202_missing_docs_for_pub_crate)]
pub(crate) fn locally_allowed() {}

#[allow(de1202_missing_docs_for_pub_crate)]
pub(crate) mod allowed_module {
    pub fn allowed_child() {}
}

/// Exercises member-level lint allowances.
pub(crate) struct AllowedMembers {
    #[allow(de1202_missing_docs_for_pub_crate)]
    pub(crate) allowed_field: usize,
}

impl AllowedMembers {
    #[allow(de1202_missing_docs_for_pub_crate)]
    pub(crate) fn allowed_method() {}
}

/// Exercises variant-level lint allowances.
pub(crate) enum AllowedVariants {
    #[allow(de1202_missing_docs_for_pub_crate)]
    AllowedVariant,
}

#[expect(de1202_missing_docs_for_pub_crate)]
pub(crate) fn expected_missing_docs() {}

#[doc = include_str!("documented_and_out_of_scope.rs")]
pub(crate) fn macro_documented() {}

#[doc = concat!("Expanded", " documentation.")]
pub(crate) fn nonempty_expanded_docs() {}

macro_rules! generate_internal {
    () => {
        pub(crate) fn generated_without_source_docs() {}
    };
}

generate_internal!();

pub struct PublicType;

/// Implementation details used to exercise inherent-method visibility.
pub(crate) mod implementation_details {
    struct PrivateType;

    impl PrivateType {
        pub fn private_method() {}
    }

    impl crate::PublicType {
        pub fn externally_visible_method() {}
    }
}
