//! Do-notation with State monad: running-balance ledger example.
//!
//! Demonstrates the State monad (`StateT<S, IdentityKind, A>`) as a
//! running-balance bank account. The state `i64` is a balance in cents;
//! `deposit` and `withdraw` update it via `modify`, while `balance` reads it
//! via `get`. The `mdo!` block threads the state implicitly — no explicit
//! passing required.
//!
//! Run with: `cargo run --example state_bank_account --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::state::{State, StateTKind};

/// Type alias: a computation that threads an `i64` balance (in cents) and
/// returns an `Identity<A>`. `State<S, A> = StateT<S, IdentityKind, A>`.
type Account<A> = State<i64, A>;

/// Kind marker alias for the `mdo!` macro: `StateTKind<i64, IdentityKind>`.
type SKind = StateTKind<i64, IdentityKind>;

// ── Ledger primitives ────────────────────────────────────────────────────────

/// Deposits `cents` into the account (increases the balance).
fn deposit(cents: i64) -> Account<()> {
    SKind::modify(move |b| b + cents)
}

/// Withdraws `cents` from the account (decreases the balance).
fn withdraw(cents: i64) -> Account<()> {
    SKind::modify(move |b| b - cents)
}

/// Reads the current balance without modifying it.
fn balance() -> Account<i64> {
    SKind::get()
}

// ── The ledger computation ───────────────────────────────────────────────────

/// A multi-step ledger session that returns `(mid, fin)`:
/// - `mid`: balance after the two deposits (before the withdrawal)
/// - `fin`: balance after the withdrawal
fn run_ledger() -> Account<(i64, i64)> {
    mdo! {
        SKind;
        _ <- deposit(10000);
        _ <- deposit(2550);
        mid <- balance();
        _ <- withdraw(4000);
        fin <- balance();
        pure((mid, fin))
    }
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Do-notation with State monad: Bank Account Ledger ===\n");

    let comp = run_ledger();

    // eval_state_t: run and keep only the produced value, discard final state.
    let Identity((mid, fin)) = comp.clone().eval_state_t(0);

    // exec_state_t: run and keep only the final threaded state.
    let Identity(final_state) = comp.clone().exec_state_t(0);

    // run_state_t: run and keep both value and final state.
    let Identity((pair_val, pair_state)) = (comp.run_state_t)(0);

    println!("Initial balance:              $0.00");
    println!("After deposit  $100.00:       ${:.2}", 10000_f64 / 100.0);
    println!(
        "After deposit  $25.50:        ${:.2}",
        (10000 + 2550) as f64 / 100.0
    );
    println!(
        "Intermediate balance (mid):   ${:.2}  ({} cents)",
        mid as f64 / 100.0,
        mid
    );
    println!(
        "After withdraw $40.00:        ${:.2}",
        (mid - 4000) as f64 / 100.0
    );
    println!(
        "Final balance (fin):          ${:.2}  ({} cents)",
        fin as f64 / 100.0,
        fin
    );
    println!(
        "exec_state_t final state:     ${:.2}  ({} cents)",
        final_state as f64 / 100.0,
        final_state
    );
    println!();

    // ── Assertions ───────────────────────────────────────────────────────────

    assert_eq!(
        mid, 12550,
        "intermediate balance after two deposits must be 12550 cents ($125.50)"
    );
    assert_eq!(
        fin, 8550,
        "final balance after withdrawal must be 8550 cents ($85.50)"
    );
    assert_eq!(
        final_state, 8550,
        "exec_state_t final state must match the read balance value"
    );

    // run_state_t returns the same value and state together.
    assert_eq!(pair_val, (mid, fin));
    assert_eq!(pair_state, fin);

    println!("All assertions passed.");
    println!();
    println!("Key insight: `mdo!` with `StateT` demonstrates implicit state threading:");
    println!("  deposit/withdraw call `modify` — each step's output state feeds the next.");
    println!("  balance calls `get` — reads without modifying the threaded state.");
    println!("  No explicit state parameter is passed; the monad wires it automatically.");
}
