//! A small expression-language interpreter built on a **four-deep monad
//! transformer stack** — every layer is genuinely load-bearing:
//!
//! - **Reader** (`Env`)   — the variable environment (`name -> value`).
//! - **State** (`i64`)    — an evaluation-step counter, bumped at each node.
//! - **Writer** (`Vec<String>`) — a post-order trace of evaluated subexpressions.
//! - **Except** (`EvalError`)   — `UnboundVariable` / `DivByZero` errors.
//!
//! The stack, innermost to outermost:
//! `ExceptT<EvalError, Identity>` ⟶ `WriterT<Log, _>` ⟶ `StateT<i64, _>`
//! ⟶ `ReaderT<Env, _>`. Running peels the layers back, yielding
//! `Result<((value, final_step), trace), EvalError>`.
//!
//! Because `ExceptT` is the **innermost** layer, a thrown error DISCARDS the
//! accumulated state and trace — you get `Err(e)`, never a partial
//! `((value, state), log)`. That ordering is the key teaching point.
//!
//! Run: `cargo run --quiet --example interpreter_full_stack --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::except::{ExceptT, ExceptTKind};
use monadify::transformers::reader::{ReaderT, ReaderTKind};
use monadify::transformers::state::StateTKind;
use monadify::transformers::writer::WriterTKind;

// ── Domain types ──────────────────────────────────────────────────────────────

/// Evaluation errors thrown by the interpreter (the `Except` layer).
#[derive(Clone, PartialEq, Debug)]
enum EvalError {
    /// A `Var` referenced a name absent from the environment.
    UnboundVariable(String),
    /// A `Div` had a zero divisor.
    DivByZero,
}

/// The variable environment (the `Reader` layer). NOT `Copy` — it holds `String`
/// keys; the `Var` arm works around the one-non-Copy-per-level rule with
/// `let`-bound clones inside the `mdo!` block.
#[derive(Clone, PartialEq, Debug)]
struct Env {
    bindings: Vec<(String, i64)>,
}

impl Env {
    fn new(bindings: &[(&str, i64)]) -> Self {
        Env {
            bindings: bindings.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }
    fn lookup(&self, name: &str) -> Option<i64> {
        self.bindings
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| *v)
    }
}

/// The accumulated trace (the `Writer` layer, a `Vec<String>` monoid).
type Log = Vec<String>;

/// The expression AST evaluated by `eval`.
#[derive(Clone, PartialEq, Debug)]
enum Expr {
    Lit(i64),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

// ── The four-deep transformer stack ─────────────────────────────────────────────

/// Innermost: short-circuiting errors over the identity base monad.
type ExceptLayerKind = ExceptTKind<EvalError, IdentityKind>;
/// Next: a monoidal trace accumulated above the error layer.
type WriterLayerKind = WriterTKind<Log, ExceptLayerKind>;
/// Next: a threaded step counter above the writer layer.
type StateLayerKind = StateTKind<i64, WriterLayerKind>;
/// Outermost: a read-only variable environment — the full application monad.
type AppKind = ReaderTKind<Env, StateLayerKind>;

/// A computation in the full stack producing an `A`.
type App<A> = ReaderT<Env, StateLayerKind, A>;

// ── Lifting helpers (NO auto-lifting: lift each primitive to the top) ───────────

/// `Reader`: read the whole variable environment.
fn ask_env() -> App<Env> {
    AppKind::ask()
}

/// `State`: read the current step counter.
fn get_step() -> App<i64> {
    AppKind::lift(StateLayerKind::get())
}

/// `State`: overwrite the step counter.
fn put_step(n: i64) -> App<()> {
    AppKind::lift(StateLayerKind::put(n))
}

/// `Writer`: append one entry to the trace (lifted two layers down).
fn tell_log(msg: String) -> App<()> {
    AppKind::lift(StateLayerKind::lift(WriterLayerKind::tell(vec![msg])))
}

/// `Except`: abort the whole computation with an error (lifted three layers down).
fn throw_err(e: EvalError) -> App<i64> {
    let thrown: ExceptT<EvalError, IdentityKind, i64> = ExceptT::throw(e);
    AppKind::lift(StateLayerKind::lift(WriterLayerKind::lift(thrown)))
}

/// Bump the step counter and record one trace entry for the given node label.
/// Uses `mdo!` to sequence the `State` and `Writer` effects.
///
/// `n` is `i64` (`Copy`) so it crosses the second bind level freely.
/// `msg` is created by a `let` binding inside the block so the inner
/// `move` closure owns a fresh `String` and `.clone()` keeps it `FnMut`.
fn visit(label: String) -> App<()> {
    mdo! { AppKind;
        n <- get_step();
        let msg = format!("step {}: {}", n + 1, label);
        _ <- put_step(n + 1);
        tell_log(msg.clone())
    }
}

// ── The interpreter ─────────────────────────────────────────────────────────────

/// Thin owned-argument wrapper so `eval` can be called from inside `mdo!`
/// `move` closures that hold an `Expr` by value.
fn eval_owned(e: Expr) -> App<i64> {
    eval(&e)
}

/// Evaluate `expr` in the full stack using `mdo!` do-notation.  Traversal is
/// **post-order**: each node's trace entry is emitted *after* all its children
/// have been evaluated, so inner nodes appear later in the trace than leaves.
///
/// Every node bumps the step counter (State), records a trace entry (Writer),
/// looks variables up in the environment (Reader), and short-circuits on an
/// unbound variable or a zero divisor (Except).
fn eval(expr: &Expr) -> App<i64> {
    match expr {
        // Lit: leaf node — just visit, then return the literal.
        // `n` is `i64` (Copy), so it crosses the bind level freely.
        Expr::Lit(n) => {
            let n = *n;
            mdo! { AppKind;
                _ <- visit(format!("Lit({n})"));
                pure(n)
            }
        }

        // Var: visit first, then look up in the environment.
        // `name` and `env` are both non-Copy; `let` bindings inside the
        // `mdo!` block clone them at the right nesting level so the outer
        // `FnMut` closure never has to move them out (which would be E0507).
        Expr::Var(name) => {
            let name = name.clone();
            let label = format!("Var({name})");
            mdo! { AppKind;
                _ <- visit(label);
                let lookup_key = name.clone();
                let err = EvalError::UnboundVariable(name.clone());
                env <- ask_env();
                match env.lookup(&lookup_key) {
                    Some(v) => pure(v),
                    None => throw_err(err.clone()),
                }
            }
        }

        // Add: post-order — evaluate children first, then visit the node.
        // `lv` and `rv` are `i64` (Copy) and cross bind levels freely.
        // `r.clone()` is load-bearing: moving the FnMut-captured `r` out
        // would be E0507.
        Expr::Add(l, r) => {
            let l = (**l).clone();
            let r = (**r).clone();
            mdo! { AppKind;
                lv <- eval_owned(l);
                rv <- eval_owned(r.clone());
                _ <- visit("Add".into());
                pure(lv + rv)
            }
        }

        // Div: post-order — evaluate children first, visit, then check divisor.
        // Same `r.clone()` rationale as `Add`.
        Expr::Div(l, r) => {
            let l = (**l).clone();
            let r = (**r).clone();
            mdo! { AppKind;
                lv <- eval_owned(l);
                rv <- eval_owned(r.clone());
                _ <- visit("Div".into());
                if rv == 0 { throw_err(EvalError::DivByZero) } else { pure(lv / rv) }
            }
        }
    }
}

/// Run a program through all four layers, peeling them back to the bare result.
/// `Reader`(env) -> `State`(s0) -> `Writer`(field) -> `Except`(field) -> `Identity`.
fn run(prog: App<i64>, env: Env, s0: i64) -> Result<((i64, i64), Log), EvalError> {
    let state_layer = (prog.run_reader_t)(env);
    let writer_layer = (state_layer.run_state_t)(s0);
    let except_layer = writer_layer.run_writer_t;
    let Identity(result) = except_layer.run_except_t;
    result
}

fn main() {
    println!("=== Four-deep stack: Reader+State+Writer+Except interpreter ===\n");

    // ── Success run ───────────────────────────────────────────────────────────
    // env = { x = 10, y = 4 };  expr = x + (20 / y) = 10 + 5 = 15
    // Post-order trace: leaves left-to-right, then inner nodes bottom-up.
    let env = Env::new(&[("x", 10), ("y", 4)]);
    let expr = Expr::Add(
        Box::new(Expr::Var("x".to_string())),
        Box::new(Expr::Div(
            Box::new(Expr::Lit(20)),
            Box::new(Expr::Var("y".to_string())),
        )),
    );
    let result = run(eval(&expr), env.clone(), 0);
    let expected_trace = vec![
        "step 1: Var(x)".to_string(),
        "step 2: Lit(20)".to_string(),
        "step 3: Var(y)".to_string(),
        "step 4: Div".to_string(),
        "step 5: Add".to_string(),
    ];
    assert_eq!(
        result,
        Ok(((15, 5), expected_trace.clone())),
        "success: value 15, 5 steps, full post-order trace"
    );
    println!("Success: x + (20 / y) with x=10, y=4");
    println!("  result = {result:?}");
    let ((value, steps), trace) = result.unwrap();
    println!("  value      = {value}");
    println!("  step count = {steps}");
    println!("  trace:");
    for entry in &trace {
        println!("    {entry}");
    }
    println!();

    // ── Error run 1: unbound variable ─────────────────────────────────────────
    // env = { x = 10 };  expr = x + z  (z is unbound)  => Err(UnboundVariable("z"))
    let env_unbound = Env::new(&[("x", 10)]);
    let expr_unbound = Expr::Add(
        Box::new(Expr::Var("x".to_string())),
        Box::new(Expr::Var("z".to_string())),
    );
    let err_result = run(eval(&expr_unbound), env_unbound, 0);
    assert_eq!(
        err_result,
        Err(EvalError::UnboundVariable("z".to_string())),
        "unbound var: Err with NO ((value,state),log) — state+trace discarded"
    );
    assert!(
        err_result.is_err(),
        "result is a bare Err: the partial 2 steps and partial trace are gone"
    );
    println!("Error 1: x + z with x=10 (z unbound)");
    println!("  result = {err_result:?}");
    println!("  (state counter and trace are DISCARDED — Except is innermost)\n");

    // ── Error run 2: division by zero ─────────────────────────────────────────
    // expr = 1 / 0  => Err(DivByZero)
    let env_div = Env::new(&[]);
    let expr_div = Expr::Div(Box::new(Expr::Lit(1)), Box::new(Expr::Lit(0)));
    let div_result = run(eval(&expr_div), env_div, 0);
    assert_eq!(
        div_result,
        Err(EvalError::DivByZero),
        "div-by-zero: Err with NO ((value,state),log) — state+trace discarded"
    );
    println!("Error 2: 1 / 0");
    println!("  result = {div_result:?}");
    println!("  (the 3 visited nodes' steps and trace are DISCARDED)\n");

    println!("=== All assertions passed! ===");
    println!();
    println!("Key insight: layer ORDER decides error semantics. With Except");
    println!("innermost, a throw collapses the whole stack to Err(e) — the");
    println!("State counter and Writer trace live in the Ok branch and vanish.");
    println!("Swapping Except above Writer/State would instead preserve them.");
}
