//! Example-based law tests for `WriterTKind` (the Writer monad transformer).
//!
//! `WriterT<W, M, A>` wraps `M::Of<(A, W)>`. The plain `Writer<W, A>` (inner
//! `IdentityKind`) has structural `Eq` once unwrapped, so most laws compare the
//! produced `(value, log)` pair directly; for the effectful inner monad we use
//! `OptionKind` and compare `Option<(A, W)>` (including `None` short-circuit).
//!
//! Coverage: Functor/Applicative/Apply/Bind/Monad smoke tests, the three Monad
//! laws, and the four `MonadWriter` operations (`tell`, `writer`, `listen`,
//! `censor`) — in particular the key Writer law `tell(w1) >> tell(w2) ==
//! tell(w1 <> w2)`.

use monadify::applicative::kind::Applicative;
use monadify::apply::kind::Apply;
use monadify::function::RcFn;
use monadify::functor::kind::Functor;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::{Bind, Monad};
use monadify::monoid::Monoid;
use monadify::transformers::writer::{MonadWriter, Writer, WriterT, WriterTKind};
use monadify::OptionKind;

// ── Identity-inner helpers (the plain Writer monad over a String log) ────────

type W<A> = Writer<String, A>; // WriterT<String, IdentityKind, A>
type WKind = WriterTKind<String, IdentityKind>;

/// Unwrap a `Writer<String, A>` to its `(value, log)` pair.
fn run_id<A>(w: W<A>) -> (A, String) {
    let Identity(pair) = w.run_writer_t;
    pair
}

fn tell(s: &str) -> W<()> {
    WKind::tell(s.to_string())
}

// ── Functor / Applicative / Apply / Bind / Monad smoke tests ────────────────

#[test]
fn functor_map_preserves_log() {
    // map touches the value, never the log.
    let w: W<i32> = WKind::writer(10, "log".to_string());
    let mapped = WKind::map(w, |v| v * 2);
    assert_eq!(run_id(mapped), (20, "log".to_string()));
}

#[test]
fn applicative_pure_has_empty_log() {
    let w: W<i32> = WKind::pure(42);
    assert_eq!(run_id(w), (42, String::empty()));
}

#[test]
fn apply_combines_logs_function_first() {
    // value carries log "v"; function carries log "f".
    let value: W<i32> = WKind::writer(5, "v".to_string());
    let func: W<RcFn<i32, i32>> =
        WriterT::new(Identity((RcFn::new(|a: i32| a * 10), "f".to_string())));
    let applied = <WKind as Apply<i32, i32>>::apply(value, func);
    // function log precedes value log: "f" <> "v" == "fv"; value 5 * 10 == 50.
    assert_eq!(run_id(applied), (50, "fv".to_string()));
}

#[test]
fn bind_combines_logs_in_order() {
    let m: W<i32> = WKind::writer(1, "a".to_string());
    let bound = WKind::bind(m, |x| WKind::writer(x + 1, "b".to_string()));
    // input log "a" then next log "b": "ab"; value 1 + 1 == 2.
    assert_eq!(run_id(bound), (2, "ab".to_string()));
}

#[test]
fn join_flattens_combining_logs() {
    let inner: W<i32> = WKind::writer(7, "in".to_string());
    let nested: W<W<i32>> = WriterT::new(Identity((inner, "out".to_string())));
    let flat = WKind::join(nested);
    // outer log "out" then inner log "in": "outin".
    assert_eq!(run_id(flat), (7, "outin".to_string()));
}

// ── Monad laws (observational over the produced (value, log) pair) ───────────

#[test]
fn monad_left_identity() {
    // pure(a).bind(f) == f(a)
    let f = |x: i32| WKind::writer(x * 3, "f".to_string());
    let lhs = WKind::bind(WKind::pure(4), f);
    let rhs = f(4);
    assert_eq!(run_id(lhs), run_id(rhs));
}

#[test]
fn monad_right_identity() {
    // m.bind(pure) == m
    let m: W<i32> = WKind::writer(9, "m".to_string());
    let m2: W<i32> = WKind::writer(9, "m".to_string());
    let lhs = WKind::bind(m, WKind::pure);
    assert_eq!(run_id(lhs), run_id(m2));
}

#[test]
fn monad_associativity() {
    // m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    let f = |x: i32| WKind::writer(x + 1, "f".to_string());
    let g = |y: i32| WKind::writer(y * 2, "g".to_string());

    let m: W<i32> = WKind::writer(3, "m".to_string());
    let lhs = WKind::bind(WKind::bind(m, f), g);

    let m2: W<i32> = WKind::writer(3, "m".to_string());
    let rhs = WKind::bind(m2, move |x| WKind::bind(f(x), g));

    assert_eq!(run_id(lhs), run_id(rhs));
}

// ── MonadWriter operations ──────────────────────────────────────────────────

#[test]
fn writer_tell_appends_monoidally() {
    // THE Writer law: tell(w1) >> tell(w2) == tell(w1 <> w2)
    let lhs = WKind::bind(tell("foo"), move |_| tell("bar"));
    let rhs = tell("foobar");
    assert_eq!(run_id(lhs.clone()), run_id(rhs));
    assert_eq!(run_id(lhs).1, "foobar");
}

#[test]
fn writer_tell_empty_is_pure_unit() {
    // tell(empty) == pure(())
    let lhs = WKind::tell(String::empty());
    let rhs: W<()> = WKind::pure(());
    assert_eq!(run_id(lhs), run_id(rhs));
}

#[test]
fn writer_constructs_value_and_log() {
    let w = WKind::writer(99, "note".to_string());
    assert_eq!(run_id(w), (99, "note".to_string()));
}

#[test]
fn writer_listen_exposes_log() {
    // listen(tell(w)) yields value ((), w) with the log left in place == w.
    let listened = tell("xyz").listen();
    let ((unit, exposed), log) = run_id(listened);
    assert_eq!(unit, ());
    assert_eq!(exposed, "xyz");
    assert_eq!(log, "xyz");
}

#[test]
fn writer_censor_rewrites_log() {
    // censor(f, tell(w)) rewrites the log to f(w), leaving the value.
    let censored = tell("quiet").censor(|w: String| w.to_uppercase());
    assert_eq!(run_id(censored), ((), "QUIET".to_string()));
}

#[test]
fn writer_vec_log_accumulates() {
    // A Vec<&str> log accumulates as an event list across binds.
    type EvKind = WriterTKind<Vec<i32>, IdentityKind>;
    let ev = |n: i32| EvKind::tell(vec![n]);
    let prog = EvKind::bind(ev(1), move |_| EvKind::bind(ev(2), move |_| ev(3)));
    let Identity(((), log)) = prog.run_writer_t;
    assert_eq!(log, vec![1, 2, 3]);
}

// ── Effectful inner monad: OptionKind threading + None short-circuit ──────────

type OptW<A> = WriterT<String, OptionKind, A>;
type OptWKind = WriterTKind<String, OptionKind>;

#[test]
fn option_inner_threads_log_when_some() {
    let tell_opt = |s: &str| OptWKind::tell(s.to_string());
    let prog = OptWKind::bind(tell_opt("a"), move |_| tell_opt("b"));
    assert_eq!(prog.run_writer_t, Some(((), "ab".to_string())));
}

#[test]
fn option_inner_none_short_circuits() {
    // A computation that fails inside the inner Option discards the log.
    let failing: OptW<i32> = WriterT::new(None);
    let bound = OptWKind::bind(failing, |x| OptWKind::writer(x, "never".to_string()));
    assert_eq!(bound.run_writer_t, None);
}

#[test]
fn option_inner_pure_empty_log() {
    let w: OptW<i32> = OptWKind::pure(7);
    assert_eq!(w.run_writer_t, Some((7, String::empty())));
}

// ── Inherent ergonomic API parity: each inherent form == the trait form ───────

/// `WKind::tell(w)` must equal `<WKind as MonadWriter<..>>::tell(w)`.
#[test]
fn inherent_tell_parity() {
    let trait_form = <WKind as MonadWriter<String, (), IdentityKind>>::tell("hello".to_string());
    let inherent_form = WKind::tell("hello".to_string());
    assert_eq!(run_id(trait_form), run_id(inherent_form));
}

/// `WKind::writer(a, w)` must equal `<WKind as MonadWriter<..>>::writer(a, w)`.
#[test]
fn inherent_writer_parity() {
    let trait_form =
        <WKind as MonadWriter<String, i32, IdentityKind>>::writer(42, "note".to_string());
    let inherent_form = WKind::writer(42, "note".to_string());
    assert_eq!(run_id(trait_form), run_id(inherent_form));
}

/// `m.listen()` must equal `<WKind as MonadWriter<..>>::listen(m)`.
#[test]
fn inherent_listen_parity() {
    let m1 = tell("xyz");
    let m2 = tell("xyz");
    let trait_form = <WKind as MonadWriter<String, (), IdentityKind>>::listen(m1);
    let inherent_form = m2.listen();
    assert_eq!(run_id(trait_form), run_id(inherent_form));
}

/// `m.censor(f)` must equal `<WKind as MonadWriter<..>>::censor(f, m)`.
#[test]
fn inherent_censor_parity() {
    let m1 = tell("quiet");
    let m2 = tell("quiet");
    let f = |w: String| w.to_uppercase();
    let trait_form = <WKind as MonadWriter<String, (), IdentityKind>>::censor(f, m1);
    let inherent_form = m2.censor(f);
    assert_eq!(run_id(trait_form), run_id(inherent_form));
}
