//! `mdo!` do-block tests for `VecKind`.
//!
//! Tests in this file are gated by the parent `do_notation` module's
//! `#![cfg(feature = "do-notation")]` attribute, so they are invisible to the
//! default build.
//!
//! Covered scenarios
//! -----------------
//! 1. Cartesian product: two `<-` bindings over non-empty vecs produce all
//!    element combinations in flat_map order (outer-first, inner-second).
//! 2. Empty short-circuit: a binding over `vec![]` (typed) anywhere in the chain
//!    makes the whole block `vec![]`.
//! 3. List-comprehension with `guard`: `guard(cond)` filters elements via the
//!    `VecKind` zero (`vec![]` on false, `vec![()]` on true). Also verifies that
//!    an always-false guard returns `vec![]`.
//! 4. Pythagorean-ish nested filter: pairs `(x, y)` drawn from two ranges,
//!    filtered by `guard(x + y == 4)`, demonstrating real comprehension power.
//! 5. Equivalence (example-based + proptest): `mdo!` output equals hand-written
//!    nested `VecKind::bind(…)` calls, across concrete cases and 256 generated
//!    `(Vec<i32>, Vec<i32>)` pairs drawn from `arb_vec_i32()`.
//!
//! **Ownership note:** `Vec<i32>` is not `Copy`. The macro emits
//! `(expr).clone()` for each monadic RHS, which means the RHS is cloned rather
//! than moved, but captured variables used *inside* the outer `move` closure are
//! still moved into that closure. Where the same `va`/`vb` is needed for both
//! `mdo!` (lhs) and a subsequent `rhs` hand-written bind, we pre-clone into
//! separate `_lhs` bindings so the originals remain available — mirroring the
//! pattern in `result.rs`.
//!
//! **Element type must be `Clone + 'static`:** `VecKind::bind` requires `A: Clone`
//! (it is called once per element via `flat_map`); `VecKind::pure` requires
//! `T: Clone`. All tests use `i32` or `(i32, i32)`, both of which satisfy this.

use monadify::applicative::kind::Applicative;
use monadify::kind_based::kind::VecKind;
use monadify::mdo;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

// Reuse the existing bounded `arb_vec_i32` strategy (size 0..=32).
use super::super::proptest_laws::arb_vec_i32;

// ── 1. Cartesian product ──────────────────────────────────────────────────────
//
// `VecKind::bind` is `flat_map`.  Two bindings produce all (x, y) combinations
// in outer-first, inner-second order:
//   x=1 → [1+10, 1+20] = [11, 21]
//   x=2 → [2+10, 2+20] = [12, 22]
//   overall = [11, 21, 12, 22]

/// Two bindings over `[1,2]` and `[10,20]` should produce `[11,21,12,22]`
/// in flat_map-nesting order.
#[test]
fn vec_mdo_cartesian_product_two_bindings() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2];
        y <- vec![10i32, 20];
        VecKind::pure(x + y)
    };
    assert_eq!(result, vec![11, 21, 12, 22]);
}

/// Three bindings produce the 8-element cross product in flat_map order:
/// outer-first nesting.
#[test]
fn vec_mdo_cartesian_product_three_bindings() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![0i32, 1];
        y <- vec![0i32, 10];
        z <- vec![0i32, 100];
        VecKind::pure(x + y + z)
    };
    // x=0 → y=0 → [0, 100]; y=10 → [10, 110]
    // x=1 → y=0 → [1, 101]; y=10 → [11, 111]
    assert_eq!(result, vec![0, 100, 10, 110, 1, 101, 11, 111]);
}

// ── 2. Empty short-circuit ────────────────────────────────────────────────────

/// A binding over `Vec::<i32>::new()` on the *first* position short-circuits the
/// whole block to `vec![]`.
#[test]
fn vec_mdo_empty_first_binding_short_circuits() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- Vec::<i32>::new();
        y <- vec![10i32, 20];
        VecKind::pure(x + y)
    };
    assert_eq!(result, vec![]);
}

/// A binding over `Vec::<i32>::new()` on the *second* position short-circuits
/// even after a successful first binding.
#[test]
fn vec_mdo_empty_second_binding_short_circuits() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2];
        y <- Vec::<i32>::new();
        VecKind::pure(x + y)
    };
    assert_eq!(result, vec![]);
}

/// A binding over `Vec::<i32>::new()` in the *middle* of three bindings
/// short-circuits the rest.
#[test]
fn vec_mdo_empty_middle_of_three_short_circuits() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2];
        y <- Vec::<i32>::new();
        z <- vec![100i32];
        VecKind::pure(x + y + z)
    };
    assert_eq!(result, vec![]);
}

// ── 3. List-comprehension with guard ─────────────────────────────────────────
//
// `guard(cond)` desugars via `MdoGuard for VecKind`:
//   cond == true  → vec![()] → bind passes `()` to next step
//   cond == false → vec![]   → bind over empty vec → contributes nothing
//
// This is the canonical list-comprehension filter.

/// `guard(x % 2 == 0)` keeps only even elements; odd elements contribute nothing.
#[test]
fn vec_mdo_guard_filters_evens() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3, 4];
        guard(x % 2 == 0);
        VecKind::pure(x)
    };
    assert_eq!(result, vec![2, 4]);
}

/// `guard(x % 2 == 0)` applied to an all-odd vec returns `vec![]`.
#[test]
fn vec_mdo_guard_all_filtered_out_returns_empty() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 3, 5, 7];
        guard(x % 2 == 0);
        VecKind::pure(x)
    };
    assert_eq!(result, vec![]);
}

/// A literal `guard(true)` never filters anything; all elements pass through.
#[test]
fn vec_mdo_guard_literal_true_passes_all() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3];
        guard(true);
        VecKind::pure(x)
    };
    assert_eq!(result, vec![1, 2, 3]);
}

/// A literal `guard(false)` eliminates every element, yielding `vec![]`.
#[test]
fn vec_mdo_guard_literal_false_yields_empty() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3];
        guard(false);
        VecKind::pure(x)
    };
    assert_eq!(result, vec![]);
}

// ── 4. Pythagorean-ish nested filter ─────────────────────────────────────────
//
// A two-variable comprehension filtered by `x + y == 4` demonstrates that guard
// correctly spans multiple outer bindings and that the resulting order matches
// flat_map nesting (outer-x first, inner-y second).
//
//   x=1: y=1 → 2≠4 skip; y=2 → 3≠4 skip; y=3 → 4==4 keep → (1,3)
//   x=2: y=1 → 3≠4 skip; y=2 → 4==4 keep → (2,2); y=3 → 5≠4 skip
//   x=3: y=1 → 4==4 keep → (3,1); y=2 → 5≠4 skip; y=3 → 6≠4 skip
//
// Expected: [(1,3), (2,2), (3,1)]

/// Pairs `(x, y)` with `x + y == 4` from ranges `[1,2,3] × [1,2,3]`.
#[test]
fn vec_mdo_guard_nested_filter_pairs_sum_to_4() {
    let result: Vec<(i32, i32)> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3];
        y <- vec![1i32, 2, 3];
        guard(x + y == 4);
        VecKind::pure((x, y))
    };
    assert_eq!(result, vec![(1, 3), (2, 2), (3, 1)]);
}

/// Guard over both bindings: `x != y` filters the diagonal, `guard(x + y > 5)`
/// further restricts. Shows composition of multiple sequential guards.
#[test]
fn vec_mdo_multiple_guards_composed() {
    let result: Vec<(i32, i32)> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3, 4];
        y <- vec![1i32, 2, 3, 4];
        guard(x != y);
        guard(x + y > 5);
        VecKind::pure((x, y))
    };
    // Pairs where x != y AND x + y > 5:
    // From [1..4] × [1..4]:
    //   x=1: y=1 (skip x==y); y=2 1+2=3≤5 skip; y=3 1+3=4≤5 skip; y=4 1+4=5≤5 skip
    //   x=2: y=1 2+1=3≤5 skip; y=2 skip; y=3 2+3=5≤5 skip; y=4 2+4=6>5 keep (2,4)
    //   x=3: y=1 3+1=4≤5 skip; y=2 3+2=5≤5 skip; y=3 skip; y=4 3+4=7>5 keep (3,4)
    //   x=4: y=1 4+1=5≤5 skip; y=2 4+2=6>5 keep (4,2); y=3 4+3=7>5 keep (4,3); y=4 skip
    assert_eq!(result, vec![(2, 4), (3, 4), (4, 2), (4, 3)]);
}

// ── 5. Equivalence (example-based) ───────────────────────────────────────────
//
// Key law: `mdo! { VecKind; x <- va; y <- vb; VecKind::pure(f(x,y)) }`
//          == `VecKind::bind(va.clone(), move |x| VecKind::bind(vb.clone(), move |y| VecKind::pure(f(x,y))))`
//
// `Vec<i32>` is not `Copy`. The macro moves `vb` into the outer `move` closure
// (captured at the first `<-` line). We pre-clone both `va` and `vb` into `_lhs`
// variants so the originals remain accessible for the hand-written rhs.

/// Both `va` and `vb` are non-empty: `mdo!` and hand-written bind yield identical
/// cartesian-product results.
#[test]
fn vec_mdo_equivalence_both_nonempty() {
    let va: Vec<i32> = vec![1, 2];
    let vb: Vec<i32> = vec![10, 20];

    let va_lhs = va.clone();
    let vb_lhs = vb.clone();
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- va_lhs;
        y <- vb_lhs;
        VecKind::pure(x + y)
    };

    let rhs: Vec<i32> = VecKind::bind(va.clone(), move |x| {
        VecKind::bind(vb.clone(), move |y| VecKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, vec![11, 21, 12, 22]);
}

/// `va` is empty: both sides return `vec![]`.
#[test]
fn vec_mdo_equivalence_first_empty() {
    let va: Vec<i32> = vec![];
    let vb: Vec<i32> = vec![10, 20];

    let va_lhs = va.clone();
    let vb_lhs = vb.clone();
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- va_lhs;
        y <- vb_lhs;
        VecKind::pure(x + y)
    };

    let rhs: Vec<i32> = VecKind::bind(va.clone(), move |x| {
        VecKind::bind(vb.clone(), move |y| VecKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, vec![]);
}

/// `vb` is empty: both sides return `vec![]`.
#[test]
fn vec_mdo_equivalence_second_empty() {
    let va: Vec<i32> = vec![1, 2];
    let vb: Vec<i32> = vec![];

    let va_lhs = va.clone();
    let vb_lhs = vb.clone();
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- va_lhs;
        y <- vb_lhs;
        VecKind::pure(x + y)
    };

    let rhs: Vec<i32> = VecKind::bind(va.clone(), move |x| {
        VecKind::bind(vb.clone(), move |y| VecKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, vec![]);
}

/// Both `va` and `vb` are empty: both sides return `vec![]`.
#[test]
fn vec_mdo_equivalence_both_empty() {
    let va: Vec<i32> = vec![];
    let vb: Vec<i32> = vec![];

    let va_lhs = va.clone();
    let vb_lhs = vb.clone();
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- va_lhs;
        y <- vb_lhs;
        VecKind::pure(x + y)
    };

    let rhs: Vec<i32> = VecKind::bind(va.clone(), move |x| {
        VecKind::bind(vb.clone(), move |y| VecKind::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, vec![]);
}

// ── 5b. Property-based equivalence ───────────────────────────────────────────
//
// Asserts the desugaring identity holds for 256 generated `(va, vb)` pairs
// drawn from `arb_vec_i32()` (size 0..=32). Uses `wrapping_add` to avoid
// overflow panics on arbitrary `i32` inputs.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// `mdo! { VecKind; x <- va; y <- vb; VecKind::pure(x.wrapping_add(y)) }`
    /// must equal the hand-written nested bind for every `(va, vb)`.
    #[test]
    fn vec_mdo_equivalence_prop(
        va in arb_vec_i32(),
        vb in arb_vec_i32(),
    ) {
        // Pre-clone for lhs (mdo! moves vb_lhs into the outer `move` closure).
        let va_lhs = va.clone();
        let vb_lhs = vb.clone();

        let lhs: Vec<i32> = mdo! {
            VecKind;
            x <- va_lhs;
            y <- vb_lhs;
            VecKind::pure(x.wrapping_add(y))
        };

        // Originals va/vb are still valid: va was borrowed-to-clone by macro,
        // vb was captured inside vb_lhs (a pre-clone) not consumed from va/vb.
        let rhs: Vec<i32> = VecKind::bind(va.clone(), move |x| {
            VecKind::bind(vb.clone(), move |y| VecKind::pure(x.wrapping_add(y)))
        });

        prop_assert_eq!(lhs, rhs);
    }
}
