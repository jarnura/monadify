//! Property-based (`proptest`) law-test harness for the kind-based data instances.
//!
//! Companion to the example-based tests in the sibling modules
//! (`functor`, `apply`, `applicative`, `monad`). This module provides the
//! reusable `proptest` strategies and closure generators shared by the
//! law-family modules added in the next phase, plus a single smoke test that
//! proves the harness compiles and runs under both the default and `legacy`
//! feature matrices.
//!
//! Design reference:
//! `.a5c/runs/01KW2RTB9JS5H05EYB7Z9P1YYY/artifacts/property-testing-design.md`.

use monadify::function::RcFn;
use monadify::identity::Identity;
use proptest::prelude::*;

pub mod applicative;
pub mod apply;
pub mod functor;
pub mod monad;
pub mod rcfn;

// --- Arbitrary strategies (reusable across the law-family modules) ---

/// `Option<i32>` exercising both the `Some` and `None` arms.
pub fn arb_option_i32() -> impl Strategy<Value = Option<i32>> {
    any::<Option<i32>>()
}

/// `Result<i32, String>` exercising both the `Ok` and `Err` arms with
/// arbitrary error strings.
pub fn arb_result_i32_string() -> impl Strategy<Value = Result<i32, String>> {
    prop_oneof![any::<i32>().prop_map(Ok), ".*".prop_map(Err)]
}

/// `Vec<i32>` with a **bounded** length `0..=32` to keep cartesian / `flat_map`
/// blow-up and runtime bounded for the Apply/Monad laws.
pub fn arb_vec_i32() -> impl Strategy<Value = Vec<i32>> {
    prop::collection::vec(any::<i32>(), 0..=32)
}

/// `Identity<i32>` wrapping an arbitrary `i32`.
pub fn arb_identity_i32() -> impl Strategy<Value = Identity<i32>> {
    any::<i32>().prop_map(Identity)
}

/// Slope/intercept `(a, b)` parameters used to materialize a deterministic
/// linear closure `move |x| x.wrapping_mul(a).wrapping_add(b)`.
///
/// Closures are not `Arbitrary` and `CFn` is **not `Clone`**, so we never
/// generate a closure value directly. Instead we generate scalar parameters and
/// rebuild a fresh closure (and a fresh `CFn`) at each use site.
pub fn arb_linear_closure_params() -> impl Strategy<Value = (i32, i32)> {
    (any::<i32>(), any::<i32>())
}

// --- Closure builders (rebuilt per use because `CFn` is not `Clone`) ---

/// Build a plain `FnMut(i32) -> i32 + Clone + 'static` from generated params.
///
/// `wrapping_*` arithmetic avoids overflow panics on arbitrary `i32` inputs.
pub fn linear_fn(a: i32, b: i32) -> impl FnMut(i32) -> i32 + Clone + 'static {
    move |x: i32| x.wrapping_mul(a).wrapping_add(b)
}

/// Fresh `RcFn<i32, i32>` for Applicative `apply` (function-in-container) laws.
///
/// Rebuilt per use site; `RcFn` is `Clone` (O(1) Rc bump) so it can also be
/// cloned cheaply when needed by cartesian `Vec` apply.
pub fn linear_cfn(a: i32, b: i32) -> RcFn<i32, i32> {
    RcFn::new(move |x: i32| x.wrapping_mul(a).wrapping_add(b))
}

// --- Smoke test: proves the harness compiles and runs ---

use monadify::functor::kind::Functor;
use monadify::kind_based::kind::OptionKind;

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Functor identity law for `Option`: `map(v, id) == v`.
    #[test]
    fn smoke_option_functor_identity(v in arb_option_i32()) {
        prop_assert_eq!(OptionKind::map(v, |x| x), v);
    }
}
