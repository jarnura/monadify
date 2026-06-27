//! Do-notation integration tests for the kind-based monad instances.
//!
//! This module is compiled **only** when the `do-notation` feature is enabled.
//! Every submodule inherits that gate.
//!
//! Run with:
//! ```text
//! cargo test --features do-notation
//! ```

#![cfg(feature = "do-notation")]
// The equivalence tests call `.clone()` on `Option<i32>` to mirror the macro's
// own `(expr).clone()` desugaring, ensuring structural parity with non-Copy
// instances (Result, Vec, Identity, ReaderT).  The redundant-clone lint is
// intentional here for cross-instance test symmetry.
#![allow(clippy::clone_on_copy)]

pub mod cfn_unsupported;
pub mod identity;
pub mod laws;
pub mod option;
pub mod reader;
pub mod result;
pub mod vec;
