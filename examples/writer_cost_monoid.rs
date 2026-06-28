//! Do-notation with WriterT: query-plan cost accumulation via a user-defined monoid.
//!
//! Demonstrates accumulating a running cost through a custom `Sum(u64)` monoid
//! defined in this file — a "bring-your-own monoidal log" example using
//! `mdo!` do-blocks and `Writer<Sum, A>`.
//! Each `charge(n)` step silently appends its cost to the monoidal log; no
//! explicit accumulator is threaded through any function argument.
//!
//! Run with: `cargo run --example writer_cost_monoid --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monoid::{Monoid, Semigroup};
use monadify::transformers::writer::{Writer, WriterTKind};

// ── User-defined monoid ───────────────────────────────────────────────────────

/// A newtype wrapper around `u64` that forms a monoid under wrapping addition.
///
/// `combine` is associative (`(a+b)+c == a+(b+c)` mod 2^64), and `Sum(0)` is
/// the identity (`Sum(0).combine(x) == x`, `x.combine(Sum(0)) == x`).
/// `WriterT` requires `W: Monoid + Clone`; both are satisfied because `Sum` is
/// `Copy` (and hence `Clone` for free).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Sum(u64);

impl Semigroup for Sum {
    fn combine(self, other: Sum) -> Sum {
        Sum(self.0.wrapping_add(other.0))
    }
}

impl Monoid for Sum {
    fn empty() -> Sum {
        Sum(0)
    }
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A computation that accumulates a `Sum` cost log and produces `A`.
/// `Writer<W, A> = WriterT<W, IdentityKind, A>`.
type Costed<A> = Writer<Sum, A>;

/// Kind marker alias for the `mdo!` macro: `WriterTKind<Sum, IdentityKind>`.
type WKind = WriterTKind<Sum, IdentityKind>;

// ── Helper wrapper ────────────────────────────────────────────────────────────

/// Record a cost of `n` units, appending `Sum(n)` to the log and yielding unit.
fn charge(n: u64) -> Costed<()> {
    WKind::tell(Sum(n))
}

// ── Programs ──────────────────────────────────────────────────────────────────

/// Model a simple query plan: scan(5) → filter(2) → sort(10) → project(1).
///
/// Each operator charges its cost via `charge`; the log accumulates the total.
fn plan() -> Costed<&'static str> {
    mdo! {
        WKind;
        _ <- charge(5);   // scan
        _ <- charge(2);   // filter
        _ <- charge(10);  // sort
        _ <- charge(1);   // project
        pure("query result")
    }
}

fn main() {
    println!("=== Do-notation with WriterT: Query-Plan Cost Accumulation ===\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: exec_writer_t — accumulated cost log only
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 1: exec_writer_t — total cost (scan=5, filter=2, sort=10, project=1)");
    let Identity(total) = plan().exec_writer_t();
    assert_eq!(total, Sum(18), "total cost must be Sum(18)");
    println!("  Total cost: {:?} (expected Sum(18))", total);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: eval_writer_t — produced value only (log discarded)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 2: eval_writer_t — produced value (log discarded)");
    let Identity(v) = plan().eval_writer_t();
    assert_eq!(v, "query result", "produced value must be \"query result\"");
    println!("  Value: {:?} (expected \"query result\")", v);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: run_writer_t — both value and log together
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 3: run_writer_t — value and accumulated log together");
    let Identity((result, cost)) = plan().run_writer_t;
    assert_eq!(result, "query result");
    assert_eq!(cost, Sum(18));
    println!("  (value, cost): ({:?}, {:?})", result, cost);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `WriterT` accumulates the cost log implicitly:");
    println!("  * `tell` (via `charge`) appends a `Sum(n)` to the log and yields unit.");
    println!("  * `Semigroup::combine` (`wrapping_add`) stitches logs together at each bind.");
    println!("  * `Monoid::empty` (`Sum(0)`) seeds the log in `pure`.");
    println!("  * `exec_writer_t` extracts only the accumulated log: Identity<Sum>.");
    println!("  * `eval_writer_t` extracts only the produced value: Identity<&str>.");
    println!("  * No explicit accumulator is threaded through any function argument.");
}
