//! `CFnOnceKind` is intentionally unsupported by `mdo!`; `RcFnKind` IS supported.
//!
//! # Why `mdo!` cannot work with `CFnOnceKind`
//!
//! The `mdo!` desugaring emits **`(expr).clone()`** for every monadic right-hand
//! side before passing it to `Bind::bind`. This clone is required because
//! `Bind::bind`'s continuation parameter is:
//!
//! ```text
//! func: impl FnMut(A) -> Self::Of<B> + Clone + 'static
//! ```
//!
//! The `FnMut + Clone` bound exists because instances such as `VecKind` invoke
//! the continuation *once per element* (via `flat_map`); the macro must therefore
//! clone the captured RHS before each invocation.
//!
//! `CFnOnce<A, B>` wraps `Box<dyn FnOnce(A) -> B + 'static>`. A `FnOnce` is
//! move-once by definition and `Box<dyn FnOnce(…)>` is **not `Clone`**, so
//! `CFnOnce` cannot be `Clone` either — and making it clone-able would be
//! semantically contradictory (cloning a once-only function would let it run
//! more than once). Calling `.clone()` on a `CFnOnce` value — which the
//! desugaring always does — therefore fails to compile at **depth ≥ 1**.
//! `CFnOnceKind<R>` consequently cannot flow through an `mdo!` block.
//!
//! # `RcFnKind` IS supported (the former restriction is lifted)
//!
//! The function monad *does* work in `mdo!` — via [`RcFnKind`], the `Rc`-backed,
//! clone-able function wrapper:
//!
//! ```text
//! pub struct RcFn<A, B>(pub Rc<dyn Fn(A) -> B + 'static>);  // Clone via Rc refcount
//! ```
//!
//! Cloning an `RcFn` bumps the reference count (cheap), exactly as `ReaderT`
//! does. `RcFnKind` implements the full `Functor → Apply → Applicative → Bind →
//! Monad` hierarchy, so `mdo!` do-blocks of arbitrary depth compile for it (see
//! the `rcfn_kind_mdo` tests in `tests/kind/cfn_clonable.rs`). `RcFnKind` is the
//! sole multi-call function marker since `CFnKind`/`CFn` were removed in 0.2.0.
//!
//! Users needing a function monad in a do-block should use `RcFnKind` (or
//! `ReaderTKind<R, IdentityKind>`). `CFnOnceKind` remains out of scope by design.

/// Anchors the module documentation above. No runtime assertions are made here
/// because no `CFnOnce` `mdo!` block can compile; the exclusion is verified at
/// the compile-error level and described in this module's doc.
#[test]
fn cfnonce_mdo_is_documented_unsupported() {
    // CFnOnce is intentionally excluded from mdo! do-blocks (move-once + not
    // Clone). RcFnKind, by contrast, IS supported — see tests/kind/cfn_clonable.rs.
    // This placeholder test anchors the documentation in the
    // `cargo test --features do-notation` output.
}
