//! Order pipeline via `ExceptT`: validate → reserve stock → charge card,
//! aborting on the first failure.
//!
//! Demonstrates:
//! - `mdo!` over `Except<OrderError, _>` short-circuiting on the first `Err`.
//! - `throw_error` and `lift_either` from `MonadError`.
//! - `catch_error` for recovery from a failed pipeline.
//! - A thread-local counter proving `charge_card` is never invoked when an
//!   earlier stage fails (short-circuit guarantee).
//!
//! Run: `cargo run --quiet --example except_order_pipeline --features do-notation`

use std::cell::Cell;

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::except::{Except, ExceptTKind, MonadError};

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum OrderError {
    EmptyCart,
    OutOfStock,
    PaymentDeclined,
}

#[derive(Clone, PartialEq, Debug)]
struct Receipt {
    total: u32,
}

#[derive(Clone)]
struct Order {
    items: u32,
    in_stock: bool,
    card_ok: bool,
    total: u32,
}

// ── Kind aliases ──────────────────────────────────────────────────────────────

/// A computation that may fail with `OrderError` and produce a value of type `A`.
type Checked<A> = Except<OrderError, A>;

/// Kind marker for `mdo!` — `ExceptT<OrderError, IdentityKind, _>`.
type CKind = ExceptTKind<OrderError, IdentityKind>;

// ── Side-effect counter — proves charge_card never runs on early failure ───────

thread_local! {
    static CHARGE_COUNT: Cell<u32> = const { Cell::new(0) };
}

fn reset_counter() {
    CHARGE_COUNT.with(|c| c.set(0));
}

fn charge_count() -> u32 {
    CHARGE_COUNT.with(|c| c.get())
}

// ── Pipeline stage functions ──────────────────────────────────────────────────

fn validate_cart(o: Order) -> Checked<Order> {
    if o.items == 0 {
        <CKind as MonadError<OrderError, Order, IdentityKind>>::throw_error(OrderError::EmptyCart)
    } else {
        <CKind as MonadError<OrderError, Order, IdentityKind>>::lift_either(Ok(o))
    }
}

fn reserve_stock(o: Order) -> Checked<Order> {
    if !o.in_stock {
        <CKind as MonadError<OrderError, Order, IdentityKind>>::throw_error(OrderError::OutOfStock)
    } else {
        <CKind as MonadError<OrderError, Order, IdentityKind>>::lift_either(Ok(o))
    }
}

fn charge_card(o: Order) -> Checked<Receipt> {
    // Incremented at the START — a count of 0 proves this function never ran.
    CHARGE_COUNT.with(|c| c.set(c.get() + 1));
    if !o.card_ok {
        <CKind as MonadError<OrderError, Receipt, IdentityKind>>::throw_error(
            OrderError::PaymentDeclined,
        )
    } else {
        <CKind as MonadError<OrderError, Receipt, IdentityKind>>::lift_either(Ok(Receipt {
            total: o.total,
        }))
    }
}

// ── Order pipeline ────────────────────────────────────────────────────────────

/// Three-stage order pipeline in a `mdo!` block.
///
/// An `Err` from any stage is propagated immediately; all later stages are
/// skipped — they never execute.
fn run_pipeline(o: Order) -> Checked<Receipt> {
    mdo! {
        CKind;
        o2 <- validate_cart(o);
        o3 <- reserve_stock(o2);
        r  <- charge_card(o3);
        pure(r)
    }
}

/// Unwrap the `Identity` wrapper to obtain the inner `Result`.
fn run<A>(m: Checked<A>) -> Result<A, OrderError> {
    let Identity(r) = m.run_except_t;
    r
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== ExceptT Order Pipeline: short-circuit on first failure ===\n");

    // ── Test 1: valid order succeeds ──────────────────────────────────────────
    reset_counter();
    let valid = Order {
        items: 2,
        in_stock: true,
        card_ok: true,
        total: 150,
    };
    let result = run(run_pipeline(valid));
    assert_eq!(
        result,
        Ok(Receipt { total: 150 }),
        "valid order must produce Ok(Receipt)"
    );
    assert_eq!(
        charge_count(),
        1,
        "charge_card must run exactly once for a valid order"
    );
    println!("Test 1 PASS — valid order => Ok(Receipt {{ total: 150 }})");

    // ── Test 2: empty cart — charge_card must NOT run ─────────────────────────
    reset_counter();
    let empty = Order {
        items: 0,
        in_stock: true,
        card_ok: true,
        total: 50,
    };
    let result = run(run_pipeline(empty));
    assert_eq!(
        result,
        Err(OrderError::EmptyCart),
        "empty cart must fail with EmptyCart"
    );
    assert_eq!(
        charge_count(),
        0,
        "charge_card must NOT run when validate_cart fails (short-circuit)"
    );
    println!("Test 2 PASS — empty cart => Err(EmptyCart); charge_card count=0 (never called)");

    // ── Test 3: out of stock — charge_card must NOT run ───────────────────────
    reset_counter();
    let no_stock = Order {
        items: 3,
        in_stock: false,
        card_ok: true,
        total: 200,
    };
    let result = run(run_pipeline(no_stock));
    assert_eq!(
        result,
        Err(OrderError::OutOfStock),
        "no stock must fail with OutOfStock"
    );
    assert_eq!(
        charge_count(),
        0,
        "charge_card must NOT run when reserve_stock fails (short-circuit)"
    );
    println!("Test 3 PASS — out of stock => Err(OutOfStock); charge_card count=0 (never called)");

    // ── Test 4: payment declined ──────────────────────────────────────────────
    reset_counter();
    let declined = Order {
        items: 1,
        in_stock: true,
        card_ok: false,
        total: 75,
    };
    let result = run(run_pipeline(declined));
    assert_eq!(
        result,
        Err(OrderError::PaymentDeclined),
        "declined card must fail with PaymentDeclined"
    );
    assert_eq!(
        charge_count(),
        1,
        "charge_card must run (and then fail) when the card is declined"
    );
    println!("Test 4 PASS — declined card => Err(PaymentDeclined)");

    // ── Test 5: catch_error recovery ─────────────────────────────────────────
    reset_counter();
    let failing = Order {
        items: 0,
        in_stock: true,
        card_ok: true,
        total: 0,
    };
    let recovered = <CKind as MonadError<OrderError, Receipt, IdentityKind>>::catch_error(
        run_pipeline(failing),
        |_e| {
            <CKind as MonadError<OrderError, Receipt, IdentityKind>>::lift_either(Ok(Receipt {
                total: 0,
            }))
        },
    );
    assert_eq!(
        run(recovered),
        Ok(Receipt { total: 0 }),
        "catch_error must recover from EmptyCart with a fallback Receipt"
    );
    println!("Test 5 PASS — catch_error(EmptyCart) => Ok(Receipt {{ total: 0 }}) [recovery]");

    println!("\n=== All 5 tests passed! ===");
    println!();
    println!("Key insights:");
    println!("  mdo! with ExceptT short-circuits: an Err in any stage skips all later stages.");
    println!("  charge_card count=0 proves it was never entered when an earlier stage failed.");
    println!("  catch_error turns a failed pipeline into a fallback success computation.");
}
