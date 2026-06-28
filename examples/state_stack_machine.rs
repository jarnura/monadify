//! Do-notation with StateT: RPN / postfix stack calculator example.
//!
//! Demonstrates threading a mutable operand stack through an RPN (Reverse Polish
//! Notation) calculator using `mdo!` do-blocks and `State<Vec<i64>, A>`.
//! Each stack operation is a pure state transition; no explicit stack is ever
//! passed through function arguments.
//!
//! Run with: `cargo run --example state_stack_machine --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::state::{State, StateTKind};

/// Type alias: a computation that threads a `Vec<i64>` operand stack and produces `A`.
/// `State<S, A> = StateT<S, IdentityKind, A>`.
type Stack<A> = State<Vec<i64>, A>;

/// Kind marker alias for the `mdo!` macro: `StateTKind<Vec<i64>, IdentityKind>`.
type SKind = StateTKind<Vec<i64>, IdentityKind>;

// ── Helper wrappers (mirror the style of reader_config.rs) ───────────────────

/// Push an operand onto the stack (returns unit; modifies state).
fn push(n: i64) -> Stack<()> {
    SKind::modify(move |mut st| {
        st.push(n);
        st
    })
}

/// Apply a binary operator: pop two operands (right then left), push one result.
fn binary_op(op: impl Fn(i64, i64) -> i64 + 'static) -> Stack<()> {
    SKind::state(move |mut st| {
        let b = st.pop().expect("stack underflow: missing right operand");
        let a = st.pop().expect("stack underflow: missing left operand");
        st.push(op(a, b));
        ((), st)
    })
}

/// Addition: pop two operands, push their sum.
fn add_op() -> Stack<()> {
    binary_op(|a, b| a + b)
}

/// Multiplication: pop two operands, push their product.
fn mul_op() -> Stack<()> {
    binary_op(|a, b| a * b)
}

/// Read the top of the stack without consuming it (state is left unchanged).
fn peek_top() -> Stack<i64> {
    SKind::gets(|st| *st.last().expect("peek_top: empty stack"))
}

// ── Programs ─────────────────────────────────────────────────────────────────

/// Evaluate the sub-expression `3 4 +` (used for the intermediate check).
fn prog_add() -> Stack<()> {
    mdo! {
        SKind;
        _ <- push(3);
        _ <- push(4);
        _ <- add_op();
        pure(())
    }
}

/// Evaluate the full postfix program `3 4 + 5 *`.
///
/// Returns `(intermediate_sum, final_product)` so both can be asserted in
/// `main` from a single execution. Both values are `i64` (Copy), so they cross
/// `mdo!` nesting levels without triggering the non-Copy capture restriction.
fn prog_full() -> Stack<(i64, i64)> {
    mdo! {
        SKind;
        _ <- push(3);
        _ <- push(4);
        _ <- add_op();
        mid    <- peek_top();   // mid   : i64 (Copy) — safe across bind levels
        _ <- push(5);
        _ <- mul_op();
        result <- peek_top();   // result: i64 (Copy)
        pure((mid, result))
    }
}

fn main() {
    println!("=== Do-notation with StateT: RPN Stack Calculator ===\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: Intermediate check — `3 4 +` leaves stack top = 7
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 1: Intermediate check — `3 4 +`");
    let Identity(stack_after_add) = prog_add().exec_state_t(vec![]);
    let top_after_add = *stack_after_add.last().expect("stack must be non-empty");
    assert_eq!(top_after_add, 7, "stack top after `3 4 +` must be 7");
    println!("  Stack after `3 4 +`: {:?}", stack_after_add);
    println!("  Stack top = {} (expected 7)", top_after_add);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: Full program `3 4 + 5 *` — value = 35, final stack = [35]
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 2: Full program `3 4 + 5 *` (expected 35)");
    let Identity(((mid, result), final_stack)) = (prog_full().run_state_t)(vec![]);
    assert_eq!(mid, 7, "intermediate value after `3 4 +` must be 7");
    assert_eq!(result, 35, "`3 4 + 5 *` must evaluate to 35");
    assert_eq!(final_stack, vec![35i64], "final stack must be exactly [35]");
    println!("  Intermediate sum (3+4) captured mid-program: {}", mid);
    println!("  Final product (7*5): {}", result);
    println!("  Final stack: {:?}", final_stack);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: Runners — eval_state_t (value only) and exec_state_t (state only)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Test 3: Runners — eval_state_t and exec_state_t");
    let Identity(value_only) = prog_full().eval_state_t(vec![]);
    let Identity(state_only) = prog_full().exec_state_t(vec![]);
    assert_eq!(
        value_only,
        (7, 35),
        "eval_state_t must return (mid=7, result=35)"
    );
    assert_eq!(
        state_only,
        vec![35i64],
        "exec_state_t must return final stack [35]"
    );
    println!("  eval_state_t -> {:?}", value_only);
    println!("  exec_state_t -> {:?}", state_only);
    println!("  PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `StateT` threads the operand stack implicitly:");
    println!("  * `modify` handles push: transforms the state without yielding a value.");
    println!("  * `state`  handles binary ops: pops two operands, pushes one result.");
    println!("  * `gets`   peeks at the top of the stack (read-only state projection).");
    println!("  * `run_state_t` returns both the produced value and the final state.");
    println!("  * No explicit stack is threaded through any function argument.");
}
