//! `mdo!` do-block tests for `ReaderTKind` (Reader monad: threading a read-only environment).
//!
//! Tests in this file are gated by the parent `do_notation` module's
//! `#![cfg(feature = "do-notation")]` attribute, so they are invisible to the
//! default build.
//!
//! ## What we test
//!
//! The "real power" of Reader is that a single environment value is silently
//! threaded through every step without being passed explicitly.  These tests
//! use `Config` (a two-field struct) as the environment and
//! `ReaderT<Config, IdentityKind, A>` (alias: `Reader<Config, A>`) as the monad.
//!
//! Using the `IdentityKind` inner monad keeps each `ReaderT` run value as
//! `Identity<A>` — no optionality or error context — so the tests focus
//! purely on environment threading rather than short-circuiting.
//!
//! ## Covered scenarios
//!
//! 1. **Env threading** — multiple `ReaderT::new(...)` computations in a do-block
//!    each receive the SAME `Config` from the environment.
//! 2. **`ask`** — `MonadReader::ask()` lifts the entire environment into the block;
//!    a `let` binding can derive values from it without creating another `ReaderT`.
//! 3. **`ask` + field-reading combined** — mixes `ask` with explicit `ReaderT::new`
//!    steps in the same do-block.
//! 4. **Equivalence** — the `mdo!` version, when run against several concrete `Config`
//!    values, produces identical results to the hand-written nested `bind` chain.
//!    (Closures cannot be compared directly; we compare by *running* both against
//!    the same environments — per the spike's note on arbitrary-closure equality.)
//! 5. **Three-binding chain** — depth > 2, confirming the desugaring is correct
//!    at greater nesting levels.
//! 6. **`local`** — a sub-computation wrapped in `local(f, comp)` sees a modified
//!    environment; earlier steps in the same do-block see the original environment.
//!
//! ## Notes
//!
//! - `guard` is intentionally absent: ReaderT has no lawful zero, so using
//!   `guard` inside a ReaderT do-block is a deliberate compile error.
//! - `ReaderT` is `Rc<dyn Fn> + #[derive(Clone)]`; every `.clone()` the macro
//!   emits is a cheap reference-count bump, not a deep copy.
//! - `type ReaderKind` is a type alias for the verbose
//!   `ReaderTKind<Config, IdentityKind>`.  A type alias is a valid `Type::Path`
//!   in the macro grammar; the compiler substitutes the underlying type when
//!   resolving the `Bind` impl.

use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::monad::kind::Bind;
use monadify::transformers::reader::{MonadReader, Reader, ReaderT, ReaderTKind};
use proptest::prelude::*;

/// A minimal read-only configuration environment used throughout these tests.
#[derive(Clone, Debug, PartialEq)]
struct Config {
    base: i32,
    factor: i32,
}

/// Kind marker alias so `mdo! { ReaderKind; … }` is readable without turbofish.
/// Expands to `ReaderTKind<Config, IdentityKind>` at compile time.
type ReaderKind = ReaderTKind<Config, IdentityKind>;

/// Concrete `Reader<Config, A>` — a computation `Config -> Identity<A>`.
type ConfigReader<A> = Reader<Config, A>;

/// Helper: call `ask()` pinned to `Config` / `IdentityKind` so tests are less verbose.
fn ask_env() -> ConfigReader<Config> {
    <ReaderKind as MonadReader<Config, Config, IdentityKind>>::ask()
}

/// Helper: run a `ConfigReader<i32>` against a `Config` and unwrap the `Identity`.
fn run_i32(computation: &ConfigReader<i32>, cfg: Config) -> i32 {
    let Identity(value) = (computation.run_reader_t)(cfg);
    value
}

// ── 1. Env threading ──────────────────────────────────────────────────────────

/// Two reader computations bound in sequence receive the SAME environment.
/// `base` and `factor` both come from the *same* `Config` passed at run time.
#[test]
fn reader_mdo_same_env_threaded_to_every_bind() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        base   <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        factor <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        ReaderKind::pure(base * factor)
    };

    assert_eq!(run_i32(&computation, Config { base: 3, factor: 7 }), 21);
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 0,
                factor: 99
            }
        ),
        0
    );
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: -2,
                factor: 5
            }
        ),
        -10
    );
    assert_eq!(run_i32(&computation, Config { base: 1, factor: 1 }), 1);
}

/// Different `Config` values produce different results — the environment is not baked in.
#[test]
fn reader_mdo_different_envs_produce_different_results() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        base   <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        factor <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        ReaderKind::pure(base + factor)
    };

    let r1 = run_i32(
        &computation,
        Config {
            base: 10,
            factor: 20,
        },
    );
    let r2 = run_i32(&computation, Config { base: 1, factor: 2 });
    assert_eq!(r1, 30);
    assert_eq!(r2, 3);
    assert_ne!(r1, r2);
}

// ── 2. `ask` reads the full environment ────────────────────────────────────────

/// `ask_env()` (i.e. `MonadReader::ask()`) binds the entire `Config` in the do-block.
/// A `let` binding derives a value from it inline — no second `ReaderT` needed.
#[test]
fn reader_mdo_ask_exposes_full_environment() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        cfg <- ask_env();
        let derived = cfg.base + cfg.factor;
        ReaderKind::pure(derived)
    };

    assert_eq!(run_i32(&computation, Config { base: 4, factor: 6 }), 10);
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 0);
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: -1,
                factor: 1
            }
        ),
        0
    );
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 100,
                factor: -3
            }
        ),
        97
    );
}

/// `ask` followed by a `let` that computes a string — shows non-`i32` result types work.
#[test]
fn reader_mdo_ask_derives_string_label() {
    let computation: ConfigReader<String> = mdo! {
        ReaderKind;
        cfg <- ask_env();
        let label = format!("base={} factor={}", cfg.base, cfg.factor);
        ReaderKind::pure(label)
    };

    let Identity(result) = (computation.run_reader_t)(Config { base: 7, factor: 3 });
    assert_eq!(result, "base=7 factor=3");
}

// ── 3. `ask` combined with explicit field readers ─────────────────────────────

/// Mixing `ask` with a `ReaderT::new(...)` in the same block — both see the same env.
#[test]
fn reader_mdo_ask_combined_with_field_reader() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        cfg   <- ask_env();
        extra <- ReaderT::new(|cfg: Config| Identity(cfg.factor * 10));
        ReaderKind::pure(cfg.base + extra)
    };

    // cfg.base = 3, extra = factor * 10 = 5 * 10 = 50, result = 53
    assert_eq!(run_i32(&computation, Config { base: 3, factor: 5 }), 53);
    // cfg.base = 0, extra = 0 * 10 = 0, result = 0
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 0);
}

// ── 4. Equivalence: mdo! == hand-written nested bind ─────────────────────────

/// The `mdo!` desugaring and an equivalent hand-written nested `Bind::bind` chain
/// must produce identical results when run against the same environments.
///
/// Closures/functions cannot be compared for structural equality; we compare by
/// *running* both sides against a set of concrete `Config` values.
#[test]
fn reader_mdo_equivalence_matches_hand_written_bind() {
    // lhs: built by the macro (desugared automatically).
    let lhs: ConfigReader<i32> = mdo! {
        ReaderKind;
        base   <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        factor <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        ReaderKind::pure(base.wrapping_add(factor))
    };

    // rhs: hand-written equivalent — the exact desugared form `mdo!` would emit.
    let rhs: ConfigReader<i32> = ReaderKind::bind(
        ReaderT::new(|cfg: Config| Identity(cfg.base)),
        move |base| {
            ReaderKind::bind(
                ReaderT::new(|cfg: Config| Identity(cfg.factor)),
                move |factor| ReaderKind::pure(base.wrapping_add(factor)),
            )
        },
    );

    let envs = [
        Config { base: 0, factor: 0 },
        Config { base: 1, factor: 2 },
        Config {
            base: -5,
            factor: 10,
        },
        Config {
            base: 42,
            factor: -7,
        },
        Config {
            base: i32::MAX,
            factor: i32::MIN,
        },
    ];

    for env in &envs {
        let lhs_result = run_i32(&lhs, env.clone());
        let rhs_result = run_i32(&rhs, env.clone());
        assert_eq!(
            lhs_result, rhs_result,
            "mdo! and hand-written bind disagree for env {:?}",
            env
        );
    }
}

/// Same equivalence check but for a three-step chain (depth 3).
#[test]
fn reader_mdo_equivalence_three_step_chain() {
    let lhs: ConfigReader<i32> = mdo! {
        ReaderKind;
        b1 <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        b2 <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        f  <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        ReaderKind::pure(b1.wrapping_add(b2).wrapping_add(f))
    };

    let rhs: ConfigReader<i32> =
        ReaderKind::bind(ReaderT::new(|cfg: Config| Identity(cfg.base)), move |b1| {
            ReaderKind::bind(ReaderT::new(|cfg: Config| Identity(cfg.base)), move |b2| {
                ReaderKind::bind(ReaderT::new(|cfg: Config| Identity(cfg.factor)), move |f| {
                    ReaderKind::pure(b1.wrapping_add(b2).wrapping_add(f))
                })
            })
        });

    let envs = [
        Config { base: 0, factor: 0 },
        Config {
            base: 10,
            factor: 3,
        },
        Config {
            base: -4,
            factor: 8,
        },
    ];

    for env in &envs {
        let lhs_result = run_i32(&lhs, env.clone());
        let rhs_result = run_i32(&rhs, env.clone());
        assert_eq!(
            lhs_result, rhs_result,
            "three-step mdo! and hand-written bind disagree for env {:?}",
            env
        );
    }
}

// ── 4b. Property-based equivalence ───────────────────────────────────────────
//
// Reader results are functions (`Config -> Identity<A>`), so they cannot be
// compared structurally. Instead we generate a random environment, build BOTH
// the `mdo!` block and the hand-written `Marker::bind` chain, RUN each against
// the same generated `Config`, and compare the produced values. The desugaring
// identity must hold for every generated environment.
//
// `base`/`factor` are drawn from `any::<i32>()`; arithmetic uses `wrapping_*`
// so no overflow panic can occur for extreme generated inputs.

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// For every generated `Config`, running the `mdo!` desugaring and the
    /// hand-written nested `bind` chain against that same environment must yield
    /// identical results.
    #[test]
    fn reader_mdo_equivalence_prop(base in any::<i32>(), factor in any::<i32>()) {
        let env = Config { base, factor };

        // lhs: built by the macro (desugared automatically).
        let lhs: ConfigReader<i32> = mdo! {
            ReaderKind;
            b <- ReaderT::new(|cfg: Config| Identity(cfg.base));
            f <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
            ReaderKind::pure(b.wrapping_mul(f).wrapping_add(b).wrapping_add(f))
        };

        // rhs: hand-written equivalent — the exact desugared form `mdo!` emits.
        let rhs: ConfigReader<i32> = ReaderKind::bind(
            ReaderT::new(|cfg: Config| Identity(cfg.base)),
            move |b| {
                ReaderKind::bind(
                    ReaderT::new(|cfg: Config| Identity(cfg.factor)),
                    move |f| ReaderKind::pure(b.wrapping_mul(f).wrapping_add(b).wrapping_add(f)),
                )
            },
        );

        let lhs_result = run_i32(&lhs, env.clone());
        let rhs_result = run_i32(&rhs, env.clone());
        prop_assert_eq!(lhs_result, rhs_result);
    }
}

// ── 5. Three-binding chain ────────────────────────────────────────────────────

/// Three bindings with all three reading from the environment demonstrate depth-3
/// desugaring is correct.
#[test]
fn reader_mdo_three_bindings_all_see_same_env() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        b1 <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        b2 <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        f  <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        ReaderKind::pure(b1 + b2 + f)
    };

    // b1 = 10, b2 = 10, f = 3 → 23
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: 10,
                factor: 3
            }
        ),
        23
    );
    // b1 = 0, b2 = 0, f = 0 → 0
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 0);
    // b1 = -1, b2 = -1, f = 5 → 3
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: -1,
                factor: 5
            }
        ),
        3
    );
}

/// Four bindings — the deepest nesting level in these tests.
#[test]
fn reader_mdo_four_bindings_correct_at_depth() {
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        a <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        b <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        c <- ReaderT::new(|cfg: Config| Identity(cfg.base * 2));
        d <- ReaderT::new(|cfg: Config| Identity(cfg.factor * 2));
        ReaderKind::pure(a + b + c + d)
    };

    // a=5, b=3, c=10, d=6 → 24
    let cfg = Config { base: 5, factor: 3 };
    assert_eq!(run_i32(&computation, cfg), 24);
}

// ── 6. `local` modifies the env for a sub-computation ─────────────────────────

/// `local(f, comp)` makes `comp` see a transformed environment.
/// Earlier bindings in the same do-block see the ORIGINAL environment.
#[test]
fn reader_mdo_local_doubles_factor_for_inner_computation() {
    // Sub-computation that reads `factor`.
    let read_factor: ConfigReader<i32> = ReaderT::new(|cfg: Config| Identity(cfg.factor));

    // Wrap it so its env has `factor` doubled.
    let doubled_factor: ConfigReader<i32> =
        <ReaderKind as MonadReader<Config, i32, IdentityKind>>::local(
            |mut cfg: Config| {
                cfg.factor *= 2;
                cfg
            },
            read_factor,
        );

    // In the do-block:
    //   `original` reads from the REAL env (factor = 5)
    //   `modified` runs `doubled_factor` which pre-transforms env → factor = 10
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        original <- ReaderT::new(|cfg: Config| Identity(cfg.factor));
        modified  <- doubled_factor;
        ReaderKind::pure(original + modified)
    };

    let cfg = Config { base: 0, factor: 5 };
    // original = 5, modified = 10, sum = 15
    assert_eq!(run_i32(&computation, cfg), 15);
}

/// `local` that inflates only `base` does not affect bindings that read `factor`.
///
/// **Macro depth note:** the `mdo!` macro emits `move` closures; a non-Copy
/// captured value at depth N can only be *borrowed* (via `.clone()`) at depth N,
/// not moved into depth N+1 (that would be a move-out-of-FnMut).  To keep
/// `inflated_base` at depth 1, we read `base` and `factor` in a single step using
/// a tuple reader instead of two separate bind steps.
#[test]
fn reader_mdo_local_only_affects_its_inner_computation() {
    // local inflates `base` × 100 for its wrapped computation
    let inflated_base: ConfigReader<i32> =
        <ReaderKind as MonadReader<Config, i32, IdentityKind>>::local(
            |mut cfg: Config| {
                cfg.base *= 100;
                cfg
            },
            ReaderT::new(|cfg: Config| Identity(cfg.base)),
        );

    // Read `base` and `factor` together in one step so `inflated_base` (non-Copy)
    // is only referenced at depth 1 in the mdo expansion.  The bound tuple
    // components `original_base: i32` and `factor: i32` are `Copy`, so the
    // innermost `move |inflated|` captures them without triggering E0507.
    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        (original_base, factor) <- ReaderT::new(|cfg: Config| Identity((cfg.base, cfg.factor)));
        inflated                <- inflated_base;
        ReaderKind::pure(original_base + factor + inflated)
    };

    let cfg = Config { base: 2, factor: 3 };
    // original_base = 2, factor = 3, inflated = 2 * 100 = 200, total = 205
    let Identity(result) = (computation.run_reader_t)(cfg);
    assert_eq!(result, 205);
}

/// `local` that negates `base` then combines with original `base` — verifies the
/// two environments are kept distinct.
#[test]
fn reader_mdo_local_negated_base_combined_with_original() {
    let negated_base: ConfigReader<i32> =
        <ReaderKind as MonadReader<Config, i32, IdentityKind>>::local(
            |mut cfg: Config| {
                cfg.base = -cfg.base;
                cfg
            },
            ReaderT::new(|cfg: Config| Identity(cfg.base)),
        );

    let computation: ConfigReader<i32> = mdo! {
        ReaderKind;
        pos_base <- ReaderT::new(|cfg: Config| Identity(cfg.base));
        neg_base <- negated_base;
        ReaderKind::pure(pos_base + neg_base)
    };

    // pos_base = 7, neg_base = -7 → sum = 0
    assert_eq!(run_i32(&computation, Config { base: 7, factor: 0 }), 0);
    // pos_base = 0 → 0
    assert_eq!(run_i32(&computation, Config { base: 0, factor: 0 }), 0);
    // pos_base = -3 → pos + neg = -3 + 3 = 0
    assert_eq!(
        run_i32(
            &computation,
            Config {
                base: -3,
                factor: 0
            }
        ),
        0
    );
}
