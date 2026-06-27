//! Apply (`<*>`) law property tests for the kind-based data instances.
//!
//! Covers the **composition** law over the three property-testable Apply
//! instances (Option, Result<i32, String>, Identity) — three cells total.
//! `Vec` is excluded: its `apply` is cartesian and must *reuse* (clone) each
//! wrapped `CFn` across elements, and `CFn` is **not `Clone`** (see the gap
//! matrix). `CFn`/`CFnOnce`/`ReaderT` are function-typed and remain
//! example-only. Companion to the example-based tests in `tests/kind/apply.rs`;
//! the shared strategies live in the parent module (`super`) and are reused
//! here, not redefined.
//!
//! ## Formulation (read me)
//!
//! The Apply **composition** law is canonically
//! `apply(apply(apply(pure(compose), u), v), w) == apply(u, apply(v, w))`
//! with `compose = |f| |g| |x| f(g(x))`.
//!
//! The literal left-hand side **cannot be constructed** with this crate's API:
//! `pure(compose)` would place `compose` in the container as a
//! `CFn<CFn<B,C>, CFn<CFn<A,B>, CFn<A,C>>>`. Building it requires the middle
//! `|g| ...` closure (which must be `Fn`, because `CFn::new` requires `Fn`) to
//! **move** its captured `CFn` `f` into the returned closure — that consumes a
//! capture, making the closure `FnOnce`, not `Fn`. The usual escape hatch
//! (clone `f` inside) is unavailable because **`CFn` is not `Clone`**.
//!
//! So we assert the **strongest expressible equivalent**: the right-associated
//! side `u <*> (v <*> w)` — fully constructible here as
//! `apply(apply(w, v), u)` (recall this crate spells `g <*> x` as
//! `apply(x, g)`) — checked against the **ground-truth semantics** `g(f(x))`.
//! For a lawful Apply the canonical left-hand side
//! `pure(compose) <*> u <*> v <*> w` equals the right side `u <*> (v <*> w)`
//! by the law, and both provably reduce to exactly this ground truth, so
//! asserting `(u <*> (v <*> w)) == ground_truth` verifies the composition law
//! without needing the non-constructible `pure(compose)`.
//!
//! The function containers `u`/`v` are themselves generated (both the
//! present/absent arm and the wrapped linear closure), so this exercises the
//! container's short-circuit/error-precedence behaviour as well as the
//! functional composition. Linear closures are materialized from
//! `arb_linear_closure_params` via `linear_fn`/`linear_cfn` (rebuilt fresh per
//! use because `CFn` is not `Clone`), using `wrapping_*` arithmetic to avoid
//! overflow panics on arbitrary `i32` inputs.

use super::{
    arb_identity_i32, arb_linear_closure_params, arb_result_i32_string, linear_cfn, linear_fn,
};
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::{OptionKind, ResultKind};
use proptest::prelude::*;

type TestError = String;

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Option ---

    /// Apply composition for `Option`:
    /// `u <*> (v <*> w) == fmap (g ∘ f) w`, i.e.
    /// `apply(apply(w, v), u)` equals `g(f(x))` lifted into `Option`, with the
    /// `None` arms short-circuiting. `v` carries `f`, `u` carries `g`; both the
    /// presence arm and the linear function are generated.
    #[test]
    fn option_apply_composition(
        w in any::<Option<i32>>(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
        f_present in any::<bool>(),
        g_present in any::<bool>(),
    ) {
        let v: Option<RcFn<i32, i32>> = if f_present { Some(linear_cfn(fa, fb)) } else { None };
        let u: Option<RcFn<i32, i32>> = if g_present { Some(linear_cfn(ga, gb)) } else { None };

        // LHS: u <*> (v <*> w) == apply(apply(w, v), u)
        let lhs = OptionKind::apply(OptionKind::apply(w, v), u);

        // Ground truth: present only when w, v and u are all present.
        let rhs: Option<i32> = match (w, f_present, g_present) {
            (Some(x), true, true) => {
                let mut f = linear_fn(fa, fb);
                let mut g = linear_fn(ga, gb);
                Some(g(f(x)))
            }
            _ => None,
        };

        prop_assert_eq!(lhs, rhs);
    }

    // --- Result<i32, String> ---

    /// Apply composition for `Result<i32, String>`:
    /// `apply(apply(w, v), u)` equals `g(f(x))` lifted into `Result`, with the
    /// `Err` arms short-circuiting in `w`-then-`v`-then-`u` precedence (matching
    /// this crate's `and_then`/`map` based `apply`). `v` carries `f`, `u`
    /// carries `g`; presence arm, error string and linear function are generated.
    #[test]
    fn result_apply_composition(
        w in arb_result_i32_string(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
        f_ok in any::<bool>(),
        g_ok in any::<bool>(),
        ev in ".*",
        eu in ".*",
    ) {
        let v: Result<RcFn<i32, i32>, TestError> =
            if f_ok { Ok(linear_cfn(fa, fb)) } else { Err(ev.clone()) };
        let u: Result<RcFn<i32, i32>, TestError> =
            if g_ok { Ok(linear_cfn(ga, gb)) } else { Err(eu.clone()) };

        // LHS: u <*> (v <*> w) == apply(apply(w, v), u)
        let inner = <ResultKind<TestError> as Apply<i32, i32>>::apply(w.clone(), v);
        let lhs = <ResultKind<TestError> as Apply<i32, i32>>::apply(inner, u);

        // Ground truth with w-then-v-then-u error precedence.
        let rhs: Result<i32, TestError> = match w {
            Err(e) => Err(e),
            Ok(x) => {
                if !f_ok {
                    Err(ev)
                } else if !g_ok {
                    Err(eu)
                } else {
                    let mut f = linear_fn(fa, fb);
                    let mut g = linear_fn(ga, gb);
                    Ok(g(f(x)))
                }
            }
        };

        prop_assert_eq!(lhs, rhs);
    }

    // --- Identity ---

    /// Apply composition for `Identity` (no short-circuit arm; always present):
    /// `apply(apply(Identity(x), Identity(f)), Identity(g)) == Identity(g(f(x)))`.
    #[test]
    fn identity_apply_composition(
        i in arb_identity_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let x = i.0;
        let v = Identity(linear_cfn(fa, fb));
        let u = Identity(linear_cfn(ga, gb));

        // LHS: u <*> (v <*> w) == apply(apply(Identity(x), v), u)
        let lhs = IdentityKind::apply(IdentityKind::apply(i, v), u);

        // Ground truth.
        let rhs = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            Identity(g(f(x)))
        };

        prop_assert_eq!(lhs, rhs);
    }
}
