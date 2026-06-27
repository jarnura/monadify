//! `mdo!` do-block tests for `ResultKind<E>`.
//!
//! Tests in this file are gated by the parent `do_notation` module's
//! `#![cfg(feature = "do-notation")]` attribute, so they are invisible to the
//! default build.
//!
//! Covered scenarios
//! -----------------
//! 1. Happy path: two/three `Ok` bindings produce the expected `Ok` result.
//! 2. Short-circuit on `Err`: the first `Err` aborts the whole block and the
//!    error value is preserved (verified via observable return value).
//! 3. Equivalence (key law test): `mdo!` output equals hand-written nested
//!    `ResultKind::<String>::bind(…)` calls across `Ok`/`Err` combinations,
//!    including a property-based leg with 256 generated inputs.
//! 4. `let` binding inside the block and bare-expr sequencing, to exercise
//!    those statement forms for `ResultKind`.
//!
//! **Note on `guard`:** `guard` is intentionally NOT supported for `ResultKind`
//! (it is a deliberate compile error). No guard tests appear here.
//!
//! **`E: Clone`:** `ResultKind<E>` requires `E: Clone`; `String` satisfies this.
//! All tests use `ResultKind::<String>` as the explicit block marker.
//!
//! **Variable ownership note:** `Result<i32, String>` is not `Copy`. The macro
//! emits `(expr).clone()` for each monadic RHS, which moves the variable. Where
//! a variable is needed both for `mdo!` and for a subsequent `rhs` expression,
//! we pre-clone into separate bindings.

use monadify::applicative::kind::Applicative;
use monadify::kind_based::kind::ResultKind;
use monadify::mdo;
use monadify::monad::kind::Bind;
use proptest::prelude::*;

// Reuse the existing proptest strategy helpers (already `pub` in the harness).
use super::super::proptest_laws::arb_result_i32_string;

// ── 1. Happy path ────────────────────────────────────────────────────────────

/// Two `Ok` bindings should propagate `Ok(2 + 3) == Ok(5)`.
#[test]
fn result_mdo_two_bindings_both_ok() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(2);
        y <- Ok(3);
        ResultKind::<String>::pure(x + y)
    };
    assert_eq!(result, Ok(5));
}

/// Three `Ok` bindings should produce `Ok(1 + 2 + 3) == Ok(6)`.
#[test]
fn result_mdo_three_bindings_all_ok() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(1);
        y <- Ok(2);
        z <- Ok(3);
        ResultKind::<String>::pure(x + y + z)
    };
    assert_eq!(result, Ok(6));
}

// ── 2. Short-circuit on Err ───────────────────────────────────────────────────
//
// Short-circuit semantics are verified through the observable return value:
// the block returns exactly the first `Err` and nothing else.

/// An `Err` on the *first* binding short-circuits the entire block.
#[test]
fn result_mdo_short_circuit_first_binding_err() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Err::<i32, String>("first-err".to_string());
        y <- Ok::<i32, String>(3);
        ResultKind::<String>::pure(x + y)
    };
    assert_eq!(result, Err("first-err".to_string()));
}

/// An `Err` on the *second* binding short-circuits even after a successful first.
#[test]
fn result_mdo_short_circuit_second_binding_err() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(2);
        y <- Err::<i32, String>("second-err".to_string());
        ResultKind::<String>::pure(x + y)
    };
    assert_eq!(result, Err("second-err".to_string()));
}

/// An `Err` in the *middle* of three bindings short-circuits the rest.
#[test]
fn result_mdo_short_circuit_middle_of_three() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(1);
        y <- Err::<i32, String>("mid-err".to_string());
        z <- Ok(3);
        ResultKind::<String>::pure(x + y + z)
    };
    assert_eq!(result, Err("mid-err".to_string()));
}

/// The error value propagated is the *first* `Err` encountered, not any later one.
#[test]
fn result_mdo_first_err_wins() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Err::<i32, String>("first".to_string());
        y <- Err::<i32, String>("second".to_string());
        ResultKind::<String>::pure(x + y)
    };
    assert_eq!(result, Err("first".to_string()));
}

// ── 3. Equivalence (example-based) ───────────────────────────────────────────
//
// The key law: `mdo! { M; x <- ma; y <- mb; M::pure(f(x, y)) }`
//              == `M::bind(ma.clone(), move |x| M::bind(mb.clone(), move |y| M::pure(f(x, y))))`
//
// Because `Result<i32, String>` is not `Copy`, the macro's emitted `(ma).clone()`
// moves `ma` into the outer closure. We pre-clone `ma`/`mb` into separate bindings
// so both `lhs` (mdo!) and `rhs` (hand-written bind) each receive their own copy.

/// Both `ma` and `mb` are `Ok`: result should be `Ok(2 + 3) == Ok(5)`.
#[test]
fn result_mdo_equivalence_both_ok() {
    let ma: Result<i32, String> = Ok(2);
    let mb: Result<i32, String> = Ok(3);

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- ma_lhs;
        y <- mb_lhs;
        ResultKind::<String>::pure(x + y)
    };

    let rhs: Result<i32, String> = ResultKind::<String>::bind(ma, move |x| {
        ResultKind::<String>::bind(mb.clone(), move |y| ResultKind::<String>::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Ok(5));
}

/// `ma` is `Err`: both `mdo!` and the hand-written bind should return `Err("e")`.
#[test]
fn result_mdo_equivalence_first_err() {
    let ma: Result<i32, String> = Err("e".to_string());
    let mb: Result<i32, String> = Ok(3);

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- ma_lhs;
        y <- mb_lhs;
        ResultKind::<String>::pure(x + y)
    };

    let rhs: Result<i32, String> = ResultKind::<String>::bind(ma, move |x| {
        ResultKind::<String>::bind(mb.clone(), move |y| ResultKind::<String>::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Err("e".to_string()));
}

/// `mb` is `Err`: both `mdo!` and the hand-written bind should return `Err("e")`.
#[test]
fn result_mdo_equivalence_second_err() {
    let ma: Result<i32, String> = Ok(7);
    let mb: Result<i32, String> = Err("e".to_string());

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- ma_lhs;
        y <- mb_lhs;
        ResultKind::<String>::pure(x + y)
    };

    let rhs: Result<i32, String> = ResultKind::<String>::bind(ma, move |x| {
        ResultKind::<String>::bind(mb.clone(), move |y| ResultKind::<String>::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Err("e".to_string()));
}

/// Both `ma` and `mb` are `Err`: first `Err` wins on both sides.
#[test]
fn result_mdo_equivalence_both_err() {
    let ma: Result<i32, String> = Err("first".to_string());
    let mb: Result<i32, String> = Err("second".to_string());

    let ma_lhs = ma.clone();
    let mb_lhs = mb.clone();
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- ma_lhs;
        y <- mb_lhs;
        ResultKind::<String>::pure(x + y)
    };

    let rhs: Result<i32, String> = ResultKind::<String>::bind(ma, move |x| {
        ResultKind::<String>::bind(mb.clone(), move |y| ResultKind::<String>::pure(x + y))
    });

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, Err("first".to_string()));
}

// ── 3b. Property-based equivalence ───────────────────────────────────────────
//
// Asserts the desugaring identity holds for 256 generated `(ma, mb)` pairs
// drawn from `arb_result_i32_string()`. Reuses the existing proptest harness.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// `mdo! { ResultKind::<String>; x <- ma; y <- mb; ResultKind::<String>::pure(x.wrapping_add(y)) }`
    /// must equal the hand-written nested bind for every `(ma, mb)`.
    #[test]
    fn result_mdo_equivalence_prop(
        ma in arb_result_i32_string(),
        mb in arb_result_i32_string(),
    ) {
        // Pre-clone for mdo! (which moves its inputs into closures) and rhs.
        let ma_lhs = ma.clone();
        let mb_lhs = mb.clone();

        let lhs: Result<i32, String> = mdo! {
            ResultKind::<String>;
            x <- ma_lhs;
            y <- mb_lhs;
            ResultKind::<String>::pure(x.wrapping_add(y))
        };

        let rhs: Result<i32, String> = ResultKind::<String>::bind(ma, move |x| {
            ResultKind::<String>::bind(mb.clone(), move |y| {
                ResultKind::<String>::pure(x.wrapping_add(y))
            })
        });

        prop_assert_eq!(lhs, rhs);
    }
}

// ── 4. `let` binding and bare-expr sequencing ─────────────────────────────────

/// A `let` binding inside an `mdo!` block introduces a pure local name.
#[test]
fn result_mdo_let_binding_inside_block() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(10);
        let doubled = x * 2;
        ResultKind::<String>::pure(doubled)
    };
    assert_eq!(result, Ok(20));
}

/// A `let` binding can reference multiple previously bound names.
#[test]
fn result_mdo_let_binding_combines_two_values() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(4);
        y <- Ok(6);
        let sum = x + y;
        ResultKind::<String>::pure(sum * 2)
    };
    assert_eq!(result, Ok(20));
}

/// A bare-expr sequencing line threads the monadic effect and discards the value;
/// subsequent bindings still see earlier bound names.
#[test]
fn result_mdo_bare_expr_sequencing_ok() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(5);
        Ok::<(), String>(());
        ResultKind::<String>::pure(x * 3)
    };
    assert_eq!(result, Ok(15));
}

/// A bare-expr sequencing line that returns `Err` short-circuits, just like a bind.
#[test]
fn result_mdo_bare_expr_sequencing_err_short_circuits() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(5);
        Err::<(), String>("bare-err".to_string());
        ResultKind::<String>::pure(x * 3)
    };
    assert_eq!(result, Err("bare-err".to_string()));
}

/// Combining `let` and bare-expr: both forms can appear together.
#[test]
fn result_mdo_let_and_bare_expr_combined() {
    let result: Result<String, String> = mdo! {
        ResultKind::<String>;
        x <- Ok::<i32, String>(7);
        let label = "value";
        Ok::<(), String>(());
        ResultKind::<String>::pure(format!("{}: {}", label, x))
    };
    assert_eq!(result, Ok("value: 7".to_string()));
}
