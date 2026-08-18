// simulated_dir=/workspace/src/lib.rs
// compile-flags: --crate-type=lib

// Should trigger DE1202 - missing docs on pub(crate) module
pub(crate) mod internal {
    // Should trigger DE1202 - missing docs on pub(crate) constant
    pub(crate) const RETRIES: usize = 3;

    // Should trigger DE1202 - missing docs on pub(crate) function
    pub(crate) fn retry() {}

    // Should trigger DE1202 - missing docs on effectively crate-public function
    pub fn visible_through_crate_module() {}

    #[doc = ""]
    // Should trigger DE1202 - missing docs when doc attribute is empty
    pub(crate) fn empty_docs() {}

    // Should trigger DE1202 - missing docs on pub(crate) struct
    pub(crate) struct Policy {
        // Should trigger DE1202 - missing docs on pub(crate) field
        pub(crate) attempts: usize,
        private: usize,
    }

    // Should trigger DE1202 - missing docs on pub(crate) enum
    pub(crate) enum State {
        // Should trigger DE1202 - missing docs on enum variant
        Ready,
        /// A documented variant.
        Failed {
            // Should trigger DE1202 - missing docs on named variant field
            code: u16,
        },
    }

    // Should trigger DE1202 - missing docs on pub(crate) trait
    pub(crate) trait Execute {
        // Should trigger DE1202 - missing docs on trait item
        fn execute(&self);
    }

    impl Policy {
        // Should trigger DE1202 - missing docs on pub(crate) associated item
        pub(crate) fn attempts(&self) -> usize {
            self.attempts
        }
    }
}

pub struct Exported {
    // Should trigger DE1202 - missing docs on pub(crate) field of a pub struct
    pub(crate) internal: usize,
}

#[rustfmt::skip]
// Should trigger DE1202 - missing docs on pub(in crate) function
pub(in crate) fn visible_in_crate() {}

// Should trigger DE1202 - missing docs on pub(crate) type with public method
pub(crate) struct CratePublicType;

impl CratePublicType {
    // Should trigger DE1202 - missing docs on effectively crate-public associated item
    pub fn crate_public_method() {}
}

unsafe extern "C" {
    // Should trigger DE1202 - missing docs on pub(crate) foreign function
    pub(crate) fn crate_public_foreign_function();

    // Should trigger DE1202 - missing docs on pub(crate) foreign static
    pub(crate) static CRATE_PUBLIC_FOREIGN_STATIC: i32;
}

#[doc = concat!()]
// Should trigger DE1202 - missing docs when expanded docs are empty
pub(crate) fn empty_expanded_docs() {}

mod private_parent {
    // Should trigger DE1202 - missing docs on explicit pub(crate) under a private module
    pub(crate) fn explicitly_crate_visible() {}
}

fn main() {}
