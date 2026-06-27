pub mod applicative;
pub mod apply;
// RcFn / RcFnKind tests — RED phase (types not yet implemented).
// Will compile once the implementer adds RcFn, RcFnKind, and lift_a1_rc.
pub mod cfn_clonable;
pub mod functor;
pub mod identity;
// The `kind` submodule intentionally mirrors the crate's `kind_based::kind` path.
#[allow(clippy::module_inception)]
pub mod kind;
pub mod monad;
pub mod proptest_laws;
pub mod transformers;

// Do-notation tests: compiled only when the `do-notation` feature is enabled.
#[cfg(feature = "do-notation")]
pub mod do_notation;
