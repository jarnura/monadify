//! Property-based (`proptest`) law tests for `ExceptTKind` (the Except transformer).
//!
//! Companion to the example-based tests in
//! `tests/kind/transformers/except.rs`. The plain `Except<E, A>` (inner
//! `IdentityKind`) has structural `Eq` once unwrapped, so laws compare the
//! produced `Result<A, E>` directly.
//!
//! Covers the three Monad laws and the `MonadError` laws (throw-left-zero,
//! catch-throw, catch-pure) over `ExceptTKind<String, IdentityKind>`, plus the
//! **Applicative–Monad consistency** law (`apply == bind-derived ap`) over the
//! multiplicative `VecKind` inner — the law that pins `apply` to the
//! short-circuiting inner-`Bind` definition and rules out the unlawful
//! "run both effects" inner-`Apply`-only variant.

use super::linear_fn;
use monadify::applicative::kind::Applicative;
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::VecKind;
use monadify::monad::kind::Bind;
use monadify::transformers::except::{Except, ExceptT, ExceptTKind};
use proptest::prelude::*;

type E<A> = Except<String, A>;
type EKind = ExceptTKind<String, IdentityKind>;

fn run<A>(m: E<A>) -> Result<A, String> {
    let Identity(r) = m.run_except_t;
    r
}

/// A possibly-throwing Kleisli arrow: maps by a linear fn, or throws when
/// `throw_at` matches the input (to exercise short-circuit paths).
fn arrow(slope: i32, intercept: i32, err: String) -> impl Fn(i32) -> E<i32> + Clone {
    move |x: i32| {
        if x == 0 {
            ExceptT::throw(err.clone())
        } else {
            let mut lf = linear_fn(slope, intercept);
            ExceptT::ok(lf(x))
        }
    }
}

fn arb_result() -> impl Strategy<Value = Result<i32, String>> {
    prop_oneof![any::<i32>().prop_map(Ok), "[a-z]{1,8}".prop_map(Err)]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Monad laws ---

    /// Left identity: `bind(pure(a), f) == f(a)`.
    #[test]
    fn except_monad_left_identity(a in any::<i32>(),
                                  (sl, ic) in (any::<i32>(), any::<i32>()), err in "[a-z]{1,8}") {
        let f = arrow(sl, ic, err);
        let lhs = EKind::bind(EKind::pure(a), f.clone());
        let rhs = f(a);
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// Right identity: `bind(m, pure) == m`.
    #[test]
    fn except_monad_right_identity(r in arb_result()) {
        let m = || ExceptT::<String, IdentityKind, i32>::new(Identity(r.clone()));
        let lhs = EKind::bind(m(), EKind::pure);
        prop_assert_eq!(run(lhs), run(m()));
    }

    /// Associativity: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn except_monad_associativity(r in arb_result(),
                                  (sl1, ic1) in (any::<i32>(), any::<i32>()), e1 in "[a-z]{1,8}",
                                  (sl2, ic2) in (any::<i32>(), any::<i32>()), e2 in "[a-z]{1,8}") {
        let f = arrow(sl1, ic1, e1);
        let g = arrow(sl2, ic2, e2);
        let m = || ExceptT::<String, IdentityKind, i32>::new(Identity(r.clone()));
        let lhs = EKind::bind(EKind::bind(m(), f.clone()), g.clone());
        let rhs = EKind::bind(m(), move |x| EKind::bind(f(x), g.clone()));
        prop_assert_eq!(run(lhs), run(rhs));
    }

    // --- MonadError laws ---

    /// Throw is a left zero for bind: `bind(throw(e), k) == throw(e)`.
    #[test]
    fn except_throw_left_zero(err in "[a-z]{1,8}", (sl, ic) in (any::<i32>(), any::<i32>())) {
        let k = arrow(sl, ic, "unused".to_string());
        let lhs = EKind::bind(
            ExceptT::throw(err.clone()),
            k,
        );
        let rhs: E<i32> = ExceptT::throw(err);
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// catch-throw: `catch(throw(e), h) == h(e)`.
    #[test]
    fn except_catch_throw(err in "[a-z]{1,8}", n in any::<i32>()) {
        let h = move |_e: String| ExceptT::ok(n);
        let lhs = ExceptT::<String, IdentityKind, i32>::throw(err.clone()).catch(h);
        let rhs = h(err);
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// catch-pure: `catch(pure(a), h) == pure(a)` (handler never runs).
    #[test]
    fn except_catch_pure(a in any::<i32>()) {
        let h = |_e: String| ExceptT::<String, IdentityKind, i32>::throw("ignored".to_string());
        let lhs = EKind::pure(a).catch(h);
        prop_assert_eq!(run(lhs), Ok(a));
    }
}

// --- Applicative–Monad consistency over the multiplicative VecKind inner ---

type Ve<A> = ExceptT<String, VecKind, A>;
type VeKind = ExceptTKind<String, VecKind>;

fn run_vec<A>(m: Ve<A>) -> Vec<Result<A, String>> {
    m.run_except_t
}

/// `flags` builds a function container `Vec<Result<RcFn, String>>`: `true`
/// slots carry the linear function, `false` slots carry an `Err`.
fn build_func(flags: &[bool], slope: i32, intercept: i32) -> Ve<RcFn<i32, i32>> {
    let v: Vec<Result<RcFn<i32, i32>, String>> = flags
        .iter()
        .map(|&ok| {
            if ok {
                Ok(RcFn::new(move |x: i32| {
                    x.wrapping_mul(slope).wrapping_add(intercept)
                }))
            } else {
                Err("ferr".to_string())
            }
        })
        .collect();
    ExceptT::new(v)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 192, ..ProptestConfig::default() })]

    /// `apply(value, func) == bind(func, |g| bind(value, |a| pure(g(a))))`.
    ///
    /// Over `VecKind` the two only agree if `apply` short-circuits `Err` by
    /// inner `bind` (running the value side once per `Ok` function element and
    /// not at all per `Err`). The unlawful inner-`Apply`-only `apply` would run
    /// the value side unconditionally and disagree on cardinality.
    #[test]
    fn except_apply_equals_monad_ap(
        value in prop::collection::vec(
            prop_oneof![any::<i32>().prop_map(Ok::<i32, String>), "[a-z]{1,4}".prop_map(Err)],
            0..=5),
        flags in prop::collection::vec(any::<bool>(), 0..=5),
        slope in any::<i32>(), intercept in any::<i32>(),
    ) {
        let value_ex: Ve<i32> = ExceptT::new(value);

        // apply(value, func)
        let via_apply = <VeKind as Apply<i32, i32>>::apply(
            value_ex.clone(),
            build_func(&flags, slope, intercept),
        );

        // monad-derived ap: bind(func, |g| bind(value, |a| pure(g(a))))
        let via_ap = VeKind::bind(build_func(&flags, slope, intercept), move |g: RcFn<i32, i32>| {
            let g = g.clone();
            VeKind::bind(value_ex.clone(), move |a: i32| VeKind::pure(g.call(a)))
        });

        prop_assert_eq!(run_vec(via_apply), run_vec(via_ap));
    }
}
