//! Property-based law tests for `RcFnKind<i32>` (the function / Reader monad).
//!
//! Because closures cannot implement `PartialEq`, two `RcFn<Env, A>` values are
//! compared by **evaluating both at a generated environment** — the same technique
//! the example-based tests in `tests/kind/cfn_clonable.rs` use. A single generated
//! `env: i32` is sufficient to distinguish the two sides for all linear
//! materializations used here.
//!
//! `RcFn` is `Clone`, so Kleisli arrows and function containers can be cloned and
//! reused across both sides without rebuilding from parameters — this was the key
//! gap that made `CFnKind` untestable for property-based laws.
//!
//! Covered laws (Functor + Monad; twelve cells total):
//! - **Functor identity:** `map(fa, id)(env) == fa(env)`
//! - **Functor composition:** `map(map(fa, f), g)(env) == map(fa, g∘f)(env)`
//! - **Monad left identity:** `bind(pure(a), k)(env) == k(a)(env)`
//! - **Monad right identity:** `bind(m, pure)(env) == m(env)`
//! - **Monad associativity:**
//!   `bind(bind(m, f), g)(env) == bind(m, |x| bind(f(x), g))(env)`

use super::arb_linear_closure_params;
use monadify::applicative::kind::Applicative;
use monadify::function::RcFn;
use monadify::functor::kind::Functor;
use monadify::kind_based::kind::RcFnKind;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

/// Materialise a deterministic linear `RcFn<i32, i32>` from slope/intercept params.
///
/// `fa(env) = env.wrapping_mul(a).wrapping_add(b)`.
fn make_rcfn(a: i32, b: i32) -> RcFn<i32, i32> {
    RcFn::new(move |env: i32| env.wrapping_mul(a).wrapping_add(b))
}

/// A Kleisli arrow returning an `RcFn<i32, i32>` whose output depends on
/// both the bound value `x` and the environment.
///
/// `k(x)(env) = env.wrapping_add(x.wrapping_mul(p)).wrapping_add(q)`.
fn make_kleisli(p: i32, q: i32) -> impl Fn(i32) -> RcFn<i32, i32> + Clone + 'static {
    move |x: i32| RcFn::new(move |env: i32| env.wrapping_add(x.wrapping_mul(p)).wrapping_add(q))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // ── Functor identity ──────────────────────────────────────────────────────

    /// Functor identity for `RcFnKind<i32>`:
    /// `map(fa, |x| x)(env) == fa(env)` for all generated `env` and `fa`.
    #[test]
    fn rcfn_kind_functor_identity(
        env in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let fa = make_rcfn(a, b);
        let mapped = RcFnKind::<i32>::map(fa.clone(), |x: i32| x);
        prop_assert_eq!(mapped.call(env), fa.call(env));
    }

    // ── Functor composition ───────────────────────────────────────────────────

    /// Functor composition for `RcFnKind<i32>`:
    /// `map(map(fa, f), g)(env) == map(fa, g∘f)(env)`.
    ///
    /// Both `f` and `g` are linear closures materialised from generated params.
    #[test]
    fn rcfn_kind_functor_composition(
        env in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
        (fa_p, fa_q) in arb_linear_closure_params(),
        (ga_p, ga_q) in arb_linear_closure_params(),
    ) {
        let fa = make_rcfn(a, b);

        // f: i32 -> i32 — first mapping
        let f = move |x: i32| x.wrapping_mul(fa_p).wrapping_add(fa_q);
        // g: i32 -> i32 — second mapping
        let g = move |y: i32| y.wrapping_mul(ga_p).wrapping_add(ga_q);

        // Sequential map
        let sequential =
            RcFnKind::<i32>::map(RcFnKind::<i32>::map(fa.clone(), f), g);

        // Single composed map
        let composed = RcFnKind::<i32>::map(fa.clone(), move |x| g(f(x)));

        prop_assert_eq!(sequential.call(env), composed.call(env));
    }

    // ── Monad left identity ───────────────────────────────────────────────────

    /// Monad left identity for `RcFnKind<i32>`:
    /// `bind(pure(a), k)(env) == k(a)(env)`.
    ///
    /// `pure(a)` is a constant reader ignoring its environment; `k` is a Kleisli
    /// arrow whose result still reads the environment.
    #[test]
    fn rcfn_kind_monad_left_identity(
        env in any::<i32>(),
        a in any::<i32>(),
        (p, q) in arb_linear_closure_params(),
    ) {
        let k = make_kleisli(p, q);
        let pure_a: RcFn<i32, i32> = RcFnKind::<i32>::pure(a);

        let lhs = RcFnKind::<i32>::bind(pure_a, k.clone());
        let rhs = k(a);

        prop_assert_eq!(lhs.call(env), rhs.call(env));
    }

    // ── Monad right identity ──────────────────────────────────────────────────

    /// Monad right identity for `RcFnKind<i32>`:
    /// `bind(m, pure)(env) == m(env)`.
    ///
    /// `bind(m, |x| pure(x))` wraps each output of `m` back into a constant
    /// reader; the net effect must equal `m` itself.
    #[test]
    fn rcfn_kind_monad_right_identity(
        env in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
    ) {
        let m = make_rcfn(a, b);
        let lhs =
            RcFnKind::<i32>::bind(m.clone(), |x: i32| RcFnKind::<i32>::pure(x));
        prop_assert_eq!(lhs.call(env), m.call(env));
    }

    // ── Monad associativity ───────────────────────────────────────────────────

    /// Monad associativity for `RcFnKind<i32>`:
    /// `bind(bind(m, f), g)(env) == bind(m, |x| bind(f(x), g))(env)`.
    ///
    /// `m`, `f`, and `g` are all materialised from generated scalar params.
    /// Because `RcFn` is `Clone`, both `f` and `g` can be shared between the two
    /// sides without rebuilding from parameters.
    #[test]
    fn rcfn_kind_monad_associativity(
        env in any::<i32>(),
        (a, b) in arb_linear_closure_params(),
        (fp, fq) in arb_linear_closure_params(),
        (gp, gq) in arb_linear_closure_params(),
    ) {
        let m = make_rcfn(a, b);
        let f = make_kleisli(fp, fq);
        let g = make_kleisli(gp, gq);

        // LHS: bind(bind(m, f), g)
        let lhs = RcFnKind::<i32>::bind(
            RcFnKind::<i32>::bind(m.clone(), f.clone()),
            g.clone(),
        );

        // RHS: bind(m, |x| bind(f(x), g))
        let f_rhs = f.clone();
        let g_rhs = g.clone();
        let rhs = RcFnKind::<i32>::bind(m.clone(), move |x: i32| {
            RcFnKind::<i32>::bind(f_rhs.clone()(x), g_rhs.clone())
        });

        prop_assert_eq!(lhs.call(env), rhs.call(env));
    }
}
