//! A small expression-language interpreter built on a **four-deep monad
//! transformer stack** — every layer is genuinely load-bearing:
//!
//! - **Reader** (`Env`)   — the variable environment (`name -> value`).
//! - **State** (`i64`)    — an evaluation-step counter, bumped at each node.
//! - **Writer** (`Vec<String>`) — a pre-order trace of evaluated subexpressions.
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
//! Run: `cargo run --quiet --example interpreter_full_stack`

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::Bind;
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
/// keys, so `eval` uses explicit `AppKind::bind` rather than `mdo!`.
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

/// Bump the step counter and record one trace entry for the node being entered.
/// Combines the `State` and `Writer` effects; called pre-order at every node.
fn visit(label: String) -> App<()> {
    AppKind::bind(get_step(), move |n| {
        let label = label.clone();
        AppKind::bind(put_step(n + 1), move |_| {
            tell_log(format!("step {}: {}", n + 1, label))
        })
    })
}

// ── The interpreter ─────────────────────────────────────────────────────────────

/// Evaluate `expr` in the full stack. Each node: bumps the counter (State),
/// records a trace entry (Writer), looks vars up in the env (Reader), and throws
/// on unbound var / div-by-zero (Except). Uses explicit `AppKind::bind` because
/// `Env` is not `Copy`.
fn eval(expr: &Expr) -> App<i64> {
    match expr {
        Expr::Lit(n) => {
            let n = *n;
            AppKind::bind(visit(format!("Lit({n})")), move |_| AppKind::pure(n))
        }
        Expr::Var(name) => {
            let name = name.clone();
            AppKind::bind(visit(format!("Var({name})")), move |_| {
                let name = name.clone();
                AppKind::bind(ask_env(), move |env: Env| match env.lookup(&name) {
                    Some(v) => AppKind::pure(v),
                    None => throw_err(EvalError::UnboundVariable(name.clone())),
                })
            })
        }
        Expr::Add(l, r) => {
            let l = (**l).clone();
            let r = (**r).clone();
            AppKind::bind(visit("Add".to_string()), move |_| {
                let l = l.clone();
                let r = r.clone();
                AppKind::bind(eval(&l), move |lv| {
                    let r = r.clone();
                    AppKind::bind(eval(&r), move |rv| AppKind::pure(lv + rv))
                })
            })
        }
        Expr::Div(l, r) => {
            let l = (**l).clone();
            let r = (**r).clone();
            AppKind::bind(visit("Div".to_string()), move |_| {
                let l = l.clone();
                let r = r.clone();
                AppKind::bind(eval(&l), move |lv| {
                    let r = r.clone();
                    AppKind::bind(eval(&r), move |rv| {
                        if rv == 0 {
                            throw_err(EvalError::DivByZero)
                        } else {
                            AppKind::pure(lv / rv)
                        }
                    })
                })
            })
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
        "step 1: Add".to_string(),
        "step 2: Var(x)".to_string(),
        "step 3: Div".to_string(),
        "step 4: Lit(20)".to_string(),
        "step 5: Var(y)".to_string(),
    ];
    assert_eq!(
        result,
        Ok(((15, 5), expected_trace.clone())),
        "success: value 15, 5 steps, full pre-order trace"
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
