//! Tests for `MonadTrans::lift` across all three transformers.
//!
//! `lift` embeds an inner `M::Of<A>` into a transformer, adding none of the
//! transformer's own effect: an ignored environment (`ReaderT`), an unchanged
//! threaded state (`StateT`), or an empty log (`WriterT`). We also check the
//! monad-morphism identity `lift(pure(a)) == pure(a)` for each.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::transformers::reader::{Reader, ReaderTKind};
use monadify::transformers::state::{State, StateTKind};
use monadify::transformers::trans::MonadTrans;
use monadify::transformers::writer::{Writer, WriterTKind};

// ── ReaderT: lift ignores the environment ───────────────────────────────────

#[test]
fn reader_lift_ignores_env() {
    type RKind = ReaderTKind<i32, IdentityKind>;
    let lifted: Reader<i32, &'static str> =
        <RKind as MonadTrans<&'static str, IdentityKind>>::lift(IdentityKind::pure("hi"));
    // The lifted computation yields the inner value for any environment.
    assert_eq!((lifted.run_reader_t)(0), Identity("hi"));
}

#[test]
fn reader_lift_equals_pure() {
    // lift(pure(a)) == pure(a)
    type RKind = ReaderTKind<i32, IdentityKind>;
    let lifted: Reader<i32, i32> =
        <RKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(5));
    let pured: Reader<i32, i32> = RKind::pure(5);
    assert_eq!((lifted.run_reader_t)(42), (pured.run_reader_t)(42));
}

// ── StateT: lift threads state unchanged ────────────────────────────────────

#[test]
fn state_lift_threads_state_unchanged() {
    type SKind = StateTKind<i32, IdentityKind>;
    let lifted: State<i32, &'static str> =
        <SKind as MonadTrans<&'static str, IdentityKind>>::lift(IdentityKind::pure("v"));
    // Value comes from the inner computation; state passes through untouched.
    assert_eq!((lifted.run_state_t)(99), Identity(("v", 99)));
}

#[test]
fn state_lift_equals_pure() {
    type SKind = StateTKind<i32, IdentityKind>;
    let lifted: State<i32, i32> =
        <SKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(8));
    let pured: State<i32, i32> = SKind::pure(8);
    assert_eq!((lifted.run_state_t)(3), (pured.run_state_t)(3));
}

// ── WriterT: lift adds an empty log ─────────────────────────────────────────

#[test]
fn writer_lift_adds_empty_log() {
    type WKind = WriterTKind<String, IdentityKind>;
    let lifted: Writer<String, i32> =
        <WKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(7));
    let Identity((v, log)) = lifted.run_writer_t;
    assert_eq!(v, 7);
    assert_eq!(log, ""); // empty log
}

#[test]
fn writer_lift_equals_pure() {
    type WKind = WriterTKind<String, IdentityKind>;
    let lifted: Writer<String, i32> =
        <WKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(6));
    let pured: Writer<String, i32> = WKind::pure(6);
    assert_eq!(lifted.run_writer_t, pured.run_writer_t);
}

// ── Effectful inner monad: lift over OptionKind ─────────────────────────────

#[test]
fn writer_lift_over_option_some() {
    use monadify::OptionKind;
    type WKind = WriterTKind<String, OptionKind>;
    let lifted = <WKind as MonadTrans<i32, OptionKind>>::lift(Some(11));
    assert_eq!(lifted.run_writer_t, Some((11, String::new())));
}

#[test]
fn writer_lift_over_option_none() {
    use monadify::OptionKind;
    type WKind = WriterTKind<String, OptionKind>;
    let lifted = <WKind as MonadTrans<i32, OptionKind>>::lift(None);
    assert_eq!(lifted.run_writer_t, None);
}

// ── ExceptT: lift wraps the inner value on the success (Ok) branch ───────────

#[test]
fn except_lift_wraps_ok() {
    use monadify::transformers::except::{Except, ExceptTKind};
    type EKind = ExceptTKind<String, IdentityKind>;
    let lifted: Except<String, i32> =
        <EKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(7));
    let Identity(r) = lifted.run_except_t;
    assert_eq!(r, Ok(7)); // lifting adds no error
}

#[test]
fn except_lift_equals_pure() {
    use monadify::transformers::except::{Except, ExceptTKind};
    type EKind = ExceptTKind<String, IdentityKind>;
    let lifted: Except<String, i32> =
        <EKind as MonadTrans<i32, IdentityKind>>::lift(IdentityKind::pure(6));
    let pured: Except<String, i32> = EKind::pure(6);
    assert_eq!(lifted.run_except_t, pured.run_except_t);
}

#[test]
fn except_lift_over_option_some() {
    use monadify::transformers::except::ExceptTKind;
    use monadify::OptionKind;
    type EKind = ExceptTKind<String, OptionKind>;
    let lifted = <EKind as MonadTrans<i32, OptionKind>>::lift(Some(11));
    assert_eq!(lifted.run_except_t, Some(Ok(11)));
}

#[test]
fn except_lift_over_option_none() {
    use monadify::transformers::except::ExceptTKind;
    use monadify::OptionKind;
    type EKind = ExceptTKind<String, OptionKind>;
    let lifted = <EKind as MonadTrans<i32, OptionKind>>::lift(None);
    assert_eq!(lifted.run_except_t, None::<Result<i32, String>>);
}
