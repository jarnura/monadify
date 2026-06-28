//! Do-notation with WriterT: arithmetic expression evaluator with step-by-step trace.
//!
//! Demonstrates accumulating a textual trace through an arithmetic evaluation
//! using `mdo!` do-blocks and `Writer<String, A>`.
//! Each arithmetic step emits a human-readable trace line via `tell`; the log
//! is combined monoidally (string concatenation) by `bind` — no trace string is
//! ever threaded through function arguments.
//!
//! Run with: `cargo run --example writer_eval_trace --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::writer::{MonadWriter, Writer, WriterTKind};

/// Type alias: a computation that accumulates a `String` trace log and
/// produces a value `A`.
/// `Writer<W, A> = WriterT<W, IdentityKind, A>`.
type Traced<A> = Writer<String, A>;

/// Kind marker alias for the `mdo!` macro: `WriterTKind<String, IdentityKind>`.
type WKind = WriterTKind<String, IdentityKind>;

// ── Helper wrappers ───────────────────────────────────────────────────────────

/// Appends a single trace line to the log, yielding unit.
fn trace(line: String) -> Traced<()> {
    <WKind as MonadWriter<String, (), IdentityKind>>::tell(line)
}

// ── Step combinators ─────────────────────────────────────────────────────────

/// Adds two integers and emits a trace line recording the operation.
fn add(a: i64, b: i64) -> Traced<i64> {
    mdo! { WKind;
        _ <- trace(format!("add {} + {} = {}\n", a, b, a + b));
        pure(a + b)
    }
}

/// Multiplies two integers and emits a trace line recording the operation.
fn mul(a: i64, b: i64) -> Traced<i64> {
    mdo! { WKind;
        _ <- trace(format!("mul {} * {} = {}\n", a, b, a * b));
        pure(a * b)
    }
}

// ── Evaluator ─────────────────────────────────────────────────────────────────

/// Evaluates `(2 + 3) * 4` and accumulates a step-by-step trace.
///
/// `s` and `p` are `i64` (Copy), so they cross `mdo!` nesting levels freely
/// without triggering the non-Copy capture restriction.
fn eval() -> Traced<i64> {
    mdo! { WKind;
        s <- add(2, 3);
        p <- mul(s, 4);
        pure(p)
    }
}

fn main() {
    println!("=== Do-notation with WriterT: Arithmetic Expression Evaluator ===\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: eval_writer_t — value only (trace discarded)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 1: eval_writer_t — produced value only");
    let Identity(value) = eval().eval_writer_t();
    assert_eq!(value, 20, "eval_writer_t must return 20 for (2+3)*4");
    println!("  eval_writer_t -> {} (expected 20)", value);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: exec_writer_t — trace log only (value discarded)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 2: exec_writer_t — accumulated trace log only");
    let Identity(trace_log) = eval().exec_writer_t();
    assert_eq!(
        trace_log, "add 2 + 3 = 5\nmul 5 * 4 = 20\n",
        "exec_writer_t must return the two-line trace in evaluation order"
    );
    println!("  exec_writer_t trace:");
    for line in trace_log.lines() {
        println!("    {}", line);
    }
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: run_writer_t — both value and trace in one shot
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 3: run_writer_t — value and trace together");
    let Identity((result, full_trace)) = eval().run_writer_t;
    assert_eq!(result, 20, "run_writer_t value must be 20");
    assert_eq!(
        full_trace, "add 2 + 3 = 5\nmul 5 * 4 = 20\n",
        "run_writer_t trace must match the two-line evaluation log"
    );
    println!("  Result: {}", result);
    println!("  Full trace:\n{}", full_trace);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `WriterT<String, _>` accumulates a step-by-step trace:");
    println!("  * `tell`          appends one trace line and yields unit.");
    println!("  * `bind`          sequences steps and concatenates their logs monoidally.");
    println!("  * `run_writer_t`  is a field exposing `Identity<(value, log)>` directly.");
    println!("  * `eval_writer_t` projects the produced value  (trace discarded).");
    println!("  * `exec_writer_t` projects the accumulated log (value discarded).");
    println!("  * No trace string is ever threaded through any function argument.");
}
