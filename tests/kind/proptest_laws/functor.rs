//! Functor law property tests for the kind-based data instances.
//!
//! Covers the **identity** and **composition** laws over the four
//! property-testable data instances (Option, Result<i32, String>, Vec,
//! Identity) — eight cells total. Companion to the example-based tests in
//! `tests/kind/functor.rs`; the shared strategies live in the parent module
//! (`super`) and are reused here, not redefined.
//!
//! Laws:
//! - identity: `map(x, |a| a) == x`
//! - composition: `map(x, |a| g(f(a))) == map(map(x, f), g)`
//!
//! The composition closures `f`/`g` are materialized from
//! `arb_linear_closure_params` via `linear_fn` (rebuilt fresh per use because
//! `CFn` is not `Clone`), using `wrapping_*` arithmetic to avoid overflow
//! panics on arbitrary `i32` inputs.

use super::{
    arb_identity_i32, arb_linear_closure_params, arb_option_i32, arb_result_i32_string,
    arb_vec_i32, linear_fn,
};
use monadify::functor::kind::Functor;
use monadify::identity::IdentityKind;
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
use proptest::prelude::*;

type TestError = String;

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Option ---

    /// Functor identity for `Option`: `map(v, id) == v`.
    #[test]
    fn option_functor_identity(v in arb_option_i32()) {
        prop_assert_eq!(OptionKind::map(v, |x| x), v);
    }

    /// Functor composition for `Option`: `map(v, g∘f) == map(map(v, f), g)`.
    #[test]
    fn option_functor_composition(
        v in arb_option_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let composed = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            OptionKind::map(v, move |x| g(f(x)))
        };
        let sequential = OptionKind::map(OptionKind::map(v, linear_fn(fa, fb)), linear_fn(ga, gb));
        prop_assert_eq!(composed, sequential);
    }

    // --- Result<i32, String> ---

    /// Functor identity for `Result<i32, String>`: `map(r, id) == r`.
    #[test]
    fn result_functor_identity(r in arb_result_i32_string()) {
        prop_assert_eq!(ResultKind::<TestError>::map(r.clone(), |x| x), r);
    }

    /// Functor composition for `Result<i32, String>`:
    /// `map(r, g∘f) == map(map(r, f), g)`.
    #[test]
    fn result_functor_composition(
        r in arb_result_i32_string(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let composed = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            ResultKind::<TestError>::map(r.clone(), move |x| g(f(x)))
        };
        let sequential = ResultKind::<TestError>::map(
            ResultKind::<TestError>::map(r, linear_fn(fa, fb)),
            linear_fn(ga, gb),
        );
        prop_assert_eq!(composed, sequential);
    }

    // --- Vec ---

    /// Functor identity for `Vec`: `map(v, id) == v`.
    #[test]
    fn vec_functor_identity(v in arb_vec_i32()) {
        prop_assert_eq!(VecKind::map(v.clone(), |x| x), v);
    }

    /// Functor composition for `Vec`: `map(v, g∘f) == map(map(v, f), g)`.
    #[test]
    fn vec_functor_composition(
        v in arb_vec_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let composed = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            VecKind::map(v.clone(), move |x| g(f(x)))
        };
        let sequential = VecKind::map(VecKind::map(v, linear_fn(fa, fb)), linear_fn(ga, gb));
        prop_assert_eq!(composed, sequential);
    }

    // --- Identity ---

    /// Functor identity for `Identity`: `map(i, id) == i`.
    #[test]
    fn identity_functor_identity(i in arb_identity_i32()) {
        prop_assert_eq!(IdentityKind::map(i.clone(), |x| x), i);
    }

    /// Functor composition for `Identity`: `map(i, g∘f) == map(map(i, f), g)`.
    #[test]
    fn identity_functor_composition(
        i in arb_identity_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let composed = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            IdentityKind::map(i.clone(), move |x| g(f(x)))
        };
        let sequential =
            IdentityKind::map(IdentityKind::map(i, linear_fn(fa, fb)), linear_fn(ga, gb));
        prop_assert_eq!(composed, sequential);
    }
}
