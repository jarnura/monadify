//! Phase 4 law-preservation tests: monad laws expressed through `mdo!`.
//!
//! Compiled only under `--features do-notation` (gate inherited from the parent
//! `do_notation` module's `#![cfg(feature = "do-notation")]`).
//!
//! ## Property tests (proptest, 256 cases each)
//!
//! For each proptest-able instance — `Option<i32>`, `Result<i32, String>`,
//! `Vec<i32>`, `Identity<i32>` — three monad laws are stated with `mdo!` on at
//! least one side:
//!
//! 1. **Left identity**: `mdo!{M; x <- M::pure(a); k(x)}` == `k(a)`
//! 2. **Right identity**: `mdo!{M; x <- m; M::pure(x)}` == `m`
//! 3. **Associativity**:
//!    `mdo!{M; x <- m; y <- k(x); h(y)}`
//!    == `mdo!{M; y <- mdo!{M; x <- m; k(x)}; h(y)}` (nested do-block on RHS)
//!    Both sides are also compared with the hand-written nested bind.
//!
//! The per-instance `*_mdo_equivalence_prop` tests (which assert `mdo!` ==
//! hand-written bind) live in the sibling source files and are **not** duplicated
//! here.
//!
//! ### Kleisli-arrow design notes
//!
//! `VecKind::bind` is `flat_map` — it calls its continuation once **per element**.
//! That continuation is `FnMut + Clone`, so any value the outer `move |x|` closure
//! captures and the inner `move |y|` also needs would be "moved out of a FnMut",
//! triggering E0507.  To avoid this, the Kleisli arrows `k` and `h` used in
//! associativity tests capture **only Copy types** (`i32`, `bool`), making the
//! closures Copy and eliminating the issue on all four instances.
//!
//! ## Macro-hygiene edge-case tests (concrete)
//!
//! - Variable shadowing: monadic rebind and `let`-shadowing of the same ident.
//! - Nested do-blocks: explicit named tests beyond the associativity RHS.
//! - Guard on empty: `guard(false)` for `OptionKind` → `None`;
//!   for `VecKind` → `vec![]`.
//! - Evaluation order: dependent bindings prove top-to-bottom sequencing; a
//!   `Rc<Cell<i32>>` counter proves a `let`-statement body runs exactly once.
//! - Bare-expr sequencing: a mid-block `Some(())` that must pass through before
//!   the next bind is evaluated.
//! - Macro discard pattern: user code with `let _ = …` next to bare-expr
//!   sequencing (macro emits `move |_| {…}`) confirms no identifier collision.
//! - Tuple-pattern bind: destructuring in the `<-` position.

use super::super::proptest_laws::{
    arb_identity_i32, arb_linear_closure_params, arb_option_i32, arb_result_i32_string,
    arb_vec_i32, linear_fn,
};
use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

// ── Option ───────────────────────────────────────────────────────────────────
//
// `Option<i32>` is Copy.  Kleisli arrows capturing only `i32`/`bool` are also
// Copy, so no pre-cloning is needed for `m` or the arrows.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Left identity for Option via `mdo!`:
    /// `mdo!{OptionKind; x <- OptionKind::pure(a); k(x)}` == `k(a)`.
    #[test]
    fn option_mdo_law_left_identity(
        a in any::<i32>(),
        (ka, kb) in arb_linear_closure_params(),
        k_present in any::<bool>(),
    ) {
        let k = move |x: i32| -> Option<i32> {
            if k_present {
                let mut lf = linear_fn(ka, kb);
                Some(lf(x))
            } else {
                None
            }
        };
        // k is Copy (captures i32 and bool only).
        let lhs: Option<i32> = mdo! {
            OptionKind;
            x <- OptionKind::pure(a);
            k(x)
        };
        let rhs: Option<i32> = k(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Right identity for Option via `mdo!`:
    /// `mdo!{OptionKind; x <- m; OptionKind::pure(x)}` == `m`.
    #[test]
    fn option_mdo_law_right_identity(m in arb_option_i32()) {
        // Option<i32> is Copy; m is usable both inside mdo! and in the assertion.
        let lhs: Option<i32> = mdo! {
            OptionKind;
            x <- m;
            OptionKind::pure(x)
        };
        prop_assert_eq!(lhs, m);
    }

    /// Associativity for Option via `mdo!` with a nested do-block on the RHS.
    ///
    /// Proves:
    ///   `mdo!{M; x <- m; y <- k(x); h(y)}`
    ///   == `mdo!{M; y <- mdo!{M; x <- m; k(x)}; h(y)}`
    ///   == `OptionKind::bind(OptionKind::bind(m, k), h)`  (hand-written)
    #[test]
    fn option_mdo_law_associativity(
        m in arb_option_i32(),
        (ka, kb) in arb_linear_closure_params(),
        (ha, hb) in arb_linear_closure_params(),
        k_present in any::<bool>(),
        h_present in any::<bool>(),
    ) {
        let k = move |x: i32| -> Option<i32> {
            if k_present {
                let mut lf = linear_fn(ka, kb);
                Some(lf(x))
            } else {
                None
            }
        };
        let h = move |y: i32| -> Option<i32> {
            if h_present {
                let mut lf = linear_fn(ha, hb);
                Some(lf(y))
            } else {
                None
            }
        };
        // k, h, and m are all Copy.

        // LHS: flat do-block   bind(m, |x| bind(k(x), h))
        let lhs: Option<i32> = mdo! {
            OptionKind;
            x <- m;
            y <- k(x);
            h(y)
        };

        // RHS: nested do-block   bind(bind(m, k), h)
        let rhs_nested: Option<i32> = mdo! {
            OptionKind;
            y <- mdo! {
                OptionKind;
                x <- m;
                k(x)
            };
            h(y)
        };

        // Hand-written for cross-check.
        let rhs_hand: Option<i32> = OptionKind::bind(OptionKind::bind(m, k), h);

        prop_assert_eq!(lhs, rhs_nested, "flat mdo! vs nested mdo!");
        prop_assert_eq!(lhs, rhs_hand,   "flat mdo! vs hand-written bind");
    }
}

// ── Result<i32, String> ──────────────────────────────────────────────────────
//
// `Result<i32, String>` is Clone but not Copy; pre-clone `m` before each use.
// Kleisli arrows capture only `i32`/`bool` → they are Copy, avoiding E0507 in
// the two-level associativity mdo!.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Left identity for Result via `mdo!`:
    /// `mdo!{ResultKind::<String>; x <- ResultKind::<String>::pure(a); k(x)}` == `k(a)`.
    #[test]
    fn result_mdo_law_left_identity(
        a in any::<i32>(),
        (ka, kb) in arb_linear_closure_params(),
        k_ok in any::<bool>(),
    ) {
        let k = move |x: i32| -> Result<i32, String> {
            if k_ok {
                Ok(x.wrapping_mul(ka).wrapping_add(kb))
            } else {
                Err(String::from("k-err"))
            }
        };
        // k is Copy (captures i32 and bool only).
        let lhs: Result<i32, String> = mdo! {
            ResultKind::<String>;
            x <- ResultKind::<String>::pure(a);
            k(x)
        };
        let rhs: Result<i32, String> = k(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Right identity for Result via `mdo!`:
    /// `mdo!{ResultKind::<String>; x <- m; ResultKind::<String>::pure(x)}` == `m`.
    #[test]
    fn result_mdo_law_right_identity(m in arb_result_i32_string()) {
        let m_for_lhs = m.clone();
        let lhs: Result<i32, String> = mdo! {
            ResultKind::<String>;
            x <- m_for_lhs;
            ResultKind::<String>::pure(x)
        };
        prop_assert_eq!(lhs, m);
    }

    /// Associativity for Result via `mdo!` with a nested do-block on the RHS.
    #[test]
    fn result_mdo_law_associativity(
        m in arb_result_i32_string(),
        (ka, kb) in arb_linear_closure_params(),
        (ha, hb) in arb_linear_closure_params(),
        k_ok in any::<bool>(),
        h_ok in any::<bool>(),
    ) {
        let k = move |x: i32| -> Result<i32, String> {
            if k_ok {
                Ok(x.wrapping_mul(ka).wrapping_add(kb))
            } else {
                Err(String::from("k-err"))
            }
        };
        let h = move |y: i32| -> Result<i32, String> {
            if h_ok {
                Ok(y.wrapping_mul(ha).wrapping_add(hb))
            } else {
                Err(String::from("h-err"))
            }
        };
        // k and h are Copy.

        let m_lhs  = m.clone();
        let m_rhs  = m.clone();
        let m_hand = m;

        let lhs: Result<i32, String> = mdo! {
            ResultKind::<String>;
            x <- m_lhs;
            y <- k(x);
            h(y)
        };

        let rhs_nested: Result<i32, String> = mdo! {
            ResultKind::<String>;
            y <- mdo! {
                ResultKind::<String>;
                x <- m_rhs;
                k(x)
            };
            h(y)
        };

        let rhs_hand: Result<i32, String> = ResultKind::<String>::bind(
            ResultKind::<String>::bind(m_hand, k),
            h,
        );

        prop_assert_eq!(lhs.clone(), rhs_nested, "flat mdo! vs nested mdo!");
        prop_assert_eq!(lhs,         rhs_hand,   "flat mdo! vs hand-written bind");
    }
}

// ── Vec<i32> ─────────────────────────────────────────────────────────────────
//
// `Vec<i32>` is Clone but not Copy; pre-clone `m` before each use.
// Kleisli arrows MUST be Copy (capture only `i32`/`bool`) to avoid E0507:
// `VecKind::bind` is `flat_map`, which calls its `FnMut` continuation once per
// element.  If `h` (captured by the outer `move |x|`) is not Copy, the inner
// `move |y| { h(y) }` would move `h` out of the outer FnMut on each call.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Left identity for Vec via `mdo!`:
    /// `mdo!{VecKind; x <- VecKind::pure(a); k(x)}` == `k(a)`.
    #[test]
    fn vec_mdo_law_left_identity(
        a in any::<i32>(),
        (ka, kb) in arb_linear_closure_params(),
        k_empty in any::<bool>(),
    ) {
        let k = move |x: i32| -> Vec<i32> {
            if k_empty {
                Vec::new()
            } else {
                vec![x.wrapping_mul(ka).wrapping_add(kb)]
            }
        };
        // k is Copy.
        let lhs: Vec<i32> = mdo! {
            VecKind;
            x <- VecKind::pure(a);
            k(x)
        };
        let rhs: Vec<i32> = k(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Right identity for Vec via `mdo!`:
    /// `mdo!{VecKind; x <- m; VecKind::pure(x)}` == `m`.
    #[test]
    fn vec_mdo_law_right_identity(m in arb_vec_i32()) {
        let m_for_lhs = m.clone();
        let lhs: Vec<i32> = mdo! {
            VecKind;
            x <- m_for_lhs;
            VecKind::pure(x)
        };
        prop_assert_eq!(lhs, m);
    }

    /// Associativity for Vec via `mdo!` with a nested do-block on the RHS.
    ///
    /// Arrows `k` and `h` each produce at most one element (0 or 1) to keep
    /// the flat_map cross-product bounded to at most `|m|` elements.
    #[test]
    fn vec_mdo_law_associativity(
        m in arb_vec_i32(),
        (ka, kb) in arb_linear_closure_params(),
        (ha, hb) in arb_linear_closure_params(),
        k_empty in any::<bool>(),
        h_empty in any::<bool>(),
    ) {
        let k = move |x: i32| -> Vec<i32> {
            if k_empty {
                Vec::new()
            } else {
                vec![x.wrapping_mul(ka).wrapping_add(kb)]
            }
        };
        let h = move |y: i32| -> Vec<i32> {
            if h_empty {
                Vec::new()
            } else {
                vec![y.wrapping_mul(ha).wrapping_add(hb)]
            }
        };
        // k and h are Copy.

        let m_lhs  = m.clone();
        let m_rhs  = m.clone();
        let m_hand = m;

        let lhs: Vec<i32> = mdo! {
            VecKind;
            x <- m_lhs;
            y <- k(x);
            h(y)
        };

        let rhs_nested: Vec<i32> = mdo! {
            VecKind;
            y <- mdo! {
                VecKind;
                x <- m_rhs;
                k(x)
            };
            h(y)
        };

        let rhs_hand: Vec<i32> = VecKind::bind(VecKind::bind(m_hand, k), h);

        prop_assert_eq!(lhs.clone(), rhs_nested, "flat mdo! vs nested mdo!");
        prop_assert_eq!(lhs,         rhs_hand,   "flat mdo! vs hand-written bind");
    }
}

// ── Identity<i32> ────────────────────────────────────────────────────────────
//
// `Identity<i32>` is Clone but not Copy (derives Clone only).
// Kleisli arrows capturing only `i32` are Copy.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// Left identity for Identity via `mdo!`:
    /// `mdo!{IdentityKind; x <- IdentityKind::pure(a); k(x)}` == `k(a)`.
    #[test]
    fn identity_mdo_law_left_identity(
        a in any::<i32>(),
        (ka, kb) in arb_linear_closure_params(),
    ) {
        let k = move |x: i32| -> Identity<i32> {
            let mut lf = linear_fn(ka, kb);
            Identity(lf(x))
        };
        // k is Copy (captures i32 only).
        let lhs: Identity<i32> = mdo! {
            IdentityKind;
            x <- IdentityKind::pure(a);
            k(x)
        };
        let rhs: Identity<i32> = k(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Right identity for Identity via `mdo!`:
    /// `mdo!{IdentityKind; x <- m; IdentityKind::pure(x)}` == `m`.
    #[test]
    fn identity_mdo_law_right_identity(m in arb_identity_i32()) {
        let m_for_lhs = m.clone();
        let lhs: Identity<i32> = mdo! {
            IdentityKind;
            x <- m_for_lhs;
            IdentityKind::pure(x)
        };
        prop_assert_eq!(lhs, m);
    }

    /// Associativity for Identity via `mdo!` with a nested do-block on the RHS.
    #[test]
    fn identity_mdo_law_associativity(
        m in arb_identity_i32(),
        (ka, kb) in arb_linear_closure_params(),
        (ha, hb) in arb_linear_closure_params(),
    ) {
        let k = move |x: i32| -> Identity<i32> {
            let mut lf = linear_fn(ka, kb);
            Identity(lf(x))
        };
        let h = move |y: i32| -> Identity<i32> {
            let mut lf = linear_fn(ha, hb);
            Identity(lf(y))
        };
        // k and h are Copy.

        let m_lhs  = m.clone();
        let m_rhs  = m.clone();
        let m_hand = m;

        let lhs: Identity<i32> = mdo! {
            IdentityKind;
            x <- m_lhs;
            y <- k(x);
            h(y)
        };

        let rhs_nested: Identity<i32> = mdo! {
            IdentityKind;
            y <- mdo! {
                IdentityKind;
                x <- m_rhs;
                k(x)
            };
            h(y)
        };

        let rhs_hand: Identity<i32> = IdentityKind::bind(IdentityKind::bind(m_hand, k), h);

        prop_assert_eq!(lhs.clone(), rhs_nested, "flat mdo! vs nested mdo!");
        prop_assert_eq!(lhs,         rhs_hand,   "flat mdo! vs hand-written bind");
    }
}

// ── Macro-hygiene edge-case tests ────────────────────────────────────────────

/// Rebinding the same identifier in successive `<-` lines: the second binding
/// shadows the first.  The outer `x = 5` is visible in `Some(x * 2)` (the RHS
/// of the second bind), and the inner `x = 10` is what the final expression sees.
#[test]
fn hygiene_variable_shadowing_monadic_rebind() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(5i32);
        x <- Some(x * 2i32);    // outer x = 5 used here; inner x = 10
        OptionKind::pure(x)      // x is the inner binding = 10
    };
    assert_eq!(result, Some(10));
}

/// A `let` binding inside `mdo!` shadows a previously monadic-bound name.
/// The let desugars to `{ let x = x + 3; rest }` inside the outer closure;
/// the new `x = 10` shadows without corrupting the earlier `x = 7`.
#[test]
fn hygiene_variable_shadowing_let_binding() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(7i32);
        let x = x + 3i32;      // pure-let shadows the monadic x; x = 10
        OptionKind::pure(x)     // x is the let binding = 10
    };
    assert_eq!(result, Some(10));
}

/// Explicit nested `mdo!` (Option inside Option), distinct from the
/// associativity law test.  The inner block produces `Some(12)` which the
/// outer bind unwraps into `inner`, then returns `Some(13)`.
#[test]
fn hygiene_nested_do_block_option_in_option() {
    let result: Option<i32> = mdo! {
        OptionKind;
        inner <- mdo! {
            OptionKind;
            a <- Some(3i32);
            b <- Some(4i32);
            OptionKind::pure(a * b)      // = Some(12)
        };
        OptionKind::pure(inner + 1i32)   // = Some(13)
    };
    assert_eq!(result, Some(13));
}

/// When the inner do-block short-circuits to `None`, the outer bind propagates
/// `None` without executing the outer continuation.
#[test]
fn hygiene_nested_do_block_inner_short_circuits() {
    let result: Option<i32> = mdo! {
        OptionKind;
        inner <- mdo! {
            OptionKind;
            a <- Some(3i32);
            b <- None::<i32>;
            OptionKind::pure(a + b)        // unreachable; inner = None
        };
        OptionKind::pure(inner + 100i32)   // unreachable
    };
    assert_eq!(result, None);
}

/// Nested do-blocks using `VecKind` (Vec in Vec).  The inner block produces a
/// four-element Vec; the outer block doubles each element.
#[test]
fn hygiene_nested_do_block_vec_in_vec() {
    let result: Vec<i32> = mdo! {
        VecKind;
        s <- mdo! {
            VecKind;
            a <- vec![1i32, 2];
            b <- vec![10i32, 20];
            VecKind::pure(a + b)      // [11, 21, 12, 22]
        };
        VecKind::pure(s * 2i32)       // [22, 42, 24, 44]
    };
    assert_eq!(result, vec![22, 42, 24, 44]);
}

/// `guard(false)` inside an Option do-block yields `None` regardless of the
/// surrounding binds, demonstrating the guard/bind interplay.
#[test]
fn hygiene_guard_false_option_yields_none() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(42i32);
        guard(false);
        OptionKind::pure(x)
    };
    assert_eq!(result, None);
}

/// `guard(false)` inside a Vec do-block yields `vec![]` regardless of the
/// surrounding binds.
#[test]
fn hygiene_guard_false_vec_yields_empty() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3];
        guard(false);
        VecKind::pure(x)
    };
    assert_eq!(result, Vec::<i32>::new());
}

/// Evaluation order via dependent bindings: each step uses the value produced
/// by the PREVIOUS step, so the result of `x + y + z = 1 + 2 + 3 = 6` proves
/// steps executed in source order.
#[test]
fn hygiene_evaluation_order_dependent_bindings() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(1i32);
        y <- Identity(x + 1i32);    // y = 2, uses x = 1
        z <- Identity(y + 1i32);    // z = 3, uses y = 2
        IdentityKind::pure(x + y + z)
    };
    assert_eq!(result, Identity(6)); // 1 + 2 + 3
}

/// A bare `Some(())` expression in the middle of an Option do-block runs
/// top-to-bottom: subsequent binds still see names from earlier steps.
#[test]
fn hygiene_bare_expr_sequencing_runs_in_order() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(10i32);
        Some(());              // bare expr; must not short-circuit
        y <- Some(x + 5i32);  // x = 10 is still in scope
        OptionKind::pure(y)
    };
    assert_eq!(result, Some(15));
}

/// A `let` statement body runs exactly once and its side effect is observable
/// via an `Rc<Cell<i32>>` counter retained in the outer scope.
///
/// Only a single closure depth is used here (no second bind that would
/// require capturing the counter at two different nesting levels), which avoids
/// the E0507 "move-out-of-FnMut" constraint on the mdo! desugaring.
#[test]
fn hygiene_let_stmt_runs_exactly_once_cell_counter() {
    use std::cell::Cell;
    use std::rc::Rc;

    let counter = Rc::new(Cell::new(0i32));
    let handle = Rc::clone(&counter); // kept in outer scope for assertion

    // `counter` is captured by `move |x|` (the outer bind's closure) and used
    // only in the `let` statement — no deeper closure references it.
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(10i32);
        let n = { counter.set(counter.get() + 1); counter.get() };
        OptionKind::pure(x + n)
    };

    // n = 1 (incremented once), so result = Some(10 + 1) = Some(11).
    assert_eq!(result, Some(11));
    // `handle` still points to the shared Cell; proves the let ran exactly once.
    assert_eq!(handle.get(), 1);
}

/// The macro emits `move |_| { … }` for bare-expr sequencing.  A user-written
/// `let _ = …` inside the same block takes the `Stmt::Let` desugaring path and
/// becomes `{ let _ = …; rest }` — it does NOT clash with the closure's `_`
/// parameter, which lives in a separate scope.
#[test]
fn hygiene_user_let_underscore_no_clash_with_macro_discard_pattern() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(5i32);
        Some(());          // bare expr — macro emits: move |_| { … }
        let _ = x * 100;  // user let _ — Stmt::Let path: { let _ = x*100; rest }
        OptionKind::pure(x * 2i32)
    };
    assert_eq!(result, Some(10));
}

/// A tuple-pattern in the `<-` position destructures the monadic value
/// correctly, binding both components to fresh names.
#[test]
fn hygiene_tuple_pattern_bind_destructures_correctly() {
    let result: Option<i32> = mdo! {
        OptionKind;
        (a, b) <- Some((3i32, 4i32));
        OptionKind::pure(a + b)
    };
    assert_eq!(result, Some(7));
}
