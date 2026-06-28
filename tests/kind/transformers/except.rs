//! Example-based law tests for `ExceptTKind` (the Except monad transformer).
//!
//! `ExceptT<E, M, A>` wraps `M::Of<Result<A, E>>`. The plain `Except<E, A>`
//! (inner `IdentityKind`) has structural `Eq` once unwrapped, so most laws
//! compare the produced `Result<A, E>` directly; for the effectful inner monad
//! we use `OptionKind` and compare `Option<Result<A, E>>` (including the inner
//! `None`).
//!
//! Coverage: Functor/Applicative/Apply/Bind/Monad smoke tests (with `Err`
//! short-circuit), the three Monad laws, and the `MonadError` operations
//! (`throw_error`, `catch_error`, `lift_either`) — in particular the laws
//! catch-throw, catch-pure, and throw-left-zero.

use monadify::applicative::kind::Applicative;
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::functor::kind::Functor;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::{Bind, Monad};
use monadify::transformers::except::{Except, ExceptT, ExceptTKind, MonadError};
use monadify::OptionKind;

// ── Identity-inner helpers (the plain Except monad over a String error) ──────

type E<A> = Except<String, A>; // ExceptT<String, IdentityKind, A>
type EKind = ExceptTKind<String, IdentityKind>;

/// Unwrap an `Except<String, A>` to its `Result<A, String>`.
fn run_id<A>(m: E<A>) -> Result<A, String> {
    let Identity(r) = m.run_except_t;
    r
}

fn throw(msg: &str) -> E<i32> {
    <EKind as MonadError<String, i32, IdentityKind>>::throw_error(msg.to_string())
}

fn ok(n: i32) -> E<i32> {
    <EKind as MonadError<String, i32, IdentityKind>>::lift_either(Ok(n))
}

// ── Functor / Applicative / Apply / Bind / Monad smoke tests ────────────────

#[test]
fn functor_maps_over_ok() {
    let mapped = EKind::map(ok(10), |v| v * 2);
    assert_eq!(run_id(mapped), Ok(20));
}

#[test]
fn functor_skips_err() {
    // map leaves the error branch untouched.
    let mapped = EKind::map(throw("boom"), |v| v * 2);
    assert_eq!(run_id(mapped), Err("boom".to_string()));
}

#[test]
fn applicative_pure_is_ok() {
    let m: E<i32> = EKind::pure(42);
    assert_eq!(run_id(m), Ok(42));
}

#[test]
fn apply_applies_when_both_ok() {
    let value: E<i32> = ok(5);
    let func: E<RcFn<i32, i32>> = ExceptT::new(Identity(Ok(RcFn::new(|a: i32| a * 10))));
    let applied = <EKind as Apply<i32, i32>>::apply(value, func);
    assert_eq!(run_id(applied), Ok(50));
}

#[test]
fn apply_short_circuits_on_err_function() {
    // function side is Err => result is that Err, value side ignored.
    let value: E<i32> = ok(5);
    let func: E<RcFn<i32, i32>> = ExceptT::new(Identity(Err("nofunc".to_string())));
    let applied = <EKind as Apply<i32, i32>>::apply(value, func);
    assert_eq!(run_id(applied), Err("nofunc".to_string()));
}

#[test]
fn apply_propagates_err_from_value_side() {
    // value side is Err, function side Ok => result is the value's Err.
    let value: E<i32> = throw("noval");
    let func: E<RcFn<i32, i32>> = ExceptT::new(Identity(Ok(RcFn::new(|a: i32| a + 1))));
    let applied = <EKind as Apply<i32, i32>>::apply(value, func);
    assert_eq!(run_id(applied), Err("noval".to_string()));
}

#[test]
fn bind_chains_when_ok() {
    let bound = EKind::bind(ok(1), |x| ok(x + 1));
    assert_eq!(run_id(bound), Ok(2));
}

#[test]
fn bind_short_circuits_on_err() {
    // the function is never called once an Err appears.
    let bound = EKind::bind(throw("stop"), |x| ok(x + 100));
    assert_eq!(run_id(bound), Err("stop".to_string()));
}

#[test]
fn join_flattens_ok() {
    let inner: E<i32> = ok(7);
    let nested: E<E<i32>> = ExceptT::new(Identity(Ok(inner)));
    let flat = EKind::join(nested);
    assert_eq!(run_id(flat), Ok(7));
}

#[test]
fn join_propagates_outer_err() {
    let nested: E<E<i32>> = ExceptT::new(Identity(Err("outer".to_string())));
    let flat = EKind::join(nested);
    assert_eq!(run_id(flat), Err("outer".to_string()));
}

// ── Monad laws (observational over the produced Result) ──────────────────────

#[test]
fn monad_left_identity() {
    // pure(a).bind(f) == f(a)
    let f = |x: i32| ok(x * 3);
    let lhs = EKind::bind(EKind::pure(4), f);
    let rhs = f(4);
    assert_eq!(run_id(lhs), run_id(rhs));
}

#[test]
fn monad_right_identity() {
    // m.bind(pure) == m
    let lhs = EKind::bind(ok(9), EKind::pure);
    assert_eq!(run_id(lhs), run_id(ok(9)));
}

#[test]
fn monad_associativity() {
    // m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    let f = |x: i32| ok(x + 1);
    let g = |y: i32| ok(y * 2);
    let lhs = EKind::bind(EKind::bind(ok(3), f), g);
    let rhs = EKind::bind(ok(3), move |x| EKind::bind(f(x), g));
    assert_eq!(run_id(lhs), run_id(rhs));
}

#[test]
fn monad_associativity_with_err() {
    // associativity holds even when the middle step throws.
    let f = |_x: i32| throw("mid");
    let g = |y: i32| ok(y * 2);
    let lhs = EKind::bind(EKind::bind(ok(3), f), g);
    let rhs = EKind::bind(ok(3), move |x| EKind::bind(f(x), g));
    assert_eq!(run_id(lhs.clone()), run_id(rhs));
    assert_eq!(run_id(lhs), Err("mid".to_string()));
}

// ── MonadError operations ───────────────────────────────────────────────────

#[test]
fn throw_left_zero() {
    // THE error law: throw(e) >>= k == throw(e)
    let lhs = EKind::bind(throw("e"), |x| ok(x + 1));
    let rhs = throw("e");
    assert_eq!(run_id(lhs), run_id(rhs));
}

#[test]
fn catch_throw_runs_handler() {
    // catch(throw(e), h) == h(e)
    let h = |e: String| ok(e.len() as i32);
    let lhs = <EKind as MonadError<String, i32, IdentityKind>>::catch_error(throw("err"), h);
    let rhs = h("err".to_string());
    assert_eq!(run_id(lhs.clone()), run_id(rhs));
    assert_eq!(run_id(lhs), Ok(3)); // "err".len() == 3
}

#[test]
fn catch_pure_ignores_handler() {
    // catch(pure(a), h) == pure(a)  — the handler never runs on success.
    let h = |_e: String| ok(-1);
    let lhs = <EKind as MonadError<String, i32, IdentityKind>>::catch_error(ok(7), h);
    assert_eq!(run_id(lhs), Ok(7));
}

#[test]
fn lift_either_embeds_result() {
    assert_eq!(run_id(ok(5)), Ok(5));
    let err: E<i32> =
        <EKind as MonadError<String, i32, IdentityKind>>::lift_either(Err("x".into()));
    assert_eq!(run_id(err), Err("x".to_string()));
}

#[test]
fn with_except_t_maps_error_channel() {
    // with_except_t rewrites the error, leaving Ok untouched.
    let mapped_err = throw("boom").with_except_t(|e: String| e.len());
    let Identity(r) = mapped_err.run_except_t;
    assert_eq!(r, Err(4)); // "boom".len() == 4

    let mapped_ok = ok(9).with_except_t(|e: String| e.len());
    let Identity(r2) = mapped_ok.run_except_t;
    assert_eq!(r2, Ok(9));
}

// ── Effectful inner monad: OptionKind threading + inner None ─────────────────

type OptE<A> = ExceptT<String, OptionKind, A>;
type OptEKind = ExceptTKind<String, OptionKind>;

#[test]
fn option_inner_threads_ok() {
    let m: OptE<i32> = OptEKind::pure(3);
    let bound = OptEKind::bind(m, |x| {
        <OptEKind as MonadError<String, i32, OptionKind>>::lift_either(Ok(x + 1))
    });
    assert_eq!(bound.run_except_t, Some(Ok(4)));
}

#[test]
fn option_inner_carries_err() {
    let thrown: OptE<i32> =
        <OptEKind as MonadError<String, i32, OptionKind>>::throw_error("e".to_string());
    let bound = OptEKind::bind(thrown, |x| {
        <OptEKind as MonadError<String, i32, OptionKind>>::lift_either(Ok(x))
    });
    // The ExceptT-level error is carried in the inner Some.
    assert_eq!(bound.run_except_t, Some(Err("e".to_string())));
}

#[test]
fn option_inner_none_short_circuits() {
    // A failure in the inner Option discards everything (distinct from an Err).
    let failing: OptE<i32> = ExceptT::new(None);
    let bound = OptEKind::bind(failing, |x| {
        <OptEKind as MonadError<String, i32, OptionKind>>::lift_either(Ok(x))
    });
    assert_eq!(bound.run_except_t, None);
}

#[test]
fn option_inner_catch_recovers() {
    let thrown: OptE<i32> =
        <OptEKind as MonadError<String, i32, OptionKind>>::throw_error("e".to_string());
    let recovered = <OptEKind as MonadError<String, i32, OptionKind>>::catch_error(thrown, |_e| {
        <OptEKind as MonadError<String, i32, OptionKind>>::lift_either(Ok(0))
    });
    assert_eq!(recovered.run_except_t, Some(Ok(0)));
}
