//! `mdo!` bare-`pure` auto-lift tests — **RED phase**.
//!
//! These tests verify the *not-yet-implemented* feature where a bare `pure(expr)`
//! call inside an `mdo!` block is automatically rewritten by the macro to
//! `<Marker as ::monadify::Applicative>::pure(expr)`.
//!
//! ## RED rationale
//!
//! The current `mdo!` macro passes each monadic-position expression through
//! verbatim — it has no special handling for the identifier `pure`.  A bare
//! `pure(…)` call therefore references a nonexistent free function and fails:
//!
//! ```text
//! error[E0425]: cannot find function `pure` in this scope
//! ```
//!
//! Every `#[test]` function that uses bare `pure(…)` will produce this compile
//! error until the macro's token-walk rewriter is implemented.  The
//! backward-compatibility tests use only `Marker::pure(…)`, which already
//! compiles in isolation, but they reside in the same file, so the entire file
//! fails to compile in the RED phase.
//!
//! ## Covered monadic-expression positions
//!
//! For each instance the tests exercise bare `pure(…)` in all four positions
//! mandated by the feature specification:
//!
//! 1. **Final line** — `pure(expr)` as the trailing, non-`;` expression.
//! 2. **Bind RHS** — `x <- pure(expr)`.
//! 3. **Bare sequencing** — `pure(())` as a semicolon-terminated statement with
//!    no bind variable (the value is discarded via the `move |_|` continuation).
//! 4. **Nested** — `pure(expr)` as a sub-expression inside another call, e.g.
//!    `x <- std::convert::identity(pure(3))`.  The macro must recursively walk
//!    the token stream of the bind RHS to reach it.
//!
//! Plus backward-compatibility (qualified `Marker::pure(…)` unchanged) and,
//! for `OptionKind` and `VecKind`, guard interaction.
//!
//! ## Instances covered
//!
//! - `OptionKind` (7 tests + guard interaction × 2)
//! - `ResultKind<String>` (5 tests)
//! - `VecKind` (5 tests + guard interaction × 2)
//! - `IdentityKind` (5 tests)
//! - `ReaderTKind<Config, IdentityKind>` (5 tests)

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::reader::{Reader, ReaderT, ReaderTKind};

// ── ReaderT test infrastructure ───────────────────────────────────────────────

/// Minimal read-only configuration environment for all `ReaderT` tests in this file.
#[derive(Clone, Debug, PartialEq)]
struct Config {
    base: i32,
    factor: i32,
}

/// Kind marker alias for `ReaderTKind<Config, IdentityKind>`.
type ReaderKind = ReaderTKind<Config, IdentityKind>;

/// `ReaderT<Config, IdentityKind, A>` — a computation `Config -> Identity<A>`.
type ConfigReader<A> = Reader<Config, A>;

/// Run a `ConfigReader<i32>` against `cfg` and unwrap the `Identity`.
fn run_i32(computation: &ConfigReader<i32>, cfg: Config) -> i32 {
    let Identity(value) = (computation.run_reader_t)(cfg);
    value
}

// ═════════════════════════════════════════════════════════════════════════════
// OptionKind
// ═════════════════════════════════════════════════════════════════════════════
//
// RED: all tests below that use bare `pure(…)` fail with E0425 today.

// ── 1. Final-position bare pure ───────────────────────────────────────────────

/// Bare `pure(x + y)` in the final line must resolve to `OptionKind::pure(x + y)`
/// and yield the same result as the hand-written nested bind with qualified pure.
///
/// RED: `pure(x + y)` → E0425 until the token-walk rewriter is implemented.
#[test]
fn option_final_bare_pure_eq_qualified() {
    // LHS: bare pure in final position — the new feature under test.
    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- Some(2i32);
        y <- Some(3i32);
        pure(x + y)
    };
    // RHS: hand-written nested bind with fully-qualified pure.
    let rhs: Option<i32> = OptionKind::bind(Some(2i32), move |x| {
        OptionKind::bind(Some(3i32), move |y| OptionKind::pure(x + y))
    });
    assert_eq!(lhs, Some(5));
    assert_eq!(lhs, rhs);
}

// ── 2. Bind-position + bare sequencing ───────────────────────────────────────

/// `x <- pure(2)` (bind RHS) and bare sequencing `pure(())` must each resolve
/// to the `OptionKind` applicative, leaving the final `Some(5)` intact.
///
/// RED: `pure(2i32)` and `pure(())` → E0425.
#[test]
fn option_bind_position_bare_pure_and_sequencing() {
    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- pure(2i32);   // bind position: OptionKind::pure(2) = Some(2)
        pure(());            // sequencing:   OptionKind::pure(()) = Some(()), discarded
        pure(x + 3)         // final
    };
    // Equivalent hand-written form using qualified pure throughout.
    let rhs: Option<i32> = OptionKind::bind(OptionKind::pure(2i32), move |x| {
        OptionKind::bind(OptionKind::pure(()), move |_| OptionKind::pure(x + 3))
    });
    assert_eq!(lhs, Some(5));
    assert_eq!(lhs, rhs);
}

// ── 3. Mixed two-depth bare pure ──────────────────────────────────────────────

/// Bare `pure` in BOTH a bind position AND the final position within the same
/// block — exercises per-position rewriting at two nesting levels.
///
/// `x: i32` is `Copy`, so it crosses the generated `move` closures freely.
///
/// RED: both bare `pure` occurrences → E0425.
#[test]
fn option_mixed_two_depth_bare_pure() {
    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- pure(10i32);  // bind: OptionKind::pure(10) = Some(10), x = 10
        y <- Some(5i32);
        pure(x - y)        // final: OptionKind::pure(5) = Some(5)
    };
    assert_eq!(lhs, Some(5));
}

// ── 4. Nested bare pure inside a function call ────────────────────────────────

/// `pure(4)` is nested as an argument inside `std::convert::identity(…)` in the
/// bind RHS.  The macro token-walk must recurse into the argument list to rewrite
/// the inner `pure`.
///
/// Once implemented: `identity(pure(4))` → `identity(OptionKind::pure(4))`
///                   = `identity(Some(4))` = `Some(4)`.
///
/// RED: `pure(4i32)` → E0425.
#[test]
fn option_nested_bare_pure_in_bind_rhs() {
    let lhs: Option<i32> = mdo! {
        OptionKind;
        x <- std::convert::identity(pure(4i32));
        pure(x + 1)
    };
    assert_eq!(lhs, Some(5));
}

// ── 5. Backward-compat: fully-qualified `Marker::pure` unchanged ──────────────

/// Fully-qualified `OptionKind::pure(…)` in BOTH bind and final positions must
/// compile and produce correct results unchanged after the feature is added.
///
/// The rewriter must not touch `::``-prefixed path calls.
///
/// NOTE: this test would compile fine in isolation today. It fails to compile in
/// the RED phase only because other tests in this file trigger a file-level error.
#[test]
fn option_backward_compat_qualified_pure() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- OptionKind::pure(2i32);
        y <- OptionKind::pure(3i32);
        OptionKind::pure(x + y)
    };
    assert_eq!(result, Some(5));
}

// ── 6–7. Guard interaction ────────────────────────────────────────────────────

/// `guard(x > 0)` passes when `x = 7`; the bare `pure(x * 2)` final is reached.
///
/// RED: `pure(x * 2)` → E0425.
#[test]
fn option_guard_true_with_bare_pure_final() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(7i32);
        guard(x > 0);
        pure(x * 2)
    };
    assert_eq!(result, Some(14));
}

/// `guard(x < 0)` fails when `x = 7` → `None`; the bare `pure(x * 2)` final
/// is never reached (short-circuit semantics).
///
/// RED: `pure(x * 2)` → E0425.
#[test]
fn option_guard_false_with_bare_pure_final() {
    let result: Option<i32> = mdo! {
        OptionKind;
        x <- Some(7i32);
        guard(x < 0);
        pure(x * 2)
    };
    assert_eq!(result, None);
}

// ── 8. Struct-field colon does NOT suppress bare-pure rewrite ────────────────

/// A bare `pure(7)` used as the value of a struct-literal field (e.g.
/// `Wrap { v: pure(7) }`) must be rewritten to `OptionKind::pure(7)`.
///
/// The lone `:` that separates the field name from its value is NOT the `::` path
/// separator — the two-token look-behind guard must distinguish them.  Before the
/// fix a single-`:` check caused `pure(7)` here to be silently left alone,
/// producing E0425 ("cannot find function `pure`").
#[test]
fn option_struct_field_colon_does_not_suppress_bare_pure() {
    struct Wrap {
        v: Option<i32>,
    }

    let r: Option<i32> = mdo! {
        OptionKind;
        w <- {
            let p = Wrap { v: pure(7i32) };
            p.v
        };
        pure(w)
    };
    assert_eq!(r, Some(7));
}

// ═════════════════════════════════════════════════════════════════════════════
// ResultKind<String>
// ═════════════════════════════════════════════════════════════════════════════
//
// `guard` is intentionally absent: `ResultKind` has no lawful zero element.
// RED: all bare `pure(…)` occurrences fail with E0425 today.

// ── 1. Final-position bare pure ───────────────────────────────────────────────

/// Bare `pure(x + y)` in the final line must resolve to
/// `ResultKind::<String>::pure(x + y)` and agree with the hand-written form.
///
/// RED: `pure(x + y)` → E0425.
#[test]
fn result_final_bare_pure_eq_qualified() {
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- Ok(2i32);
        y <- Ok(3i32);
        pure(x + y)
    };
    let rhs: Result<i32, String> = ResultKind::<String>::bind(Ok(2i32), move |x| {
        ResultKind::<String>::bind(Ok(3i32), move |y| ResultKind::<String>::pure(x + y))
    });
    assert_eq!(lhs, Ok(5));
    assert_eq!(lhs, rhs);
}

// ── 2. Bind-position + bare sequencing ───────────────────────────────────────

/// `x <- pure(2)` (bind RHS) and bare sequencing `pure(())` with
/// `ResultKind::<String>` as the block marker.
///
/// RED: `pure(2i32)` and `pure(())` → E0425.
#[test]
fn result_bind_position_bare_pure_and_sequencing() {
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- pure(2i32);  // ResultKind::<String>::pure(2) = Ok(2)
        pure(());          // Ok(()), discarded
        pure(x + 3)       // final
    };
    let rhs: Result<i32, String> =
        ResultKind::<String>::bind(ResultKind::<String>::pure(2i32), move |x| {
            ResultKind::<String>::bind(ResultKind::<String>::pure(()), move |_| {
                ResultKind::<String>::pure(x + 3)
            })
        });
    assert_eq!(lhs, Ok(5));
    assert_eq!(lhs, rhs);
}

// ── 3. Mixed two-depth bare pure ──────────────────────────────────────────────

/// Bare `pure` in BOTH bind and final positions in a `ResultKind` block.
///
/// RED: both bare `pure` occurrences → E0425.
#[test]
fn result_mixed_two_depth_bare_pure() {
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- pure(10i32);  // bind: Ok(10)
        y <- Ok(3i32);
        pure(x + y)        // final: Ok(13)
    };
    assert_eq!(lhs, Ok(13));
}

// ── 4. Nested bare pure inside a function call ────────────────────────────────

/// `pure(5)` nested inside `std::convert::identity(…)` in the bind RHS.
///
/// Once implemented: `identity(pure(5))` → `identity(Ok(5))` = `Ok(5)`.
///
/// RED: `pure(5i32)` → E0425.
#[test]
fn result_nested_bare_pure_in_bind_rhs() {
    let lhs: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- std::convert::identity(pure(5i32));
        pure(x + 1)
    };
    assert_eq!(lhs, Ok(6));
}

// ── 5. Backward-compat: fully-qualified `Marker::pure` unchanged ──────────────

/// Fully-qualified `ResultKind::<String>::pure(…)` in both bind and final
/// positions must be left verbatim and continue to produce `Ok(10)`.
#[test]
fn result_backward_compat_qualified_pure() {
    let result: Result<i32, String> = mdo! {
        ResultKind::<String>;
        x <- ResultKind::<String>::pure(4i32);
        y <- ResultKind::<String>::pure(6i32);
        ResultKind::<String>::pure(x + y)
    };
    assert_eq!(result, Ok(10));
}

// ═════════════════════════════════════════════════════════════════════════════
// VecKind
// ═════════════════════════════════════════════════════════════════════════════
//
// RED: all bare `pure(…)` occurrences fail with E0425 today.

// ── 1. Final-position bare pure ───────────────────────────────────────────────

/// Bare `pure(x + y)` in the final line of a `VecKind` block.
/// `VecKind::bind` is `flat_map`; the result is the Cartesian product.
///
/// RED: `pure(x + y)` → E0425.
#[test]
fn vec_final_bare_pure_eq_qualified() {
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2];
        y <- vec![10i32, 20];
        pure(x + y)
    };
    let rhs: Vec<i32> = VecKind::bind(vec![1i32, 2], move |x| {
        VecKind::bind(vec![10i32, 20], move |y| VecKind::pure(x + y))
    });
    // flat_map order: x=1→[11,21], x=2→[12,22]
    assert_eq!(lhs, vec![11, 21, 12, 22]);
    assert_eq!(lhs, rhs);
}

// ── 2. Bind-position + bare sequencing ───────────────────────────────────────

/// `x <- pure(3)` (bind RHS) and bare sequencing `pure(())` with `VecKind`.
/// `VecKind::pure(3)` = `vec![3]` (singleton); the flat_map yields one element.
///
/// RED: `pure(3i32)` and `pure(())` → E0425.
#[test]
fn vec_bind_position_bare_pure_and_sequencing() {
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- pure(3i32);   // VecKind::pure(3) = vec![3]
        pure(());           // VecKind::pure(()) = vec![()], sequencing
        pure(x + 2)        // final: vec![5]
    };
    let rhs: Vec<i32> = VecKind::bind(VecKind::pure(3i32), move |x| {
        VecKind::bind(VecKind::pure(()), move |_| VecKind::pure(x + 2))
    });
    assert_eq!(lhs, vec![5]);
    assert_eq!(lhs, rhs);
}

// ── 3. Mixed two-depth bare pure ──────────────────────────────────────────────

/// Bare `pure` in BOTH bind and final positions in a `VecKind` block.
/// `pure(2)` in bind position yields `vec![2]`; the Cartesian product with
/// `vec![1, 2]` then produces `vec![3, 4]`.
///
/// RED: both bare `pure` occurrences → E0425.
#[test]
fn vec_mixed_two_depth_bare_pure() {
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- pure(2i32);        // bind: vec![2], x = 2
        y <- vec![1i32, 2i32];  // Cartesian with x = 2
        pure(x + y)             // final: vec![3, 4]
    };
    assert_eq!(lhs, vec![3, 4]);
}

// ── 4. Nested bare pure inside a function call ────────────────────────────────

/// `pure(3)` nested inside `std::convert::identity(…)` in the bind RHS.
///
/// Once implemented: `identity(VecKind::pure(3))` = `identity(vec![3])` = `vec![3]`.
///
/// RED: `pure(3i32)` → E0425.
#[test]
fn vec_nested_bare_pure_in_bind_rhs() {
    let lhs: Vec<i32> = mdo! {
        VecKind;
        x <- std::convert::identity(pure(3i32));
        pure(x * 10)
    };
    assert_eq!(lhs, vec![30]);
}

// ── 5. Backward-compat: fully-qualified `Marker::pure` unchanged ──────────────

/// Fully-qualified `VecKind::pure(…)` in both bind and final positions must be
/// preserved verbatim, yielding the same singleton Cartesian product.
#[test]
fn vec_backward_compat_qualified_pure() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- VecKind::pure(2i32);
        y <- VecKind::pure(3i32);
        VecKind::pure(x + y)
    };
    assert_eq!(result, vec![5]);
}

// ── 6–7. Guard interaction ────────────────────────────────────────────────────

/// `guard(x % 2 == 0)` retains only even elements; bare `pure(x)` final.
///
/// RED: `pure(x)` → E0425.
#[test]
fn vec_guard_true_with_bare_pure_final() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3, 4];
        guard(x % 2 == 0);
        pure(x)
    };
    assert_eq!(result, vec![2, 4]);
}

/// `guard(false)` eliminates every element; bare `pure(x)` final is never reached.
///
/// RED: `pure(x)` → E0425.
#[test]
fn vec_guard_false_with_bare_pure_final() {
    let result: Vec<i32> = mdo! {
        VecKind;
        x <- vec![1i32, 2, 3];
        guard(false);
        pure(x)
    };
    assert_eq!(result, vec![]);
}

// ═════════════════════════════════════════════════════════════════════════════
// IdentityKind
// ═════════════════════════════════════════════════════════════════════════════
//
// `guard` is absent: `Identity` has no lawful zero / short-circuit semantics.
// RED: all bare `pure(…)` occurrences fail with E0425 today.

// ── 1. Final-position bare pure ───────────────────────────────────────────────

/// Bare `pure(x + y)` in the final line of an `IdentityKind` block.
///
/// RED: `pure(x + y)` → E0425.
#[test]
fn identity_final_bare_pure_eq_qualified() {
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- Identity(2i32);
        y <- Identity(3i32);
        pure(x + y)
    };
    let rhs: Identity<i32> = IdentityKind::bind(Identity(2i32), move |x| {
        IdentityKind::bind(Identity(3i32), move |y| IdentityKind::pure(x + y))
    });
    assert_eq!(lhs, Identity(5));
    assert_eq!(lhs, rhs);
}

// ── 2. Bind-position + bare sequencing ───────────────────────────────────────

/// `x <- pure(2)` (bind) and bare sequencing `pure(())` with `IdentityKind`.
///
/// RED: `pure(2i32)` and `pure(())` → E0425.
#[test]
fn identity_bind_position_bare_pure_and_sequencing() {
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- pure(2i32);  // IdentityKind::pure(2) = Identity(2)
        pure(());          // Identity(()), discarded
        pure(x + 8)       // final: Identity(10)
    };
    let rhs: Identity<i32> = IdentityKind::bind(IdentityKind::pure(2i32), move |x| {
        IdentityKind::bind(IdentityKind::pure(()), move |_| IdentityKind::pure(x + 8))
    });
    assert_eq!(lhs, Identity(10));
    assert_eq!(lhs, rhs);
}

// ── 3. Mixed two-depth bare pure ──────────────────────────────────────────────

/// Bare `pure` in BOTH bind and final positions within the same `IdentityKind` block.
/// `x: i32` is `Copy`, so crossing the generated `move` closures is safe.
///
/// RED: both bare `pure` occurrences → E0425.
#[test]
fn identity_mixed_two_depth_bare_pure() {
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- pure(7i32);  // bind: Identity(7)
        pure(x * 3)       // final: Identity(21)
    };
    assert_eq!(lhs, Identity(21));
}

// ── 4. Nested bare pure inside a function call ────────────────────────────────

/// `pure(7)` nested inside `std::convert::identity(…)` in the bind RHS.
///
/// Once implemented: `identity(IdentityKind::pure(7))` = `identity(Identity(7))`
///                   = `Identity(7)`.
///
/// RED: `pure(7i32)` → E0425.
#[test]
fn identity_nested_bare_pure_in_bind_rhs() {
    let lhs: Identity<i32> = mdo! {
        IdentityKind;
        x <- std::convert::identity(pure(7i32));
        pure(x + 3)
    };
    assert_eq!(lhs, Identity(10));
}

// ── 5. Backward-compat: fully-qualified `Marker::pure` unchanged ──────────────

/// Fully-qualified `IdentityKind::pure(…)` in both bind and final positions must
/// be preserved verbatim after the feature is added.
#[test]
fn identity_backward_compat_qualified_pure() {
    let result: Identity<i32> = mdo! {
        IdentityKind;
        x <- IdentityKind::pure(5i32);
        y <- IdentityKind::pure(5i32);
        IdentityKind::pure(x + y)
    };
    assert_eq!(result, Identity(10));
}

// ═════════════════════════════════════════════════════════════════════════════
// ReaderTKind<Config, IdentityKind>
// ═════════════════════════════════════════════════════════════════════════════
//
// `guard` is absent: `ReaderT` has no lawful zero.
//
// Non-Copy capture constraint: `ReaderT` is `Clone` but not `Copy`; values bound
// from `ReaderT` steps are typically `i32` (Copy) and cross `move` closures
// freely.  The constraint applies to *external* non-Copy values — see the macro
// docs.  All tests below keep bound values as `i32` to stay within this limit.
//
// RED: all bare `pure(…)` occurrences fail with E0425 today.

// ── 1. Final-position bare pure ───────────────────────────────────────────────

/// Bare `pure(b + f)` in the final line of a `ReaderKind` block.
/// Both `b` and `f` are `i32` (Copy); they thread freely through the closures.
///
/// Equivalence is verified by running both lhs and rhs against the same concrete
/// environments (ReaderT values are functions, not comparable directly).
///
/// RED: `pure(b + f)` → E0425.
#[test]
fn reader_final_bare_pure_eq_qualified() {
    let lhs: ConfigReader<i32> = mdo! {
        ReaderKind;
        b <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        f <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        pure(b + f)   // bare pure — must become ReaderKind::pure(b + f)
    };
    let rhs: ConfigReader<i32> =
        ReaderKind::bind(ReaderT::new(|cfg: Config| Identity(cfg.base)), move |b| {
            ReaderKind::bind(ReaderT::new(|cfg: Config| Identity(cfg.factor)), move |f| {
                ReaderKind::pure(b + f)
            })
        });
    // Compare by running both against the same environments.
    assert_eq!(
        run_i32(&lhs, Config { base: 4, factor: 6 }),
        run_i32(&rhs, Config { base: 4, factor: 6 }),
    );
    assert_eq!(
        run_i32(
            &lhs,
            Config {
                base: -3,
                factor: 10
            }
        ),
        run_i32(
            &rhs,
            Config {
                base: -3,
                factor: 10
            }
        ),
    );
    assert_eq!(run_i32(&lhs, Config { base: 4, factor: 6 }), 10);
}

// ── 2. Bind-position + bare sequencing ───────────────────────────────────────

/// `x <- pure(10)` (bind) and bare sequencing `pure(())` with `ReaderKind`.
/// All computations are pure (environment-independent), so any `Config` yields 15.
///
/// RED: `pure(10i32)` and `pure(())` → E0425.
#[test]
fn reader_bind_position_bare_pure_and_sequencing() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        x <- pure(10i32);  // ReaderKind::pure(10) — const reader returning 10
        pure(());           // ReaderKind::pure(()) — sequencing, value discarded
        pure(x + 5)        // final: ReaderKind::pure(15)
    };
    // The environment is irrelevant since all steps are pure.
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 15);
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 99,
                factor: -7
            }
        ),
        15
    );
}

// ── 3. Mixed two-depth bare pure ──────────────────────────────────────────────

/// Bare `pure` in BOTH bind and final positions within the same `ReaderKind` block.
/// `b: i32` (Copy) and `x: i32` (Copy) cross closure boundaries freely.
///
/// RED: both bare `pure` occurrences → E0425.
#[test]
fn reader_mixed_two_depth_bare_pure() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        b <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        x <- pure(10i32);   // bind: const reader, x = 10 (Copy)
        pure(b + x)         // final: ReaderKind::pure(b + 10)
    };
    // b = 5, x = 10 → 15
    assert_eq!(run_i32(&computation, Config { base: 5, factor: 0 }), 15);
    // b = 0, x = 10 → 10
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 0,
                factor: 99
            }
        ),
        10
    );
}

// ── 4. Nested bare pure inside a function call ────────────────────────────────

/// `pure(42)` nested inside `std::convert::identity(…)` in the bind RHS.
/// `ReaderKind::pure(42)` is a `ConfigReader<i32>` — Clone via Rc, so the
/// `.clone()` the macro emits is valid.
///
/// Once implemented: `identity(ReaderKind::pure(42))` = `ReaderKind::pure(42)`.
///
/// RED: `pure(42i32)` → E0425.
#[test]
fn reader_nested_bare_pure_in_bind_rhs() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        x <- std::convert::identity(pure(42i32));
        pure(x + 1)
    };
    // x = 42 (from pure) → final = 43; environment plays no role.
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 43);
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 99,
                factor: -1
            }
        ),
        43
    );
}

// ── 5. Backward-compat: fully-qualified `Marker::pure` unchanged ──────────────

/// Fully-qualified `ReaderKind::pure(…)` in both bind and final positions must
/// be preserved verbatim (the `::` path-join guard prevents rewriting).
#[test]
fn reader_backward_compat_qualified_pure() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        x <- ReaderKind::pure(10i32);
        y <- ReaderKind::pure(5i32);
        ReaderKind::pure(x + y)
    };
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 15);
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 99,
                factor: -1
            }
        ),
        15
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Coverage-gap regression tests (bare-pure feature completeness)
// ═════════════════════════════════════════════════════════════════════════════

// ── R1. `..pure(` is NOT suppressed (two-dot look-behind) ────────────────────

/// Proves that the `..` double-dot prefix does NOT trigger the method-call
/// suppression guard, so `..pure(x)` inside a bind-RHS block is rewritten.
///
/// The suppression guard fires only when a SINGLE `.` immediately precedes
/// `pure` AND the token before THAT is not also `.`.  With `..`, both the
/// preceding token and the one before it are `.`, so:
///
/// ```text
/// prev_is_dot = true   (second `.`)
/// prev2_is_dot = true  (first  `.`)
/// is_method = prev_is_dot && !prev2_is_dot = true && false = false
/// ```
///
/// → NOT suppressed → `pure(x)` is rewritten to `OptionKind::pure(x)` = `Some(x)`.
///
/// Verification: the rewritten expression `..pure(x)` becomes
/// `..OptionKind::pure(x)` = `..Some(x)`, a `RangeTo<Option<i32>>`.
/// Reading its `.end` field yields `Some(x)`.  The assertion `rto.end == Some(42)`
/// holds only if `pure` resolved via the marker (not a free function, which would
/// not compile, and not an identity that drops the `Some` wrapper).
#[test]
fn option_struct_update_does_not_suppress_bare_pure() {
    let r: Option<bool> = mdo! {
        OptionKind;
        x <- Some(42i32);
        b <- {
            // `..pure(x)` in token stream — double-dot precedes `pure`.
            // After rewrite: `..OptionKind::pure(x)` = `..Some(42)` = RangeTo<Option<i32>>.
            // rto.end == Some(42) proves pure was dispatched via the marker.
            let rto = ..pure(x);
            Some(rto.end == Some(42i32))
        };
        pure(b)
    };
    assert_eq!(r, Some(true));
}

// ── R2. Monad left/right identity through bare `pure` ────────────────────────

/// Exercises the monad identity laws expressed entirely through the NEW bare
/// `pure(…)` syntax.  The existing `laws.rs` property tests use only the
/// qualified `OptionKind::pure`; this test closes that gap for the bare form.
///
/// **Left identity** (`mdo!{M; x <- pure(a); k(x)} == k(a)`):
///   `mdo!{OptionKind; x <- pure(5i32); pure(f(x))}` must equal `Some(f(5))`
///   and agree with the hand-written `OptionKind::bind(OptionKind::pure(5), …)`.
///
/// **Right identity** (`mdo!{M; x <- m; pure(x)} == m`):
///   `mdo!{OptionKind; x <- Some(7i32); pure(x)}` must equal `Some(7)`.
#[test]
fn option_law_left_right_identity_with_bare_pure() {
    let f = |x: i32| x + 1;

    // Left identity via bare pure.
    let lhs_left: Option<i32> = mdo! {
        OptionKind;
        x <- pure(5i32);   // bare pure → OptionKind::pure(5) = Some(5)
        pure(f(x))         // bare pure → OptionKind::pure(f(5)) = Some(6)
    };
    // Hand-written equivalent using qualified pure for cross-check.
    let rhs_left: Option<i32> =
        OptionKind::bind(OptionKind::pure(5i32), move |x| OptionKind::pure(f(x)));
    assert_eq!(lhs_left, Some(6));
    assert_eq!(lhs_left, rhs_left);

    // Right identity via bare pure.
    let lhs_right: Option<i32> = mdo! {
        OptionKind;
        x <- Some(7i32);
        pure(x)  // bare pure → OptionKind::pure(7) = Some(7)
    };
    assert_eq!(lhs_right, Some(7));
}

// ── R3. Bare `pure` inside `guard(…)` condition ───────────────────────────────

/// The `guard(cond)` statement forwards `cond` through `rewrite_pure` before
/// evaluating it.  This test verifies that a bare `pure(x)` inside the
/// condition expression is rewritten to `OptionKind::pure(x)` = `Some(x)`,
/// making the condition a `bool` comparison between two `Option<i32>` values.
///
/// - True case: `x = 7`, `guard(pure(7) == Some(7))` → `guard(true)` → `Some(())`
///   → computation continues → `pure(x)` = `Some(7)`.
/// - False case: `x = 42`, `guard(pure(42) == Some(7))` → `guard(false)` → `None`
///   → short-circuit → result is `None`.
#[test]
fn option_bare_pure_inside_guard_cond() {
    // True branch: guard passes, result reaches the final pure.
    let r_true: Option<i32> = mdo! {
        OptionKind;
        x <- Some(7i32);
        guard(pure(x) == Some(7i32));  // pure(x) → OptionKind::pure(x) = Some(x); Some(7)==Some(7)
        pure(x)
    };
    assert_eq!(r_true, Some(7));

    // False branch: guard short-circuits to None.
    let r_false: Option<i32> = mdo! {
        OptionKind;
        x <- Some(42i32);
        guard(pure(x) == Some(7i32));  // Some(42) == Some(7) = false → guard returns None
        pure(x)
    };
    assert_eq!(r_false, None);
}

// ── R4. Method call `.pure(…)` is left verbatim (suppressed) ─────────────────

/// Proves the suppression branch: `obj.pure(x)` — where `pure` is an ordinary
/// method, not a bare call — is left untouched by the rewriter.
///
/// Look-behind at the point where `pure` is encountered:
/// ```text
/// prev_is_dot  = true   (the single `.` method-call dot)
/// prev2_is_dot = false  (the token before that is the `obj` ident, not `.`)
/// is_method    = prev_is_dot && !prev2_is_dot = true && true = true  → SUPPRESS
/// ```
///
/// The local `HasPure::pure` method multiplies its argument by 100.  If the
/// macro had rewritten `obj.pure(3)` the expression would be a compile error
/// (UFCS applied via method dispatch is invalid syntax).  The runtime assertion
/// `Some(300)` distinguishes method dispatch (`3 * 100`) from any other path.
#[test]
fn option_method_pure_is_not_rewritten() {
    struct HasPure;
    impl HasPure {
        fn pure(&self, x: i32) -> i32 {
            x * 100
        }
    }

    let r: Option<i32> = mdo! {
        OptionKind;
        m <- {
            // `obj.pure(3)` — single `.` before `pure`, token before that is `obj` (not `.`)
            // → is_method = true → suppressed → dispatches to HasPure::pure → 3 * 100 = 300
            let obj = HasPure;
            Some(obj.pure(3))
        };
        pure(m)  // bare pure → OptionKind::pure(300) = Some(300)
    };
    assert_eq!(r, Some(300));
}
