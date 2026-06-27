//! `mdo!` do-block tests for `OptionKind`.
//!
//! Tests in this file are gated by the parent `do_notation` module's
//! `#![cfg(feature = "do-notation")]` attribute, so they are invisible to the
//! default build.
//!
//! Covered scenarios
//! -----------------
//! 1. Worked do-block: 2–3 bindings that all succeed → asserts the computed `Some(_)`.
//! 2. Short-circuit: a `None` binding anywhere in the chain → whole block is `None`.
//! 3. Equivalence (key law test): `mdo!` output equals hand-written nested
//!    `OptionKind::bind(…)` calls for several concrete inputs, including `None` cases.
//! 4. Guard: `guard(cond)` passes through when `cond` is `true` and short-circuits
//!    to `None` when `cond` is `false`.
//! 5. Property-based equivalence: asserts the desugaring identity holds for many
//!    generated `(ma, mb)` pairs (256 cases, reusing the existing proptest harness).
//!
//! **RED phase**: the `mdo!` stub in `monadify-macros/src/lib.rs` emits `()` for
//! every invocation regardless of input. Every test below will therefore fail to
//! compile (type mismatch: `()` vs `Option<i32>`) or fail at runtime if the
//! compiler somehow accepts the body.  That is the intentional failing state.

use monadify::applicative::kind::Applicative;
use monadify::kind_based::kind::OptionKind;
use monadify::mdo;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

// Reuse the existing proptest strategy helpers (already `pub` in the harness).
use super::super::proptest_laws::arb_option_i32;

// ── 1. Worked do-block ────────────────────────────────────────────────────────

/// Two bindings that both succeed should propagate `Some(2 + 3) == Some(5)`.
#[test]
fn option_mdo_two_bindings_both_some() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(2);
        y <- Some(3);
        OptionKind::pure(x + y)
    };
    assert_eq!(result, Some(5));
}

/// Three bindings that all succeed should produce `Some(1 + 2 + 3) == Some(6)`.
#[test]
fn option_mdo_three_bindings_all_some() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(1);
        y <- Some(2);
        z <- Some(3);
        OptionKind::pure(x + y + z)
    };
    assert_eq!(result, Some(6));
}

// ── 2. Short-circuit ─────────────────────────────────────────────────────────

/// A `None` on the *first* binding short-circuits the entire block.
#[test]
fn option_mdo_short_circuit_first_binding_none() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- None::<i32>;
        y <- Some(3);
        OptionKind::pure(x + y)
    };
    assert_eq!(result, None);
}

/// A `None` on the *second* binding short-circuits even after a successful first.
#[test]
fn option_mdo_short_circuit_second_binding_none() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(2);
        y <- None::<i32>;
        OptionKind::pure(x + y)
    };
    assert_eq!(result, None);
}

/// A `None` in the *middle* of three bindings short-circuits the rest.
#[test]
fn option_mdo_short_circuit_middle_of_three() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(1);
        y <- None::<i32>;
        z <- Some(3);
        OptionKind::pure(x + y + z)
    };
    assert_eq!(result, None);
}

// ── 3. Equivalence (example-based) ───────────────────────────────────────────
//
// The key law: `mdo! { M; x <- ma; y <- mb; M::pure(f(x, y)) }`
//              == `M::bind(ma.clone(), move |x| M::bind(mb.clone(), move |y| M::pure(f(x, y))))`

/// Both `ma` and `mb` are `Some`: result should be `Some(2 + 3) == Some(5)`.
#[test]
fn option_mdo_equivalence_both_some() {
    let ma: Option<i32> = Some(2);
    let mb: Option<i32> = Some(3);

    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- ma;
        y <- mb;
        OptionKind::pure(x + y)
    };

    let rhs: Option<i32> = OptionKind::bind(ma.clone(), move |x| {
        OptionKind::bind(mb.clone(), move |y| OptionKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Some(5));
}

/// `ma` is `None`: both `mdo!` and the hand-written bind should return `None`.
#[test]
fn option_mdo_equivalence_first_none() {
    let ma: Option<i32> = None;
    let mb: Option<i32> = Some(3);

    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- ma;
        y <- mb;
        OptionKind::pure(x + y)
    };

    let rhs: Option<i32> = OptionKind::bind(ma.clone(), move |x| {
        OptionKind::bind(mb.clone(), move |y| OptionKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, None);
}

/// `mb` is `None`: both `mdo!` and the hand-written bind should return `None`.
#[test]
fn option_mdo_equivalence_second_none() {
    let ma: Option<i32> = Some(7);
    let mb: Option<i32> = None;

    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- ma;
        y <- mb;
        OptionKind::pure(x + y)
    };

    let rhs: Option<i32> = OptionKind::bind(ma.clone(), move |x| {
        OptionKind::bind(mb.clone(), move |y| OptionKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, None);
}

/// Both `ma` and `mb` are `None`: result is `None` on both sides.
#[test]
fn option_mdo_equivalence_both_none() {
    let ma: Option<i32> = None;
    let mb: Option<i32> = None;

    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- ma;
        y <- mb;
        OptionKind::pure(x + y)
    };

    let rhs: Option<i32> = OptionKind::bind(ma.clone(), move |x| {
        OptionKind::bind(mb.clone(), move |y| OptionKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, None);
}

// ── 4. Guard ─────────────────────────────────────────────────────────────────

/// `guard(x > 0)` with a positive `x` should pass through (`Some(x)`).
#[test]
fn option_mdo_guard_condition_true_passes_through() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(4);
        guard(x > 0);
        OptionKind::pure(x)
    };
    assert_eq!(result, Some(4));
}

/// `guard(x < 0)` with a positive `x` makes the condition `false` → `None`.
#[test]
fn option_mdo_guard_condition_false_short_circuits() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(4);
        guard(x < 0);
        OptionKind::pure(x)
    };
    assert_eq!(result, None);
}

/// Guard with a literal `false` always short-circuits regardless of earlier bindings.
#[test]
fn option_mdo_guard_literal_false() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(99);
        guard(false);
        OptionKind::pure(x)
    };
    assert_eq!(result, None);
}

/// Guard with a literal `true` always passes through.
#[test]
fn option_mdo_guard_literal_true() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(42);
        guard(true);
        OptionKind::pure(x)
    };
    assert_eq!(result, Some(42));
}

// ── 5. Property-based equivalence ────────────────────────────────────────────
//
// Asserts that the desugaring identity holds for 256 generated `(ma, mb)` pairs
// drawn from `arb_option_i32()`.  Reuses the existing proptest harness without
// introducing new dependencies.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// `mdo! { OptionKind; x <- ma; y <- mb; OptionKind::pure(x.wrapping_add(y)) }`
    /// must equal the hand-written nested bind for every `(ma, mb)`.
    #[test]
    fn option_mdo_equivalence_prop(
        ma in arb_option_i32(),
        mb in arb_option_i32(),
    ) {
        let lhs: Option<i32> = mdo! {
            OptionKind;
            x <- ma;
            y <- mb;
            OptionKind::pure(x.wrapping_add(y))
        };

        let rhs: Option<i32> = OptionKind::bind(ma.clone(), move |x| {
            OptionKind::bind(mb.clone(), move |y| OptionKind::pure(x.wrapping_add(y)))
        });

        prop_assert_eq!(lhs, rhs);
    }
}
