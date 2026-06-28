//! Example-based law tests for `StateTKind` (the State monad transformer).
//!
//! `StateT<S, M, A> ≅ S -> M::Of<(A, S)>` has no structural `Eq` (it wraps a
//! boxed `Fn`), so every law is verified **observationally**: run the left- and
//! right-hand sides against the same initial state `s0` and compare the produced
//! `(value, final_state)` pair.
//!
//! Coverage: Functor/Applicative/Apply/Bind/Monad smoke tests, the three Monad
//! laws, and the four `MonadState` laws (get-get, get-put, put-get, put-put),
//! over both `IdentityKind` (pure threading) and `OptionKind` (effectful
//! threading + `None` short-circuit).

use monadify::applicative::kind::Applicative;
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::functor::kind::Functor;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::{Bind, Monad};
use monadify::transformers::state::{MonadState, State, StateT, StateTKind};
use monadify::OptionKind;

// ── Identity-inner helpers (the plain State monad) ──────────────────────────

type TestState<A> = State<i32, A>; // StateT<i32, IdentityKind, A>
type TestStateKind = StateTKind<i32, IdentityKind>;

/// Run a `State<i32, A>` against `s0` and unwrap the `Identity` to `(A, i32)`.
fn run_id<A>(st: TestState<A>, s0: i32) -> (A, i32) {
    let Identity(pair) = (st.run_state_t)(s0);
    pair
}

/// Assert two `State<i32, A>` computations are observationally equal at `s0`.
fn assert_eq_at<A: PartialEq + std::fmt::Debug>(l: TestState<A>, r: TestState<A>, s0: i32) {
    assert_eq!(run_id(l, s0), run_id(r, s0));
}

fn st_state<A: 'static, F: Fn(i32) -> (A, i32) + 'static>(f: F) -> TestState<A> {
    <TestStateKind as MonadState<i32, A, IdentityKind>>::state(f)
}
fn st_get() -> TestState<i32> {
    <TestStateKind as MonadState<i32, i32, IdentityKind>>::get()
}
fn st_put(s: i32) -> TestState<()> {
    <TestStateKind as MonadState<i32, (), IdentityKind>>::put(s)
}

// ── Functor / Applicative / Apply / Bind / Monad smoke tests ────────────────

#[test]
fn functor_map_threads_state_and_maps_value() {
    // (value, state) starts (10, 5); map doubles the value, leaves state.
    let st: TestState<i32> = st_state(|s| (10, s));
    let mapped = TestStateKind::map(st, |v| v * 2);
    assert_eq!(run_id(mapped, 5), (20, 5));
}

#[test]
fn applicative_pure_ignores_then_threads_state() {
    let st: TestState<i32> = TestStateKind::pure(42);
    assert_eq!(run_id(st, 7), (42, 7));
}

#[test]
fn apply_threads_state_sequentially() {
    // value: s -> (s, s+1); function: s -> (\a. a*10, s+100)
    let value: TestState<i32> = st_state(|s| (s, s + 1));
    let func: TestState<RcFn<i32, i32>> = st_state(|s| (RcFn::new(|a: i32| a * 10), s + 100));
    let applied = <TestStateKind as Apply<i32, i32>>::apply(value, func);
    // run at 1: value -> (1, 2); func from 2 -> (\a.a*10, 102); apply -> (10, 102)
    assert_eq!(run_id(applied, 1), (10, 102));
}

#[test]
fn bind_threads_output_state_into_next_step() {
    let m: TestState<i32> = st_state(|s| (s, s + 1));
    let bound = TestStateKind::bind(m, |x| st_state(move |s| (x + s, s + 1)));
    // run at 10: first -> (10, 11); second sees s=11 -> (10+11, 12) = (21, 12)
    assert_eq!(run_id(bound, 10), (21, 12));
}

#[test]
fn join_flattens_threading_state() {
    let nested: TestState<TestState<i32>> =
        st_state(|s| (st_state(|s2| (s2 * 2, s2 + 1)), s + 100));
    let flat = TestStateKind::join(nested);
    // outer at 1 -> (inner, 101); inner from 101 -> (202, 102)
    assert_eq!(run_id(flat, 1), (202, 102));
}

// ── Monad laws (threaded f/g that both read and modify state) ────────────────

#[test]
fn monad_left_identity() {
    // pure(a).bind(f) == f(a)
    let f = |x: i32| st_state(move |s| (x + s, s + 1));
    for s0 in [-3, 0, 5, 42] {
        let lhs = TestStateKind::bind(TestStateKind::pure(9), f);
        let rhs = f(9);
        assert_eq_at(lhs, rhs, s0);
    }
}

#[test]
fn monad_right_identity() {
    // m.bind(pure) == m
    for s0 in [-3, 0, 5, 42] {
        let m: TestState<i32> = st_state(|s| (s * 7, s + 2));
        let m2: TestState<i32> = st_state(|s| (s * 7, s + 2));
        let lhs = TestStateKind::bind(m, TestStateKind::pure);
        assert_eq_at(lhs, m2, s0);
    }
}

#[test]
fn monad_associativity() {
    // m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    let f = |x: i32| st_state(move |s| (x + s, s + 1));
    let g = |y: i32| st_state(move |s| (y * 2, s + 10));
    for s0 in [-3, 0, 5, 42] {
        let m: TestState<i32> = st_state(|s| (s, s + 1));
        let lhs = TestStateKind::bind(TestStateKind::bind(m, f), g);

        let m2: TestState<i32> = st_state(|s| (s, s + 1));
        let rhs = TestStateKind::bind(m2, move |x| TestStateKind::bind(f(x), g));
        assert_eq_at(lhs, rhs, s0);
    }
}

// ── The four MonadState laws (over IdentityKind) ─────────────────────────────

#[test]
fn monadstate_get_get() {
    // get >>= \_ -> get  ==  get   (reading twice is reading once)
    for s0 in [-1, 0, 13, 99] {
        let lhs = TestStateKind::bind(st_get(), |_s| st_get());
        assert_eq_at(lhs, st_get(), s0);
    }
}

#[test]
fn monadstate_get_put() {
    // get >>= put  ==  pure(())   (writing back what you read is a no-op)
    for s0 in [-1, 0, 13, 99] {
        let lhs = TestStateKind::bind(st_get(), st_put);
        let rhs: TestState<()> = TestStateKind::pure(());
        assert_eq_at(lhs, rhs, s0);
    }
}

#[test]
fn monadstate_put_get() {
    // put(s) >> get  ==  put(s) >> pure(s)   (you read back what you wrote)
    for s0 in [-1, 0, 13, 99] {
        let written = 7;
        let lhs = TestStateKind::bind(st_put(written), move |_| st_get());
        let rhs = TestStateKind::bind(st_put(written), move |_| TestStateKind::pure(written));
        assert_eq_at(lhs, rhs, s0);
    }
}

#[test]
fn monadstate_put_put() {
    // put(s1) >> put(s2)  ==  put(s2)   (last write wins)
    for s0 in [-1, 0, 13, 99] {
        let lhs = TestStateKind::bind(st_put(3), move |_| st_put(8));
        assert_eq_at(lhs, st_put(8), s0);
    }
}

#[test]
fn modify_and_gets_derive_correctly() {
    let modify = <TestStateKind as MonadState<i32, (), IdentityKind>>::modify(|s| s + 100);
    // modify(f) == get.bind(|s| put(f(s)))
    let derived_modify = TestStateKind::bind(st_get(), |s| st_put(s + 100));
    assert_eq_at(modify, derived_modify, 5);

    let gets = <TestStateKind as MonadState<i32, i32, IdentityKind>>::gets(|s| s * 3);
    // gets(f) == get.map(f)
    let derived_gets = TestStateKind::map(st_get(), |s| s * 3);
    assert_eq_at(gets, derived_gets, 5);
}

// ── Effectful inner monad: OptionKind threading + None short-circuit ──────────

type OptState<A> = StateT<i32, OptionKind, A>;
type OptStateKind = StateTKind<i32, OptionKind>;

fn run_opt<A>(st: OptState<A>, s0: i32) -> Option<(A, i32)> {
    (st.run_state_t)(s0)
}

#[test]
fn option_inner_threads_state_when_all_some() {
    let m: OptState<i32> = <OptStateKind as MonadState<i32, i32, OptionKind>>::get();
    let bound = OptStateKind::bind(m, |x| {
        <OptStateKind as MonadState<i32, i32, OptionKind>>::state(move |s| (x + s, s + 1))
    });
    // get at 4 -> Some((4,4)); state from 4 -> Some((4+4, 5)) = Some((8,5))
    assert_eq!(run_opt(bound, 4), Some((8, 5)));
}

#[test]
fn option_inner_none_short_circuits() {
    // A computation that fails inside the inner Option.
    let failing: OptState<i32> = StateT::new(|_s: i32| None);
    let bound = OptStateKind::bind(failing, |x| {
        <OptStateKind as MonadState<i32, i32, OptionKind>>::state(move |s| (x, s))
    });
    assert_eq!(run_opt(bound, 10), None);
}

#[test]
fn option_inner_monadstate_laws_hold() {
    // put-put last-write-wins over OptionKind.
    let put = |s: i32| <OptStateKind as MonadState<i32, (), OptionKind>>::put(s);
    let lhs = OptStateKind::bind(put(3), move |_| put(8));
    assert_eq!(run_opt(lhs.clone(), 0), run_opt(put(8), 0));
    assert_eq!(run_opt(lhs, 0), Some(((), 8)));
}

// ── Parity tests: inherent ergonomic form == MonadState trait form ─────────────
//
// Each test runs both the inherent `StateTKind::method(..)` form and the
// explicit `<StateTKind as MonadState<..>>::method(..)` UFCS form against
// the same initial state and asserts they produce identical `(value, state)` pairs.

#[test]
fn ergonomic_state_parity_identity() {
    let s0 = 5_i32;
    let erg: TestState<i32> = TestStateKind::state(|s| (s * 2, s + 1));
    let via_trait: TestState<i32> =
        <TestStateKind as MonadState<i32, i32, IdentityKind>>::state(|s| (s * 2, s + 1));
    assert_eq!(run_id(erg, s0), run_id(via_trait, s0));
}

#[test]
fn ergonomic_get_parity_identity() {
    let s0 = 42_i32;
    let erg: TestState<i32> = TestStateKind::get();
    let via_trait: TestState<i32> = <TestStateKind as MonadState<i32, i32, IdentityKind>>::get();
    assert_eq!(run_id(erg, s0), run_id(via_trait, s0));
}

#[test]
fn ergonomic_put_parity_identity() {
    let s0 = 7_i32;
    let erg: TestState<()> = TestStateKind::put(99);
    let via_trait: TestState<()> = <TestStateKind as MonadState<i32, (), IdentityKind>>::put(99);
    assert_eq!(run_id(erg, s0), run_id(via_trait, s0));
}

#[test]
fn ergonomic_modify_parity_identity() {
    let s0 = 3_i32;
    let erg: TestState<()> = TestStateKind::modify(|s| s * 10);
    let via_trait: TestState<()> =
        <TestStateKind as MonadState<i32, (), IdentityKind>>::modify(|s| s * 10);
    assert_eq!(run_id(erg, s0), run_id(via_trait, s0));
}

#[test]
fn ergonomic_gets_parity_identity() {
    let s0 = 4_i32;
    let erg: TestState<i32> = TestStateKind::gets(|s| s + 100);
    let via_trait: TestState<i32> =
        <TestStateKind as MonadState<i32, i32, IdentityKind>>::gets(|s| s + 100);
    assert_eq!(run_id(erg, s0), run_id(via_trait, s0));
}

#[test]
fn ergonomic_get_parity_option() {
    let s0 = 10_i32;
    let erg: OptState<i32> = OptStateKind::get();
    let via_trait: OptState<i32> = <OptStateKind as MonadState<i32, i32, OptionKind>>::get();
    assert_eq!(run_opt(erg, s0), run_opt(via_trait, s0));
}

#[test]
fn ergonomic_put_parity_option() {
    let s0 = 5_i32;
    let erg: OptState<()> = OptStateKind::put(77);
    let via_trait: OptState<()> = <OptStateKind as MonadState<i32, (), OptionKind>>::put(77);
    assert_eq!(run_opt(erg, s0), run_opt(via_trait, s0));
}
