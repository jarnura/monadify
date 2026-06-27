//! Applicative law property tests for the kind-based data instances.
//!
//! Covers the **identity**, **homomorphism**, **interchange** and
//! **composition** laws over the property-testable Applicative instances per the
//! gap matrix — thirteen cells total:
//!
//! - **Option / Result<i32, String> / Identity**: all four laws.
//! - **Vec**: **interchange only**. `identity`, `homomorphism` and `composition`
//!   are blocked because they place a function in the container via
//!   `pure(CFn)` and `Vec`'s cartesian `apply` must *reuse* (clone) each wrapped
//!   `CFn`, and `CFn` is **not `Clone`** (see the gap matrix and the stubbed
//!   example tests in `tests/kind/applicative.rs`).
//!
//! `CFn` / `CFnOnce` / `ReaderT` are function-typed and remain example-only.
//! Companion to the example-based tests in `tests/kind/applicative.rs`; the
//! shared strategies live in the parent module (`super`) and are reused here,
//! not redefined.
//!
//! This crate spells the apply operator `g <*> v` as `apply(v, g)` — i.e.
//! `apply(value_container, function_container)`. The four laws therefore read:
//!
//! - **Identity:** `pure(id) <*> v == v`  →  `apply(v, pure(id)) == v`.
//! - **Homomorphism:** `pure(f) <*> pure(x) == pure(f(x))`  →
//!   `apply(pure(x), pure(f)) == pure(f(x))`.
//! - **Interchange:** `u <*> pure(y) == pure(|f| f(y)) <*> u`  →
//!   `apply(pure(y), u) == apply(u, pure(|f| f(y)))`.
//! - **Composition:** `pure(compose) <*> u <*> v <*> w == u <*> (v <*> w)`.
//!
//! ## Composition / Vec-interchange formulation (read me)
//!
//! Two cells cannot assert the *literal* canonical form and instead assert the
//! **strongest expressible equivalent** against ground-truth `g(f(x))` semantics:
//!
//! - **Composition** (Option / Result / Identity): the literal left-hand side
//!   `pure(compose) <*> u <*> v <*> w` is not constructible, exactly as
//!   documented in the sibling `apply.rs` — `pure(compose)` would need the
//!   middle composing closure to *move* a captured `CFn` into its result
//!   (making it `FnOnce`, but `RcFn::new` requires `Fn`), and the usual clone
//!   escape hatch is unavailable because **`CFn` is not `Clone`**. So we build
//!   the fully-constructible right-associated side `u <*> (v <*> w)` —
//!   `apply(apply(w, v), u)` — and assert it equals the ground truth `g(f(x))`
//!   lifted into the container. For a lawful Applicative both the canonical
//!   left-hand side and `u <*> (v <*> w)` reduce to exactly this, so this
//!   verifies the law without the non-constructible `pure(compose)`.
//! - **Vec interchange:** the canonical right-hand side `pure(|f| f(y)) <*> u`
//!   is `apply(u, pure(|f| f(y)))`, but `Vec`'s cartesian `apply` clones each
//!   element of its value container (here `u`'s `CFn` elements) — not possible
//!   since `CFn` is not `Clone`. So we assert the constructible left-hand side
//!   `apply(pure(y), u)` (each `f` applied to `y` once) against the ground truth
//!   `[f_i(y)]`, which is exactly what the canonical right-hand side denotes.
//!
//! Linear closures are materialized from `arb_linear_closure_params` via
//! `linear_fn`/`linear_cfn` (rebuilt fresh per use because `CFn` is not
//! `Clone`), using `wrapping_*` arithmetic to avoid overflow panics on arbitrary
//! `i32` inputs.

use super::{
    arb_identity_i32, arb_linear_closure_params, arb_option_i32, arb_result_i32_string, linear_cfn,
    linear_fn,
};
use monadify::applicative::kind::Applicative;
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
use proptest::prelude::*;

type TestError = String;

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Option ---

    /// Applicative identity for `Option`: `apply(v, pure(id)) == v`.
    #[test]
    fn option_applicative_identity(v in arb_option_i32()) {
        let pure_id: Option<RcFn<i32, i32>> = OptionKind::pure(RcFn::new(|x: i32| x));
        prop_assert_eq!(OptionKind::apply(v, pure_id), v);
    }

    /// Applicative homomorphism for `Option`:
    /// `apply(pure(x), pure(f)) == pure(f(x))`.
    #[test]
    fn option_applicative_homomorphism(
        x in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let pure_x: Option<i32> = OptionKind::pure(x);
        let pure_f: Option<RcFn<i32, i32>> = OptionKind::pure(linear_cfn(a, b));
        let lhs = OptionKind::apply(pure_x, pure_f);

        let mut f = linear_fn(a, b);
        let rhs: Option<i32> = OptionKind::pure(f(x));
        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative interchange for `Option`:
    /// `apply(pure(y), u) == apply(u, pure(|f| f(y)))`.
    #[test]
    fn option_applicative_interchange(
        y in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
        present in any::<bool>(),
    ) {
        let make_u = || -> Option<RcFn<i32, i32>> {
            if present { Some(linear_cfn(a, b)) } else { None }
        };

        // LHS: u <*> pure(y) == apply(pure(y), u)
        let lhs = OptionKind::apply(OptionKind::pure(y), make_u());

        // RHS: pure(|f| f(y)) <*> u == apply(u, pure(|f| f(y)))
        let pure_interchange: Option<RcFn<RcFn<i32, i32>, i32>> =
            OptionKind::pure(RcFn::new(move |f: RcFn<i32, i32>| f.call(y)));
        let rhs = OptionKind::apply(make_u(), pure_interchange);

        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative composition for `Option`: the constructible right-associated
    /// side `u <*> (v <*> w) == apply(apply(w, v), u)` equals `g(f(x))` lifted
    /// into `Option`, with the `None` arms short-circuiting. `v` carries `f`,
    /// `u` carries `g`; both the presence arm and the linear function are
    /// generated. See the module docs for why the literal `pure(compose)` chain
    /// is not constructible.
    #[test]
    fn option_applicative_composition(
        w in arb_option_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
        f_present in any::<bool>(),
        g_present in any::<bool>(),
    ) {
        let v: Option<RcFn<i32, i32>> = if f_present { Some(linear_cfn(fa, fb)) } else { None };
        let u: Option<RcFn<i32, i32>> = if g_present { Some(linear_cfn(ga, gb)) } else { None };

        let lhs = OptionKind::apply(OptionKind::apply(w, v), u);

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

    /// Applicative identity for `Result<i32, String>`: `apply(r, pure(id)) == r`.
    #[test]
    fn result_applicative_identity(r in arb_result_i32_string()) {
        let pure_id: Result<RcFn<i32, i32>, TestError> =
            ResultKind::<TestError>::pure(RcFn::new(|x: i32| x));
        prop_assert_eq!(ResultKind::<TestError>::apply(r.clone(), pure_id), r);
    }

    /// Applicative homomorphism for `Result<i32, String>`:
    /// `apply(pure(x), pure(f)) == pure(f(x))`.
    #[test]
    fn result_applicative_homomorphism(
        x in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let pure_x: Result<i32, TestError> = ResultKind::<TestError>::pure(x);
        let pure_f: Result<RcFn<i32, i32>, TestError> =
            ResultKind::<TestError>::pure(linear_cfn(a, b));
        let lhs = ResultKind::<TestError>::apply(pure_x, pure_f);

        let mut f = linear_fn(a, b);
        let rhs: Result<i32, TestError> = ResultKind::<TestError>::pure(f(x));
        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative interchange for `Result<i32, String>`:
    /// `apply(pure(y), u) == apply(u, pure(|f| f(y)))`.
    #[test]
    fn result_applicative_interchange(
        y in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
        ok in any::<bool>(),
        e in ".*",
    ) {
        let make_u = || -> Result<RcFn<i32, i32>, TestError> {
            if ok { Ok(linear_cfn(a, b)) } else { Err(e.clone()) }
        };

        // LHS: u <*> pure(y) == apply(pure(y), u)
        let lhs = ResultKind::<TestError>::apply(ResultKind::<TestError>::pure(y), make_u());

        // RHS: pure(|f| f(y)) <*> u == apply(u, pure(|f| f(y)))
        let pure_interchange: Result<RcFn<RcFn<i32, i32>, i32>, TestError> =
            ResultKind::<TestError>::pure(RcFn::new(move |f: RcFn<i32, i32>| f.call(y)));
        let rhs = ResultKind::<TestError>::apply(make_u(), pure_interchange);

        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative composition for `Result<i32, String>`: the constructible
    /// right-associated side `apply(apply(w, v), u)` equals `g(f(x))` lifted into
    /// `Result`, with `Err` short-circuiting in `w`-then-`v`-then-`u` precedence
    /// (matching this crate's `and_then`/`map` based `apply`). `v` carries `f`,
    /// `u` carries `g`; presence arm, error string and linear function are
    /// generated.
    #[test]
    fn result_applicative_composition(
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

        let inner = <ResultKind<TestError> as Apply<i32, i32>>::apply(w.clone(), v);
        let lhs = <ResultKind<TestError> as Apply<i32, i32>>::apply(inner, u);

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

    /// Applicative identity for `Identity`: `apply(i, pure(id)) == i`.
    #[test]
    fn identity_applicative_identity(i in arb_identity_i32()) {
        let pure_id: Identity<RcFn<i32, i32>> = IdentityKind::pure(RcFn::new(|x: i32| x));
        prop_assert_eq!(IdentityKind::apply(i.clone(), pure_id), i);
    }

    /// Applicative homomorphism for `Identity`:
    /// `apply(pure(x), pure(f)) == pure(f(x))`.
    #[test]
    fn identity_applicative_homomorphism(
        x in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let pure_x: Identity<i32> = IdentityKind::pure(x);
        let pure_f: Identity<RcFn<i32, i32>> = IdentityKind::pure(linear_cfn(a, b));
        let lhs = IdentityKind::apply(pure_x, pure_f);

        let mut f = linear_fn(a, b);
        let rhs: Identity<i32> = IdentityKind::pure(f(x));
        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative interchange for `Identity`:
    /// `apply(pure(y), u) == apply(u, pure(|f| f(y)))`.
    #[test]
    fn identity_applicative_interchange(
        y in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let make_u = || -> Identity<RcFn<i32, i32>> { Identity(linear_cfn(a, b)) };

        // LHS: u <*> pure(y) == apply(pure(y), u)
        let lhs = IdentityKind::apply(IdentityKind::pure(y), make_u());

        // RHS: pure(|f| f(y)) <*> u == apply(u, pure(|f| f(y)))
        let pure_interchange: Identity<RcFn<RcFn<i32, i32>, i32>> =
            IdentityKind::pure(RcFn::new(move |f: RcFn<i32, i32>| f.call(y)));
        let rhs = IdentityKind::apply(make_u(), pure_interchange);

        prop_assert_eq!(lhs, rhs);
    }

    /// Applicative composition for `Identity` (no short-circuit arm; always
    /// present): the right-associated side `apply(apply(Identity(x), v), u)`
    /// equals `Identity(g(f(x)))`.
    #[test]
    fn identity_applicative_composition(
        i in arb_identity_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let x = i.0;
        let v = Identity(linear_cfn(fa, fb));
        let u = Identity(linear_cfn(ga, gb));

        let lhs = IdentityKind::apply(IdentityKind::apply(i, v), u);

        let rhs = {
            let mut f = linear_fn(fa, fb);
            let mut g = linear_fn(ga, gb);
            Identity(g(f(x)))
        };

        prop_assert_eq!(lhs, rhs);
    }

    // --- Vec (interchange only) ---

    /// Applicative interchange for `Vec`: the constructible left-hand side
    /// `u <*> pure(y) == apply(pure(y), u)` (each `f` in `u` applied to `y`
    /// once) equals the ground truth `[f_i(y)]` — exactly what the canonical
    /// right-hand side `pure(|f| f(y)) <*> u` denotes. The canonical RHS itself
    /// is not constructible here: `Vec`'s cartesian `apply` would clone `u`'s
    /// `CFn` value-container elements, and `CFn` is not `Clone`.
    #[test]
    fn vec_applicative_interchange(
        y in any::<i32>(),
        params in prop::collection::vec(arb_linear_closure_params(), 0..=8),
    ) {
        let u: Vec<RcFn<i32, i32>> = params.iter().map(|&(a, b)| linear_cfn(a, b)).collect();

        // LHS: u <*> pure(y) == apply(pure(y), u)
        let lhs = VecKind::apply(VecKind::pure(y), u);

        // Ground truth for pure(|f| f(y)) <*> u: each function applied to y once.
        let rhs: Vec<i32> = params
            .iter()
            .map(|&(a, b)| {
                let mut f = linear_fn(a, b);
                f(y)
            })
            .collect();

        prop_assert_eq!(lhs, rhs);
    }
}
