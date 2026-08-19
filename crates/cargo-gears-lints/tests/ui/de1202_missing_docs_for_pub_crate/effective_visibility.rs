// simulated_dir=/workspace/src/lib.rs
// compile-flags: --crate-type=lib

// Publicly reexported APIs belong to rustc's `missing_docs`, not DE1202.
/// Implementation module for externally reexported APIs.
mod externally_reexported {
    // Should not trigger DE1202 - missing docs for externally reexported function
    pub fn external_function() {}

    // Should not trigger DE1202 - missing docs for externally reexported struct
    pub struct ExternalStruct {
        // Should not trigger DE1202 - missing docs for externally reexported field
        pub field: usize,
    }

    impl ExternalStruct {
        // Should not trigger DE1202 - missing docs for externally reexported method
        pub fn method(&self) {}
    }

    // Should not trigger DE1202 - missing docs for externally reexported enum
    pub enum ExternalEnum {
        // Should not trigger DE1202 - missing docs for externally reexported variant and field
        Variant { field: usize },
    }

    // Should not trigger DE1202 - missing docs for externally reexported trait
    pub trait ExternalTrait {
        // Should not trigger DE1202 - missing docs for externally reexported trait item
        fn method(&self);
    }
}

pub use externally_reexported::{ExternalEnum, ExternalStruct, ExternalTrait, external_function};

// APIs with no crate-public path are outside DE1202's scope.
/// Crate-public parent used to hide a nested implementation module.
mod private_parent {
    mod private_apis {
        // Should not trigger DE1202 - missing docs without a crate-public path
        pub fn private_function() {}
        // Should not trigger DE1202 - missing docs without a crate-public path
        pub(crate) fn nominally_crate_visible_but_unreachable() {}

        // Should not trigger DE1202 - missing docs without a crate-public path
        pub struct PrivateStruct {
            // Should not trigger DE1202 - missing docs without a crate-public path
            pub field: usize,
        }

        impl PrivateStruct {
            // Should not trigger DE1202 - missing docs without a crate-public path
            pub fn method(&self) {}
        }

        // Should not trigger DE1202 - missing docs without a crate-public path
        pub enum PrivateEnum {
            // Should not trigger DE1202 - missing docs without a crate-public path
            Variant { field: usize },
        }

        // Should not trigger DE1202 - missing docs without a crate-public path
        pub trait PrivateTrait {
            // Should not trigger DE1202 - missing docs without a crate-public path
            fn method(&self);
        }
    }
}

// Every API below is accessible throughout the crate through a crate-only
// reexport, so DE1202 should require documentation for it and its members.
/// Implementation module for crate-reexported APIs.
mod crate_reexported {
    // Should trigger DE1202 - missing docs for crate-only reexported function
    pub fn crate_function() {}

    // Should trigger DE1202 - missing docs for crate-only reexported struct
    pub struct CrateStruct {
        // Should trigger DE1202 - missing docs for crate-only reexported field
        pub field: usize,
    }

    impl CrateStruct {
        // Should trigger DE1202 - missing docs for crate-only reexported method
        pub fn method(&self) {}
    }

    // Should trigger DE1202 - missing docs for crate-only reexported enum
    pub enum CrateEnum {
        // Should trigger DE1202 - missing docs for crate-only reexported variant and field
        Variant { field: usize },
    }

    // Should trigger DE1202 - missing docs for crate-only reexported trait
    pub trait CrateTrait {
        // Should trigger DE1202 - missing docs for crate-only reexported trait item
        fn method(&self);
    }
}

pub(crate) use crate_reexported::{CrateEnum, CrateStruct, CrateTrait, crate_function};
