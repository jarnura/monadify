//! `mdo!` do-block tests for `StateTKind` (the State monad: threading a mutable state).
//!
//! Gated by the parent `do_notation` module's `#![cfg(feature = "do-notation")]`.
//!
//! ## What we test
//!
//! The "real power" of State is that a single state value is silently threaded —
//! and updated — through every step. These tests use `State<i32, A>`
//! (`StateT<i32, IdentityKind, A>`) so each run value is `Identity<(A, i32)>` and
//! the focus is purely on state threading.
//!
//! 1. **State threading** — `get`/`put` steps in a do-block thread the updated
//!    state forward; the final `(value, state)` reflects every write.
//! 2. **Equivalence** — the `mdo!` block produces identical `(value, state)` to a
//!    hand-written nested `bind` chain (closures can't be compared, so we compare
//!    by *running* both against the same `s0`).
//! 3. **Deeper chains** — depth > 2 confirms the desugaring.
//! 4. **proptest** — `mdo!` vs hand-written `bind`, run-and-compared across
//!    generated initial states.
//!
//! `guard` is intentionally absent: `StateT` has no lawful zero, so a `guard`
//! inside a State do-block is a deliberate compile error (same as Reader).
//! `StateT` is `Rc<dyn Fn> + #[derive(Clone)]`; the `.clone()` the macro emits is
//! a cheap reference-count bump.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::state::{State, StateTKind};
use proptest::prelude::*;

type StKind = StateTKind<i32, IdentityKind>;
type Counter<A> = State<i32, A>;

fn get() -> Counter<i32> {
    StKind::get()
}
fn put(s: i32) -> Counter<()> {
    StKind::put(s)
}
fn run_id<A>(st: Counter<A>, s0: i32) -> (A, i32) {
    let Identity(pair) = (st.run_state_t)(s0);
    pair
}

// ── 1. State threading in a do-block ─────────────────────────────────────────

#[test]
fn state_mdo_threads_and_updates() {
    let comp: Counter<i32> = mdo! {
        StKind;
        x <- get();
        _ <- put(x + 1);
        y <- get();
        StKind::pure(x + y)
    };
    // s0 = 10: x = 10, state -> 11, y = 11, result = 21, final state = 11.
    assert_eq!(run_id(comp, 10), (21, 11));
}

#[test]
fn state_mdo_uses_modify() {
    let comp: Counter<i32> = mdo! {
        StKind;
        _ <- StKind::modify(|s| s * 2);
        x <- get();
        StKind::pure(x + 1)
    };
    // s0 = 5: state -> 10, x = 10, result = 11, final state = 10.
    assert_eq!(run_id(comp, 5), (11, 10));
}

// ── 2. Equivalence with a hand-written bind chain ────────────────────────────

#[test]
fn state_mdo_equivalent_to_manual_bind() {
    let via_mdo: Counter<i32> = mdo! {
        StKind;
        x <- get();
        _ <- put(x + 1);
        y <- get();
        StKind::pure(x + y)
    };

    let via_bind: Counter<i32> = StKind::bind(get(), |x| {
        StKind::bind(put(x + 1), move |_| {
            StKind::bind(get(), move |y| StKind::pure(x + y))
        })
    });

    for s0 in [-5, 0, 3, 10, 100] {
        assert_eq!(run_id(via_mdo.clone(), s0), run_id(via_bind.clone(), s0));
    }
}

// ── 3. Deeper chain (depth > 2) ──────────────────────────────────────────────

#[test]
fn state_mdo_four_binding_chain() {
    let comp: Counter<i32> = mdo! {
        StKind;
        a <- get();
        _ <- put(a + 1);
        b <- get();
        _ <- put(b + 1);
        c <- get();
        StKind::pure(a + b + c)
    };
    // s0 = 0: a=0 -> s1; put1 -> s=1; b=1 -> ; put2 -> s=2; c=2; result=0+1+2=3, state=2.
    assert_eq!(run_id(comp, 0), (3, 2));
}

// ── 4. proptest: mdo! == manual bind, run-and-compared ───────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn state_mdo_matches_manual_over_generated_states(s0 in any::<i32>(), d in any::<i32>()) {
        let via_mdo: Counter<i32> = mdo! {
            StKind;
            x <- get();
            _ <- StKind::modify(move |s| s.wrapping_add(d));
            y <- get();
            StKind::pure(x.wrapping_add(y))
        };
        let via_bind: Counter<i32> = StKind::bind(get(), move |x| {
            StKind::bind(
                StKind::modify(move |s| s.wrapping_add(d)),
                move |_| StKind::bind(get(), move |y| StKind::pure(x.wrapping_add(y))),
            )
        });
        prop_assert_eq!(run_id(via_mdo, s0), run_id(via_bind, s0));
    }
}
