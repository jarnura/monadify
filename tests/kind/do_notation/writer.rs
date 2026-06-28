//! `mdo!` do-block tests for `WriterTKind` (the Writer monad: accumulating a log).
//!
//! Gated by the parent `do_notation` module's `#![cfg(feature = "do-notation")]`.
//!
//! ## What we test
//!
//! The "real power" of Writer is that a monoidal log is silently accumulated —
//! every `tell` step appends to it — while the do-block carries ordinary values.
//! These tests use `Writer<Vec<i32>, A>` (`WriterT<Vec<i32>, IdentityKind, A>`)
//! so each run yields `Identity<(A, Vec<i32>)>` and the focus is on log
//! accumulation.
//!
//! 1. **Log accumulation** — successive `tell` steps concatenate their logs.
//! 2. **Equivalence** — the `mdo!` block produces the identical `(value, log)`
//!    to a hand-written nested `bind` chain.
//! 3. **Deeper chains** — depth > 2 confirms the desugaring.
//! 4. **proptest** — `mdo!` vs hand-written `bind`, run-and-compared over
//!    generated log values.
//!
//! `guard` is intentionally absent: `WriterT` has no lawful zero, so a `guard`
//! inside a Writer do-block is a deliberate compile error (same as Reader/State).
//! `WriterT` is `Clone` when its inner `M::Of<(A, W)>` is, so the `.clone()` the
//! macro emits works for `IdentityKind`.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::writer::{Writer, WriterTKind};
use proptest::prelude::*;

type WKind = WriterTKind<Vec<i32>, IdentityKind>;
type Logged<A> = Writer<Vec<i32>, A>;

fn tell(n: i32) -> Logged<()> {
    WKind::tell(vec![n])
}
fn run_id<A>(w: Logged<A>) -> (A, Vec<i32>) {
    let Identity(pair) = w.run_writer_t;
    pair
}

// ── 1. Log accumulation in a do-block ────────────────────────────────────────

#[test]
fn writer_mdo_accumulates_log() {
    let comp: Logged<i32> = mdo! {
        WKind;
        _ <- tell(1);
        _ <- tell(2);
        _ <- tell(3);
        WKind::pure(42)
    };
    assert_eq!(run_id(comp), (42, vec![1, 2, 3]));
}

#[test]
fn writer_mdo_interleaves_values_and_log() {
    let comp: Logged<i32> = mdo! {
        WKind;
        _ <- tell(10);
        x <- WKind::writer(5, vec![20]);
        _ <- tell(30);
        WKind::pure(x + 1)
    };
    // value: writer yields 5, +1 => 6; log: [10] <> [20] <> [30] == [10, 20, 30].
    assert_eq!(run_id(comp), (6, vec![10, 20, 30]));
}

// ── 2. Equivalence with a hand-written bind chain ────────────────────────────

#[test]
fn writer_mdo_equivalent_to_manual_bind() {
    let via_mdo: Logged<i32> = mdo! {
        WKind;
        _ <- tell(1);
        _ <- tell(2);
        WKind::pure(7)
    };

    let via_bind: Logged<i32> = WKind::bind(tell(1), move |_| {
        WKind::bind(tell(2), move |_| WKind::pure(7))
    });

    assert_eq!(run_id(via_mdo), run_id(via_bind));
}

// ── 3. Deeper chain (depth > 2) ──────────────────────────────────────────────

#[test]
fn writer_mdo_four_step_chain() {
    let comp: Logged<i32> = mdo! {
        WKind;
        _ <- tell(1);
        _ <- tell(2);
        _ <- tell(3);
        _ <- tell(4);
        WKind::pure(0)
    };
    assert_eq!(run_id(comp), (0, vec![1, 2, 3, 4]));
}

// ── 4. proptest: mdo! == manual bind, run-and-compared ───────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn writer_mdo_matches_manual_over_generated_logs(a in any::<i32>(), b in any::<i32>(), v in any::<i32>()) {
        let via_mdo: Logged<i32> = mdo! {
            WKind;
            _ <- tell(a);
            _ <- tell(b);
            WKind::pure(v)
        };
        let via_bind: Logged<i32> =
            WKind::bind(tell(a), move |_| WKind::bind(tell(b), move |_| WKind::pure(v)));
        prop_assert_eq!(run_id(via_mdo), run_id(via_bind));
    }
}
