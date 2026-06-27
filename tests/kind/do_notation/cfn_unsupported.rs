//! `CFn` / `CFnOnce` are intentionally unsupported by `mdo!`.
//!
//! # Why `mdo!` cannot work with `CFnKind` / `CFnOnceKind`
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
//! `CFn<A, B>` wraps `Box<dyn Fn(A) -> B + 'static>`. `Box<dyn Fn(…)>` is
//! **not `Clone`** (trait objects cannot be cloned generically), so `CFn` itself
//! is not `Clone` either. Calling `.clone()` on any `CFn` value — which the
//! desugaring always does — fails to compile:
//!
//! ```text
//! error[E0599]: the method `clone` exists for struct `CFn<i32, i32>`,
//!               but its trait bounds were not satisfied
//!   = note: the following trait bounds were not satisfied:
//!           `dyn Fn(i32) -> i32: Sized`  →  `Box<dyn Fn(i32) -> i32>: Clone`
//!           `dyn Fn(i32) -> i32: Clone`  →  `Box<dyn Fn(i32) -> i32>: Clone`
//! ```
//!
//! This error appears at **depth ≥ 1** — even a single `<-` bind block fails —
//! because the very first emitted `(cfn_expr).clone()` requires `Clone`.
//! The situation is identical for `CFnOnce<A, B>` / `CFnOnceKind<R>`.
//!
//! The error was empirically confirmed during the Phase 0 spike (scratchpad
//! `cfn_probe`) and re-confirmed during Phase 3 using a disposable probe crate
//! that path-depends on the real `monadify` library with `--features do-notation`.
//!
//! # Future lift
//!
//! Lifting this restriction requires an `Rc`-backed, clone-able function wrapper,
//! e.g.:
//!
//! ```text
//! pub struct RcFn<A, B>(pub Rc<dyn Fn(A) -> B + 'static>);
//! ```
//!
//! Cloning an `RcFn` bumps the reference count (cheap), exactly as `ReaderT`
//! does today (`ReaderT` wraps `Rc<dyn Fn(R) -> M::Of<A>>` and is `#[derive(Clone)]`).
//! Adding an `RcFnKind` marker that implements the full `Functor → Bind` hierarchy
//! would allow `mdo!` do-blocks of arbitrary depth for that wrapper type. That is
//! a separate, additive design decision with no impact on the current trait surface.
//!
//! Until that wrapper exists, `mdo!` blocks over `CFnKind`/`CFnOnceKind` are
//! **out of scope** by design. Users needing a function monad in a do-block
//! should use `ReaderTKind<R, IdentityKind>` (already `Clone`) as an alternative.

/// Anchors the module documentation above. No runtime assertions are made here
/// because no `CFn`/`CFnOnce` `mdo!` block can compile; the exclusion is verified
/// at the compile-error level and described in this module's doc.
#[test]
fn cfn_cfnonce_mdo_is_documented_unsupported() {
    // CFn and CFnOnce are intentionally excluded from mdo! do-blocks.
    // See module documentation for the full explanation and the empirically
    // confirmed error (E0599 / Clone bound not satisfied at depth >= 1).
    //
    // This placeholder test exists so the module appears in `cargo test --features
    // do-notation` output, anchoring the documentation.
}
