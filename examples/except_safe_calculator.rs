//! Safe calculator using the `Except` monad for domain-error recovery.
//!
//! Demonstrates `throw_error`, `lift_either`, and `catch_error` with
//! `ExceptT` over `IdentityKind`.  Division-by-zero and square-root-of-negative
//! are modelled as typed errors; `catch_error` intercepts them and substitutes
//! a fallback value without touching successful computations.
//!
//! Run with: `cargo run --example except_safe_calculator --features do-notation`

use monadify::monad::kind::Bind;
use monadify::transformers::except::{Except, ExceptTKind, MonadError};
use monadify::{Identity, IdentityKind};

// ── Error type ────────────────────────────────────────────────────────────────

/// Domain errors that the safe calculator can raise.
#[derive(Clone, PartialEq, Debug)]
enum MathError {
    /// Attempted to divide by zero.
    DivByZero,
    /// Attempted to take the square root of a negative number.
    NegativeSqrt,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A computation that either produces an `A` or raises a `MathError`.
/// `Except<E, A> = ExceptT<E, IdentityKind, A>`.
type Checked<A> = Except<MathError, A>;

/// Kind marker for `ExceptT<MathError, IdentityKind, _>`.
type CKind = ExceptTKind<MathError, IdentityKind>;

// ── Smart constructors ────────────────────────────────────────────────────────

/// Divides `a` by `b`.  Throws `DivByZero` when `b == 0.0`.
fn safe_div(a: f64, b: f64) -> Checked<f64> {
    if b == 0.0 {
        <CKind as MonadError<MathError, f64, IdentityKind>>::throw_error(MathError::DivByZero)
    } else {
        <CKind as MonadError<MathError, f64, IdentityKind>>::lift_either(Ok(a / b))
    }
}

/// Takes the square root of `x`.  Throws `NegativeSqrt` when `x < 0.0`.
fn safe_sqrt(x: f64) -> Checked<f64> {
    if x < 0.0 {
        <CKind as MonadError<MathError, f64, IdentityKind>>::throw_error(MathError::NegativeSqrt)
    } else {
        <CKind as MonadError<MathError, f64, IdentityKind>>::lift_either(Ok(x.sqrt()))
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Except monad: safe calculator with error recovery ===\n");

    // ── Test 1: safe_div success ───────────────────────────────────────────────
    println!("Test 1: safe_div(10.0, 2.0) — success path");
    let Identity(res1) = safe_div(10.0, 2.0).run_except_t;
    assert_eq!(res1, Ok(5.0), "expected Ok(5.0), got {:?}", res1);
    println!("  {:?}  PASSED\n", res1);

    // ── Test 2: safe_div throws DivByZero ─────────────────────────────────────
    println!("Test 2: safe_div(1.0, 0.0) — divide-by-zero error");
    let Identity(res2) = safe_div(1.0, 0.0).run_except_t;
    assert_eq!(
        res2,
        Err(MathError::DivByZero),
        "expected Err(DivByZero), got {:?}",
        res2
    );
    println!("  {:?}  PASSED\n", res2);

    // ── Test 3: safe_sqrt throws NegativeSqrt ─────────────────────────────────
    println!("Test 3: safe_sqrt(-4.0) — negative-sqrt error");
    let Identity(res3) = safe_sqrt(-4.0).run_except_t;
    assert_eq!(
        res3,
        Err(MathError::NegativeSqrt),
        "expected Err(NegativeSqrt), got {:?}",
        res3
    );
    println!("  {:?}  PASSED\n", res3);

    // ── Test 4: catch_error recovers DivByZero with fallback 0.0 ──────────────
    println!("Test 4: catch_error(safe_div(1.0, 0.0), |_| Ok(0.0)) — recovery");
    let recovered = <CKind as MonadError<MathError, f64, IdentityKind>>::catch_error(
        safe_div(1.0, 0.0),
        |_e| <CKind as MonadError<MathError, f64, IdentityKind>>::lift_either(Ok(0.0)),
    );
    let Identity(res4) = recovered.run_except_t;
    assert_eq!(res4, Ok(0.0), "expected Ok(0.0), got {:?}", res4);
    println!("  {:?}  PASSED\n", res4);

    // ── Test 5: catch_error over an Ok computation passes through unchanged ────
    println!("Test 5: catch_error(safe_div(10.0, 2.0), handler) — Ok passes through");
    let passthrough = <CKind as MonadError<MathError, f64, IdentityKind>>::catch_error(
        safe_div(10.0, 2.0),
        |_e| {
            // Handler must NOT be invoked — if it were, an assertion below would
            // catch the wrong value (the fallback -1.0).
            <CKind as MonadError<MathError, f64, IdentityKind>>::lift_either(Ok(-1.0))
        },
    );
    let Identity(res5) = passthrough.run_except_t;
    assert_eq!(
        res5,
        Ok(5.0),
        "expected Ok(5.0) (handler must not fire), got {:?}",
        res5
    );
    println!("  {:?}  PASSED\n", res5);

    // ── Test 6: bind sequences two safe operations (exact: sqrt(4.0) == 2.0) ──
    println!("Test 6: bind — sqrt(safe_div(16.0, 4.0)) == sqrt(4.0) == 2.0");
    let chained = CKind::bind(safe_div(16.0, 4.0), safe_sqrt);
    let Identity(res6) = chained.run_except_t;
    assert_eq!(res6, Ok(2.0), "expected Ok(2.0), got {:?}", res6);
    println!("  {:?}  PASSED\n", res6);

    // ── Test 7: bind short-circuits — DivByZero skips the sqrt step ───────────
    println!("Test 7: bind — DivByZero short-circuits, sqrt step never runs");
    let short_circuit = CKind::bind(safe_div(1.0, 0.0), safe_sqrt);
    let Identity(res7) = short_circuit.run_except_t;
    assert_eq!(
        res7,
        Err(MathError::DivByZero),
        "expected Err(DivByZero) (short-circuit), got {:?}",
        res7
    );
    println!("  {:?}  PASSED\n", res7);

    // ── Summary ───────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight:");
    println!("  * throw_error  injects an error and short-circuits the computation.");
    println!("  * lift_either  embeds a pure Result into the Except monad.");
    println!("  * catch_error  intercepts Err and runs a recovery handler;");
    println!("                 Ok values pass through with the handler never called.");
    println!("  * bind         sequences steps, propagating Err without calling");
    println!("                 the continuation — total short-circuit semantics.");
}
