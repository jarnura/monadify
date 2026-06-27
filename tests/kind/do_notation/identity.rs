//! `mdo!` do-block tests for `IdentityKind`.
//!
//! Tests in this file are gated by the parent `do_notation` module's
//! `#![cfg(feature = "do-notation")]` attribute, so they are invisible to the
//! default build.
//!
//! Covered scenarios
//! -----------------
//! 1. Basic threading: two bindings over `Identity` values produce the expected result.
//! 2. `let` binding + bare-expr sequencing inside the block.
//! 3. Equivalence (key law test): `mdo!` output equals hand-written nested
//!    `IdentityKind::bind(…)` calls for concrete inputs; property-based leg
//!    uses `arb_identity_i32()` (256 generated cases).
//! 4. A 3+ binding chain (three and four bindings) to verify threading depth.
//!
//! **Note on `guard`:** `guard` is NOT supported for `IdentityKind` — there is no
//! lawful zero for `Identity` (no short-circuit semantics). No guard tests appear
//! here; attempting one would be a deliberate compile error.
//!
//! **Ownership note:** `Identity<i32>` derives `Clone` but not `Copy`. The macro
//! emits `(expr).clone()` for each monadic RHS. Variables that appear inside an
//! outer `move` closure are still moved into that closure, so where the same
//! `ma`/`mb` value is needed both for `mdo!` (lhs) and a subsequent hand-written
//! `rhs` expression, we pre-clone into separate `_lhs` bindings — mirroring the
//! pattern established in `result.rs`.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

// Reuse the existing `arb_identity_i32` proptest strategy (already `pub`).
use super::super::proptest_laws::arb_identity_i32;

// ── 1. Basic threading ───────────────────────────────────────────────────────

/// Two bindings should thread values through and produce `Identity(2 + 3) == Identity(5)`.
#[test]
fn identity_mdo_two_bindings() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(2);
        y <- Identity(3);
        IdentityKind::pure(x + y)
    };
    assert_eq!(result, Identity(5));
}

// ── 2. `let` binding + bare-expr sequencing ──────────────────────────────────

/// A `let` binding inside an `mdo!` block introduces a pure local name that
/// can be used in subsequent steps.
#[test]
fn identity_mdo_let_binding_inside_block() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(10);
        let doubled = x * 2;
        IdentityKind::pure(doubled)
    };
    assert_eq!(result, Identity(20));
}

/// A `let` binding can reference multiple previously bound names from distinct
/// monadic steps.
#[test]
fn identity_mdo_let_binding_combines_two_values() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(4);
        y <- Identity(6);
        let sum = x + y;
        IdentityKind::pure(sum * 2)
    };
    assert_eq!(result, Identity(20));
}

/// A bare-expr sequencing line (no `<-`) runs the monadic action for its effect
/// and discards the wrapped value; subsequent bindings still see earlier names.
#[test]
fn identity_mdo_bare_expr_sequencing() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(5);
        Identity(());
        IdentityKind::pure(x * 3)
    };
    assert_eq!(result, Identity(15));
}

/// `let` and bare-expr forms can appear together in the same block.
#[test]
fn identity_mdo_let_and_bare_expr_combined() {
    let result: Identity<String> = mdo! {
        IdentityKind;
        x <- Identity(7i32);
        let label = "value";
        Identity(());
        IdentityKind::pure(format!("{}: {}", label, x))
    };
    assert_eq!(result, Identity("value: 7".to_string()));
}

// ── 3. Equivalence (example-based) ───────────────────────────────────────────
//
// Key law: `mdo! { M; x <- ma; y <- mb; M::pure(f(x, y)) }`
//          == `M::bind(ma.clone(), move |x| M::bind(mb.clone(), move |y| M::pure(f(x, y))))`
//
// `Identity<i32>` is not `Copy`. The outer `move |x|` closure captures `mb_lhs`
// by move (to call `mb_lhs.clone()` inside it). We pre-clone into `_lhs` variants
// so the originals remain accessible for the hand-written rhs below.

/// Two concrete `Identity` values: `mdo!` and hand-written bind should agree on
/// `Identity(2 + 3) == Identity(5)`.
#[test]
fn identity_mdo_equivalence_concrete() {
    let ma: Identity<i32> = Identity(2);
    let mb: Identity<i32> = Identity(3);

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- ma_lhs;
        y <- mb_lhs;
        IdentityKind::pure(x + y)
    };

    let rhs: Identity<i32> = IdentityKind::bind(ma.clone(), move |x| {
        IdentityKind::bind(mb.clone(), move |y| IdentityKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Identity(5));
}

/// Negative values should thread identically through both `mdo!` and the
/// hand-written nested bind.
#[test]
fn identity_mdo_equivalence_negative_values() {
    let ma: Identity<i32> = Identity(-10);
    let mb: Identity<i32> = Identity(4);

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- ma_lhs;
        y <- mb_lhs;
        IdentityKind::pure(x + y)
    };

    let rhs: Identity<i32> = IdentityKind::bind(ma.clone(), move |x| {
        IdentityKind::bind(mb.clone(), move |y| IdentityKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Identity(-6));
}

// ── 3b. Property-based equivalence ───────────────────────────────────────────
//
// Asserts the desugaring identity holds for 256 generated `(ma, mb)` pairs
// drawn from `arb_identity_i32()`. Uses `wrapping_add` to avoid overflow panics
// on arbitrary `i32` inputs.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// `mdo! { IdentityKind; x <- ma; y <- mb; IdentityKind::pure(x.wrapping_add(y)) }`
    /// must equal the hand-written nested bind for every generated `(ma, mb)`.
    #[test]
    fn identity_mdo_equivalence_prop(
        ma in arb_identity_i32(),
        mb in arb_identity_i32(),
    ) {
        // Pre-clone so both lhs (mdo!) and rhs (hand-written) each own their copy.
        let ma_lhs = ma.clone();
        let mb_lhs = mb.clone();

        let lhs: Identity<i32> = mdo! {
            IdentityKind;
            x <- ma_lhs;
            y <- mb_lhs;
            IdentityKind::pure(x.wrapping_add(y))
        };

        let rhs: Identity<i32> = IdentityKind::bind(ma.clone(), move |x| {
            IdentityKind::bind(mb.clone(), move |y| IdentityKind::pure(x.wrapping_add(y)))
        });

        prop_assert_eq!(lhs, rhs);
    }
}

// ── 4. 3+ binding chain ──────────────────────────────────────────────────────

/// Three bindings should thread all three values and produce `Identity(1 + 2 + 3) == Identity(6)`.
#[test]
fn identity_mdo_three_bindings_chain() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(1);
        y <- Identity(2);
        z <- Identity(3);
        IdentityKind::pure(x + y + z)
    };
    assert_eq!(result, Identity(6));
}

/// Four bindings demonstrate that pure value threading is correct at depth — no
/// value is accidentally dropped or duplicated.
#[test]
fn identity_mdo_four_bindings_chain() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        a <- Identity(10);
        b <- Identity(20);
        c <- Identity(30);
        d <- Identity(40);
        IdentityKind::pure(a + b + c + d)
    };
    assert_eq!(result, Identity(100));
}

/// Three bindings where the computation depends on a `let` derived from earlier
/// bindings, demonstrating that `let` can appear mid-chain without disrupting
/// threading.
#[test]
fn identity_mdo_three_bindings_with_let_mid_chain() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(3);
        y <- Identity(4);
        let product = x * y;
        z <- Identity(product + 1);
        IdentityKind::pure(z)
    };
    assert_eq!(result, Identity(13)); // product = 12, z = 13
}
