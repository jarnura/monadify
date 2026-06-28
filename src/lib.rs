#![doc = include_str!("../README.md")]
// Add other crate-level attributes if needed, e.g.:
#![deny(missing_docs)] // Enforce documentation for all public items

// Module declarations

/// Provides the Kind-based `Applicative` trait and its implementations for the `monadify` library.
pub mod applicative;
/// Provides the Kind-based `Apply` trait (an extension of `Functor`) and its implementations.
pub mod apply;
/// Defines `RcFn` and `CFnOnce` for heap-allocated, callable function wrappers.
pub mod function;
/// Provides the Kind-based `Functor` trait and its implementations.
pub mod functor;
/// Defines the `Identity` monad and its Kind marker.
pub mod identity;
/// Core infrastructure for Kind-based programming (Higher-Kinded Types), including `Kind` and `Kind1` traits,
/// and various Kind marker types (e.g., `OptionKind`).
pub mod kind_based;
/// Provides the Kind-based `Monad` and `Bind` traits and their implementations.
pub mod monad;
/// Implements `Profunctor`, `Strong`, and `Choice` traits, primarily for function types.
pub mod profunctor;
/// Contains monad transformers like `ReaderT`.
pub mod transformers;
/// Utility functions and macros, including `fn0!`, `fn1!`, etc.
pub mod utils;

/// Contains legacy (non-Kind-based, associated type-based) implementations of functional traits.
/// This module is only available when the `legacy` feature is enabled.
#[cfg(feature = "legacy")]
pub mod legacy;

// Public re-exports of core traits (now default to Kind-based versions)
pub use applicative::Applicative; // Points to applicative::kind::Applicative
pub use apply::Apply; // Points to apply::kind::Apply
pub use functor::Functor; // Points to functor::kind::Functor
pub use monad::{Bind, Monad}; // Points to monad::kind::Bind and monad::kind::Monad
pub use profunctor::{Choice, Profunctor, Strong};
pub use transformers::reader::MonadReader; // Points to transformers::reader::kind::MonadReader
pub use transformers::state::MonadState; // Points to transformers::state::kind::MonadState

// Public re-exports of key structs/types (optional, but can be convenient)
pub use function::{CFnOnce, RcFn};
pub use identity::Identity; // Points to identity::kind::Identity
pub use transformers::reader::{Reader, ReaderT}; // Points to transformers::reader::kind::ReaderT etc.
pub use transformers::state::{State, StateT}; // Points to transformers::state::kind::StateT etc.

// Re-export Kind markers and core Kind traits by default
pub use crate::identity::IdentityKind; // Changed from IdentityHKTMarker
pub use crate::transformers::reader::ReaderTKind;
pub use crate::transformers::state::StateTKind;
pub use kind_based::kind::{
    CFnOnceKind,
    Kind,
    Kind1, // Core Kind traits
    OptionKind,
    RcFnKind,
    ResultKind,
    VecKind,
}; // Changed from ReaderTHKTMarker
   // Reader alias is re-exported above.

/// Support items for the `do-notation` feature's `mdo!` macro (e.g. the
/// `MdoGuard` helper trait backing `guard(..)`). Compiled only under the
/// non-default `do-notation` feature.
#[cfg(feature = "do-notation")]
pub mod do_notation;

/// Procedural `do`-notation macro. Available only under the non-default
/// `do-notation` feature; desugars an imperative monadic block over the
/// `Bind`/`Applicative` hierarchy. Usable as `monadify::mdo!`.
///
/// The block names its Kind marker explicitly (marker inference is impossible
/// because the GAT `Of<Arg>` is not injective), terminated by `;`. Each
/// statement is one of `let …`, `pat <- expr` (bind), `guard(expr)` (filter,
/// for [`OptionKind`]/[`VecKind`] only), or a bare `expr` (sequencing),
/// followed by a trailing raw final expression.
///
/// # Example
///
/// ```rust
/// use monadify::{mdo, Applicative, OptionKind};
///
/// let r: Option<i32> = mdo! { OptionKind;
///     x <- Some(2);
///     y <- Some(3);
///     guard(x + y > 0);
///     pure(x + y)      // bare `pure(...)` resolves to OptionKind::pure, == Some(5)
/// };
/// assert_eq!(r, Some(5));
/// ```
///
/// Inside an `mdo!` block, any bare `pure(expr)` — not `::` -qualified or a
/// method call — is automatically rewritten to the block's marker's `Applicative::pure`.
///
/// # Limitations
///
/// **`CFnOnceKind` is not supported** (not `Clone`; wraps `Box<dyn FnOnce(…)>`).
/// The desugaring emits `(expr).clone()` on every monadic right-hand side,
/// which fails with `E0599` at depth ≥ 1.
/// **`RcFnKind` is supported** — it is `Clone`-able via `Rc<dyn Fn(…)>` (O(1) bump).
/// See `tests/kind/do_notation/cfn_unsupported.rs` for details.
///
/// At most **one non-`Copy` external value may be captured per `mdo!` nesting
/// level**. Because the desugaring emits nested `move` closures bound by
/// `FnMut + Clone + 'static`, referencing the same non-`Copy` captured value
/// (e.g. a [`ReaderT`], `String`, or other
/// non-`Copy` binding) at two different bind depths moves it out of the outer
/// `FnMut` and triggers `E0507`.
///
/// The bound results of monadic steps are usually `Copy` (`i32`, `bool`, …) and
/// cross nesting levels freely; the constraint applies to *external* non-`Copy`
/// values referenced inside the block. The workaround is to combine multiple
/// reads into a single tuple-returning step so only `Copy` values cross deeper
/// nesting levels. See the [`mdo`](macro@crate::mdo) macro's own documentation
/// (re-exported from `monadify-macros`) for a worked example.
#[cfg(feature = "do-notation")]
pub use monadify_macros::mdo;

/// Helper trait backing `guard(..)` inside `mdo!`. Available only under the
/// non-default `do-notation` feature.
#[cfg(feature = "do-notation")]
pub use do_notation::MdoGuard;

// Note on macros:
// Macros defined with `#[macro_export]` in submodules (like `utils.rs`) are
// automatically available at the crate root.
// So, `use monadify::fn0;` etc., should work without explicit re-export here.
// If they were not `#[macro_export]`, they would need to be re-exported like:
// pub use utils::fn0; // (if fn0 was not #[macro_export])

// Example of how to conditionally compile and export:
// #[cfg(feature = "experimental")]
// pub use experimental_apply::ExperimentalApply;
