//! Do-notation with StateT: deterministic LCG pseudo-random generator.
//!
//! Demonstrates the State monad threading a mutable `u64` seed through a
//! sequence of die rolls without any explicit seed-passing between steps.
//!
//! Run with: `cargo run --example state_rng_lcg --features do-notation`

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::state::{MonadState, State, StateTKind};

/// Type alias: a stateful computation that threads a `u64` LCG seed.
/// `State<S, A> = StateT<S, IdentityKind, A>`.
type Rng<A> = State<u64, A>;

/// Kind marker for the `mdo!` block header.
type SKind = StateTKind<u64, IdentityKind>;

// LCG constants (Numerical Recipes).
const MUL: u64 = 6364136223846793005;
const ADD: u64 = 1442695040888963407;

/// Advance the LCG seed and return the new seed as the produced value.
fn next_u64() -> Rng<u64> {
    <SKind as MonadState<u64, u64, IdentityKind>>::state(|seed| {
        let s2 = seed.wrapping_mul(MUL).wrapping_add(ADD);
        (s2, s2)
    })
}

/// Roll a six-sided die by mapping a raw `u64` output to `1..=6`.
fn roll_d6() -> Rng<u64> {
    SKind::bind(next_u64(), |raw| SKind::pure((raw % 6) + 1))
}

fn main() {
    println!("=== Do-notation with StateT: LCG Pseudo-Random Generator ===\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Build the computation: roll 5 dice using mdo! do-notation.
    // The seed threads implicitly through each roll — no explicit passing needed.
    // ─────────────────────────────────────────────────────────────────────────────
    let comp = mdo! {
        SKind;
        d1 <- roll_d6();
        d2 <- roll_d6();
        d3 <- roll_d6();
        d4 <- roll_d6();
        d5 <- roll_d6();
        pure(vec![d1, d2, d3, d4, d5])
    };

    // ─────────────────────────────────────────────────────────────────────────────
    // Run 1: eval_state_t — keep only the produced value, discard the final seed.
    // ─────────────────────────────────────────────────────────────────────────────
    let Identity(rolls) = comp.clone().eval_state_t(42);
    println!("Rolls from seed 42 (eval_state_t): {:?}", rolls);

    // Every face must be a valid die result.
    for &r in &rolls {
        assert!((1..=6).contains(&r), "die face out of range: {}", r);
    }
    println!("  All faces in 1..=6 PASSED");

    // Pin the deterministic sequence (pre-computed from the LCG formula at seed 42).
    // Each value is in 1..=6 and is fully determined by seed=42.
    assert_eq!(rolls, vec![6, 3, 6, 5, 6]);
    println!("  Pinned sequence PASSED");

    // ─────────────────────────────────────────────────────────────────────────────
    // Run 2: determinism check — same seed must yield identical rolls.
    // ─────────────────────────────────────────────────────────────────────────────
    let Identity(rolls2) = comp.clone().eval_state_t(42);
    assert_eq!(rolls, rolls2, "LCG must be deterministic");
    println!("  Determinism check PASSED");

    // ─────────────────────────────────────────────────────────────────────────────
    // Run 3: run_state_t — keep both value and final seed.
    // ─────────────────────────────────────────────────────────────────────────────
    let Identity((rolls3, final_seed)) = (comp.run_state_t)(42);
    println!("\nFull run (run_state_t):");
    println!("  Rolls        : {:?}", rolls3);
    println!("  Final seed   : {}", final_seed);
    assert_eq!(
        rolls3, rolls,
        "run_state_t value matches eval_state_t value"
    );
    println!("  run_state_t PASSED");

    println!("\n=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `StateTKind` threads the LCG seed");
    println!("  through each `roll_d6()` step automatically.  No seed argument");
    println!("  is ever passed explicitly — the State monad handles it.");
    println!("  Running the same computation from the same seed always produces");
    println!("  the same sequence, demonstrating referential transparency.");
}
