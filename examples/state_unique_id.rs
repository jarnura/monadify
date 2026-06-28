//! Do-notation with StateT: fresh-id / gensym generator example.
//!
//! Demonstrates the State monad threading a mutable `u64` counter through a
//! multi-step computation, producing monotonically increasing unique identifiers
//! without any explicit counter passing at each step.
//!
//! Run with: `cargo run --example state_unique_id --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::state::{MonadState, State, StateTKind};

/// Type alias: a stateful computation that threads a `u64` counter and produces `A`.
/// `State<S, A>` = `StateT<S, IdentityKind, A>`.
type Gen<A> = State<u64, A>;

/// Kind marker alias used in the `mdo!` block header.
type SKind = StateTKind<u64, IdentityKind>;

/// Allocates a fresh id: returns the current counter as the id, then advances
/// the counter by one.  `state(|n| (n, n + 1))` means "value = n, new_state = n + 1".
fn fresh() -> Gen<u64> {
    <SKind as MonadState<u64, u64, IdentityKind>>::state(|n| (n, n + 1))
}

/// Assigns monotonically-increasing unique ids to the three labels in a single
/// `mdo!` block.  The `u64` ids are `Copy` and therefore cross bind levels freely;
/// the label `&str` literals are assembled in the terminal `pure(...)`.
fn assign_ids() -> Gen<Vec<(&'static str, u64)>> {
    mdo! {
        SKind;
        a <- fresh();
        b <- fresh();
        c <- fresh();
        pure(vec![("alpha", a), ("beta", b), ("gamma", c)])
    }
}

fn main() {
    println!("=== Do-notation with StateT: Fresh-Id / Gensym Generator ===\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 1: run_state_t — inspect both the produced value and the final counter.
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 1: run_state_t — value and final counter together");
    let comp = assign_ids();
    let Identity((ids, final_n)) = (comp.run_state_t)(0);
    println!("  Assigned ids : {:?}", ids);
    println!("  Final counter: {}", final_n);
    assert_eq!(
        ids,
        vec![("alpha", 0), ("beta", 1), ("gamma", 2)],
        "ids must be 0-based and monotonically increasing"
    );
    assert_eq!(final_n, 3, "counter must advance once per fresh()");
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 2: eval_state_t — project only the produced value.
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 2: eval_state_t — project only the assigned ids");
    let comp2 = assign_ids();
    let Identity(ids2) = comp2.eval_state_t(0);
    println!("  Assigned ids: {:?}", ids2);
    assert_eq!(ids2, vec![("alpha", 0), ("beta", 1), ("gamma", 2)]);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 3: exec_state_t — project only the final counter.
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 3: exec_state_t — project only the final counter");
    let comp3 = assign_ids();
    let Identity(final_counter) = comp3.exec_state_t(0);
    println!("  Final counter: {}", final_counter);
    assert_eq!(final_counter, 3, "three fresh() calls must advance to 3");
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 4: first id is 0 — counter is 0-based.
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 4: first fresh() id is 0 (0-based counter)");
    let first_id_comp = fresh();
    let Identity((first_id, next_counter)) = (first_id_comp.run_state_t)(0);
    println!("  First id     : {}", first_id);
    println!("  Next counter : {}", next_counter);
    assert_eq!(first_id, 0, "first id must be 0");
    assert_eq!(
        next_counter, 1,
        "counter must advance to 1 after one fresh()"
    );
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `StateT` threads the counter automatically:");
    println!("  Each `fresh()` call reads the current counter as the id and returns");
    println!("  new_state = counter + 1.  The `mdo!` desugaring chains the three");
    println!("  calls via nested `bind`s so the counter is never passed by hand.");
    println!("  The u64 ids are Copy, so they cross bind-levels freely in the block.");
}
