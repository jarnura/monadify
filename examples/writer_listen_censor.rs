//! Do-notation with WriterT: scoped + redacting logger demo.
//!
//! Demonstrates `listen` and `censor` from `MonadWriter` using `mdo!` do-blocks
//! and `Writer<Vec<String>, A>`.
//! - `listen` exposes a computation's accumulated log as a value while leaving
//!   the same log in the outer log.
//! - `censor` rewrites a computation's log with a pure function, leaving the
//!   value unchanged. Used here to redact sensitive entries before they appear
//!   in the outer log.
//!
//! Run with: `cargo run --example writer_listen_censor --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::writer::{MonadWriter, Writer, WriterTKind};

/// Type alias: a computation that accumulates a `Vec<String>` log and produces `A`.
/// `Writer<W, A> = WriterT<W, IdentityKind, A>`.
type Logged<A> = Writer<Vec<String>, A>;

/// Kind marker alias for the `mdo!` macro: `WriterTKind<Vec<String>, IdentityKind>`.
type WKind = WriterTKind<Vec<String>, IdentityKind>;

// ── Helper ────────────────────────────────────────────────────────────────────

/// Appends a single log entry, yielding unit.
fn step(s: &str) -> Logged<()> {
    <WKind as MonadWriter<Vec<String>, (), IdentityKind>>::tell(vec![s.to_string()])
}

// ── Shared sub-computation ────────────────────────────────────────────────────

/// Auth sub-computation: emits three log entries and yields unit.
fn auth_steps() -> Logged<()> {
    mdo! {
        WKind;
        _ <- step("auth: begin");
        _ <- step("auth: password=hunter2");
        _ <- step("auth: ok");
        pure(())
    }
}

// ── Demos ─────────────────────────────────────────────────────────────────────

/// Listen demo: captures the auth log as a value, exposing it alongside the
/// outer log (which also contains the auth log after `listen`).
fn listen_demo() -> Logged<Vec<String>> {
    mdo! {
        WKind;
        captured <- <WKind as MonadWriter<Vec<String>, (), IdentityKind>>::listen(auth_steps());
        pure(captured.1)
    }
}

/// Censor demo: redacts the password entry in the auth log, then appends a
/// final "main: done" entry from the surrounding context.
fn censor_demo() -> Logged<()> {
    mdo! {
        WKind;
        let redact = |log: Vec<String>| -> Vec<String> {
            log.into_iter()
                .map(|l| {
                    if l.contains("password") {
                        "auth: password=***REDACTED***".to_string()
                    } else {
                        l
                    }
                })
                .collect()
        };
        _ <- <WKind as MonadWriter<Vec<String>, (), IdentityKind>>::censor(redact, auth_steps());
        _ <- step("main: done");
        pure(())
    }
}

fn main() {
    println!("=== Do-notation with WriterT: Scoped + Redacting Logger ===\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: listen — captured log equals the outer log
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 1: listen — captured log exposed as value, same in outer log");
    let Identity((listened, outer_log)) = listen_demo().run_writer_t;
    let expected_auth_log = vec![
        "auth: begin".to_string(),
        "auth: password=hunter2".to_string(),
        "auth: ok".to_string(),
    ];
    assert_eq!(
        listened, expected_auth_log,
        "listen must capture the full auth log as the returned value"
    );
    assert_eq!(
        outer_log, listened,
        "listen must also leave the same log in the outer log"
    );
    println!("  Captured log (value):  {:?}", listened);
    println!("  Outer log:             {:?}", outer_log);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: censor — password entry redacted, outer steps preserved
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 2: censor — password entry redacted, outer log intact");
    let Identity(final_log) = censor_demo().exec_writer_t();
    assert_eq!(
        final_log,
        vec![
            "auth: begin".to_string(),
            "auth: password=***REDACTED***".to_string(),
            "auth: ok".to_string(),
            "main: done".to_string(),
        ],
        "censor must redact the password entry and preserve the outer step"
    );
    println!("  Final log:");
    for (i, entry) in final_log.iter().enumerate() {
        println!("    [{}] {}", i + 1, entry);
    }
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `listen` and `censor` are dual log-inspection primitives:");
    println!("  * `listen(m)`     — exposes m's accumulated log as the returned value");
    println!("                      while also leaving that same log in the outer log.");
    println!("  * `censor(f, m)`  — rewrites m's log with f, value unchanged;");
    println!("                      the outer log sees only the rewritten entries.");
    println!("  * `run_writer_t`  — a field: `Identity((value, log))`, destructure directly.");
    println!("  * `exec_writer_t` — a method: returns `Identity<W>` (log only).");
}
