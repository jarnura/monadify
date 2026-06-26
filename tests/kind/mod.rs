pub mod applicative;
pub mod apply;
pub mod functor;
pub mod identity;
// The `kind` submodule intentionally mirrors the crate's `kind_based::kind` path.
#[allow(clippy::module_inception)]
pub mod kind;
pub mod monad;
pub mod proptest_laws;
pub mod transformers;
