//! `mdo!` do-block tests for `ExceptTKind` (the Except monad: short-circuit on error).
//!
//! Gated by the parent `do_notation` module's `#![cfg(feature = "do-notation")]`.
//!
//! ## What we test
//!
//! The "real power" of Except is that an error silently aborts the rest of the
//! do-block — once a step throws, no later `bind` runs. These tests use
//! `Except<String, A>` (`ExceptT<String, IdentityKind, A>`) so each run yields
//! `Identity<Result<A, String>>`.
//!
//! 1. **Happy path** — every step is `Ok`, the block produces `Ok(value)`.
//! 2. **Short-circuit** — a `throw_error` aborts the block; later binds do not run.
//! 3. **Equivalence** — the `mdo!` block produces the identical `Result` to a
//!    hand-written nested `bind` chain.
//! 4. **proptest** — `mdo!` vs hand-written `bind`, run-and-compared.
//!
//! `guard` is intentionally absent: `ExceptT` has no lawful zero, so a `guard`
//! inside an Except do-block is a deliberate compile error (same as Reader/
//! State/Writer). `ExceptT` is `Clone` when its inner `M::Of<Result<A, E>>` is,
//! so the `.clone()` the macro emits works for `IdentityKind`.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::except::{Except, ExceptTKind, MonadError};
use proptest::prelude::*;

type EKind = ExceptTKind<String, IdentityKind>;
type Checked<A> = Except<String, A>;

fn ok(n: i32) -> Checked<i32> {
    <EKind as MonadError<String, i32, IdentityKind>>::lift_either(Ok(n))
}
fn boom(msg: &str) -> Checked<i32> {
    <EKind as MonadError<String, i32, IdentityKind>>::throw_error(msg.to_string())
}
fn run_id<A>(m: Checked<A>) -> Result<A, String> {
    let Identity(r) = m.run_except_t;
    r
}

// ── 1. Happy path ────────────────────────────────────────────────────────────

#[test]
fn except_mdo_happy_path() {
    let comp: Checked<i32> = mdo! {
        EKind;
        x <- ok(1);
        y <- ok(2);
        z <- ok(3);
        EKind::pure(x + y + z)
    };
    assert_eq!(run_id(comp), Ok(6));
}

// ── 2. Short-circuit: a throw aborts the rest of the block ───────────────────

#[test]
fn except_mdo_short_circuits_on_throw() {
    let comp: Checked<i32> = mdo! {
        EKind;
        x <- ok(1);
        _ <- boom("stop");
        y <- ok(99); // never runs
        EKind::pure(x + y)
    };
    assert_eq!(run_id(comp), Err("stop".to_string()));
}

// ── 3. Equivalence with a hand-written bind chain ────────────────────────────

#[test]
fn except_mdo_equivalent_to_manual_bind() {
    let via_mdo: Checked<i32> = mdo! {
        EKind;
        x <- ok(2);
        _ <- boom("e");
        EKind::pure(x)
    };
    let via_bind: Checked<i32> = EKind::bind(ok(2), move |x| {
        EKind::bind(boom("e"), move |_| EKind::pure(x))
    });
    assert_eq!(run_id(via_mdo), run_id(via_bind));
}

// ── 4. proptest: mdo! == manual bind, run-and-compared ───────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn except_mdo_matches_manual(a in any::<i32>(), b in any::<i32>(), throws in any::<bool>()) {
        let mid = move || -> Checked<i32> {
            if throws { boom("x") } else { ok(b) }
        };
        let via_mdo: Checked<i32> = mdo! {
            EKind;
            x <- ok(a);
            y <- mid();
            EKind::pure(x.wrapping_add(y))
        };
        let via_bind: Checked<i32> =
            EKind::bind(ok(a), move |x| EKind::bind(mid(), move |y| EKind::pure(x.wrapping_add(y))));
        prop_assert_eq!(run_id(via_mdo), run_id(via_bind));
    }
}
