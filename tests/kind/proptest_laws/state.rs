//! Property-based (`proptest`) law tests for `StateTKind` (the State transformer).
//!
//! Companion to the example-based tests in
//! `tests/kind/transformers/state.rs`. `StateT` is function-typed and has no
//! structural `Eq`, so — like `ReaderT` — laws are verified **observationally**:
//! run the two sides against the same generated initial state `s0` and compare
//! the produced `(value, final_state)` pair.
//!
//! Covers the three Monad laws and the four `MonadState` laws over
//! `StateTKind<i32, IdentityKind>`, with the threaded Kleisli arrows
//! materialized from generated `linear_fn` slope/intercept parameters (rebuilt
//! fresh per use; `wrapping_*` arithmetic avoids overflow panics).

use super::linear_fn;
use monadify::applicative::kind::Applicative;
use monadify::functor::kind::Functor;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::Bind;
use monadify::transformers::state::{State, StateTKind};
use proptest::prelude::*;

type S<A> = State<i32, A>;
type SKind = StateTKind<i32, IdentityKind>;

fn run(st: S<i32>, s0: i32) -> (i32, i32) {
    let Identity(pair) = (st.run_state_t)(s0);
    pair
}
fn run_unit(st: S<()>, s0: i32) -> ((), i32) {
    let Identity(pair) = (st.run_state_t)(s0);
    pair
}
fn get() -> S<i32> {
    SKind::get()
}
fn put(s: i32) -> S<()> {
    SKind::put(s)
}
/// A threaded Kleisli arrow: maps the value by a linear fn and bumps the state.
fn arrow(slope: i32, intercept: i32, ds: i32) -> impl Fn(i32) -> S<i32> + Clone {
    move |x: i32| {
        let mut lf = linear_fn(slope, intercept);
        let v = lf(x);
        SKind::state(move |s: i32| (v, s.wrapping_add(ds)))
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Monad laws (threaded) ---

    /// Left identity: `bind(pure(a), f) == f(a)`.
    #[test]
    fn state_monad_left_identity(a in any::<i32>(), s0 in any::<i32>(),
                                 (sl, ic) in (any::<i32>(), any::<i32>()), ds in any::<i32>()) {
        let f = arrow(sl, ic, ds);
        let lhs = SKind::bind(SKind::pure(a), f.clone());
        let rhs = f(a);
        prop_assert_eq!(run(lhs, s0), run(rhs, s0));
    }

    /// Right identity: `bind(m, pure) == m`.
    #[test]
    fn state_monad_right_identity(s0 in any::<i32>(), ds in any::<i32>()) {
        let lhs = SKind::bind(
            SKind::state(move |s: i32| (s, s.wrapping_add(ds))),
            SKind::pure,
        );
        let rhs = SKind::state(move |s: i32| (s, s.wrapping_add(ds)));
        prop_assert_eq!(run(lhs, s0), run(rhs, s0));
    }

    /// Associativity: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn state_monad_associativity(s0 in any::<i32>(),
                                 (sl1, ic1, ds1) in (any::<i32>(), any::<i32>(), any::<i32>()),
                                 (sl2, ic2, ds2) in (any::<i32>(), any::<i32>(), any::<i32>())) {
        let f = arrow(sl1, ic1, ds1);
        let g = arrow(sl2, ic2, ds2);
        let m = || SKind::state(move |s: i32| (s, s.wrapping_add(1)));

        let lhs = SKind::bind(SKind::bind(m(), f.clone()), g.clone());
        let rhs = SKind::bind(m(), move |x| SKind::bind(f(x), g.clone()));
        prop_assert_eq!(run(lhs, s0), run(rhs, s0));
    }

    // --- MonadState laws ---

    /// get-get: `get >>= \_ -> get == get`.
    #[test]
    fn state_get_get(s0 in any::<i32>()) {
        let lhs = SKind::bind(get(), |_| get());
        prop_assert_eq!(run(lhs, s0), run(get(), s0));
    }

    /// get-put: `get >>= put == pure(())`.
    #[test]
    fn state_get_put(s0 in any::<i32>()) {
        let lhs = SKind::bind(get(), put);
        let rhs: S<()> = SKind::pure(());
        prop_assert_eq!(run_unit(lhs, s0), run_unit(rhs, s0));
    }

    /// put-get: `put(s) >> get == put(s) >> pure(s)`.
    #[test]
    fn state_put_get(s0 in any::<i32>(), w in any::<i32>()) {
        let lhs = SKind::bind(put(w), move |_| get());
        let rhs = SKind::bind(put(w), move |_| SKind::pure(w));
        prop_assert_eq!(run(lhs, s0), run(rhs, s0));
    }

    /// put-put: `put(s1) >> put(s2) == put(s2)` (last write wins).
    #[test]
    fn state_put_put(s0 in any::<i32>(), w1 in any::<i32>(), w2 in any::<i32>()) {
        let lhs = SKind::bind(put(w1), move |_| put(w2));
        prop_assert_eq!(run_unit(lhs, s0), run_unit(put(w2), s0));
    }

    /// `gets(f) == get.map(f)` and `modify(f) == get >>= (put . f)`.
    #[test]
    fn state_gets_and_modify_derivations(s0 in any::<i32>(), (sl, ic) in (any::<i32>(), any::<i32>())) {
        let gets = SKind::gets(move |s| {
            let mut lf = linear_fn(sl, ic);
            lf(s)
        });
        let derived_gets = SKind::map(get(), move |s| {
            let mut lf = linear_fn(sl, ic);
            lf(s)
        });
        prop_assert_eq!(run(gets, s0), run(derived_gets, s0));

        let modify = SKind::modify(move |s| {
            let mut lf = linear_fn(sl, ic);
            lf(s)
        });
        let derived_modify = SKind::bind(get(), move |s| {
            let mut lf = linear_fn(sl, ic);
            put(lf(s))
        });
        prop_assert_eq!(run_unit(modify, s0), run_unit(derived_modify, s0));
    }
}
