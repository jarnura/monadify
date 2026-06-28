//! Do-notation with WriterT: order/payment processing audit log example.
//!
//! Demonstrates accumulating a monoidal audit trail through an order/payment
//! pipeline using `mdo!` do-blocks and `Writer<Vec<String>, A>`.
//! Each pipeline step appends one log entry via `tell`; the logs are combined
//! monoidally by `bind` — no log vector is ever threaded through arguments.
//!
//! Run with: `cargo run --example writer_audit_log --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::writer::{MonadWriter, Writer, WriterTKind};

/// Type alias: a computation that accumulates a `Vec<String>` audit log and
/// produces a value `A`.
/// `Writer<W, A> = WriterT<W, IdentityKind, A>`.
type Logged<A> = Writer<Vec<String>, A>;

/// Kind marker alias for the `mdo!` macro: `WriterTKind<Vec<String>, IdentityKind>`.
type WKind = WriterTKind<Vec<String>, IdentityKind>;

// ── Helper wrappers ───────────────────────────────────────────────────────────

/// Appends a single audit entry to the log, yielding unit.
fn step(s: &str) -> Logged<()> {
    <WKind as MonadWriter<Vec<String>, (), IdentityKind>>::tell(vec![s.to_string()])
}

// ── Pipeline ─────────────────────────────────────────────────────────────────

/// Order/payment processing pipeline for order #42 totalling $100.
///
/// Three audit steps — validate, charge, ship — each append one log entry.
/// The final value is the captured amount in cents (`100u64`).
/// Each `_ <- tell` binds `()` (which is `Copy`), so no non-`Copy` capture
/// restriction is triggered across `mdo!` nesting levels.
fn pipeline() -> Logged<u64> {
    mdo! {
        WKind;
        _ <- step("validate: order #42 ok");
        _ <- step("charge: $100 captured");
        _ <- step("ship: order #42 dispatched");
        pure(100u64)
    }
}

fn main() {
    println!("=== Do-notation with WriterT: Order/Payment Audit Log ===\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: run_writer_t — both value and accumulated log
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 1: run_writer_t — value and full audit trail");
    let Identity((value, trail)) = pipeline().run_writer_t;
    assert_eq!(value, 100u64, "captured amount must be 100");
    assert_eq!(
        trail,
        vec![
            "validate: order #42 ok".to_string(),
            "charge: $100 captured".to_string(),
            "ship: order #42 dispatched".to_string(),
        ],
        "audit trail must contain exactly the three ordered entries"
    );
    println!("  Value (captured amount): {}", value);
    println!("  Audit trail:");
    for (i, entry) in trail.iter().enumerate() {
        println!("    [{}] {}", i + 1, entry);
    }
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: eval_writer_t — value only (log discarded)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 2: eval_writer_t — value only");
    let Identity(v) = pipeline().eval_writer_t();
    assert_eq!(v, 100, "eval_writer_t must return the captured amount 100");
    println!("  eval_writer_t -> {}", v);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: exec_writer_t — log only (value discarded)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 3: exec_writer_t — audit log only");
    let Identity(log) = pipeline().exec_writer_t();
    assert_eq!(
        log.len(),
        3,
        "exec_writer_t must return exactly 3 audit entries"
    );
    println!("  exec_writer_t -> {} entries", log.len());
    for entry in &log {
        println!("    - {}", entry);
    }
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `WriterT` accumulates the audit trail implicitly:");
    println!("  * `tell`          appends one log entry and yields unit.");
    println!("  * `bind`          sequences steps and combines their logs monoidally.");
    println!("  * `run_writer_t`  is a field exposing `Identity<(value, log)>` directly.");
    println!("  * `eval_writer_t` projects the produced value (log discarded).");
    println!("  * `exec_writer_t` projects the accumulated log  (value discarded).");
    println!("  * No log vector is ever threaded through any function argument.");
}
