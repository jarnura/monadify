//! Do-notation with ReaderT: environment threading example.
//!
//! Demonstrates the "real power" of Reader monad: threading a read-only configuration
//! through a multi-step computation without passing it explicitly at each step.
//!
//! Run with: `cargo run --example reader_config --features do-notation`

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::reader::{Reader, ReaderT, ReaderTKind};

/// Application configuration: base rate and scaling factor.
#[derive(Clone, Debug, PartialEq)]
struct AppConfig {
    base_rate: f64,
    scaling_factor: f64,
}

/// Type alias: a computation that reads `AppConfig` and returns an `Identity<A>`.
/// `Reader<E, A> = ReaderT<E, IdentityKind, A>`.
type ConfigReader<A> = Reader<AppConfig, A>;

/// Kind marker alias for the `mdo!` macro: `ReaderTKind<AppConfig, IdentityKind>`.
type ReaderKind = ReaderTKind<AppConfig, IdentityKind>;

/// Helper: unwrap the `Identity` from the `ReaderT` result.
fn run_reader<A>(comp: &ConfigReader<A>, cfg: AppConfig) -> A {
    let Identity(value) = (comp.run_reader_t)(cfg);
    value
}

/// Step 1: Fetch the base rate from config.
fn get_base_rate() -> ConfigReader<f64> {
    ReaderT::new(|cfg: AppConfig| Identity(cfg.base_rate))
}

/// Step 2: Fetch the scaling factor from config.
fn get_scaling_factor() -> ConfigReader<f64> {
    ReaderT::new(|cfg: AppConfig| Identity(cfg.scaling_factor))
}

/// Step 3: Compute a derived value (e.g., adjusted rate) based on the config.
fn compute_adjusted_rate(base: f64, factor: f64) -> ConfigReader<f64> {
    ReaderT::new(move |_cfg: AppConfig| {
        // The environment is available but we use the computed values instead.
        // In practice, you might read additional fields from _cfg here.
        Identity(base * factor)
    })
}

/// Helper: access the entire config with `ask()`.
fn ask_config() -> ConfigReader<AppConfig> {
    ReaderKind::ask()
}

/// ─────────────────────────────────────────────────────────────────────────────
/// NOTE: The "BEFORE" pattern (manually threading environment through nested
/// closures) is less readable and error-prone. The `mdo!` version (see AFTER) is
/// much clearer and automatically threads the environment through all steps.
/// ─────────────────────────────────────────────────────────────────────────────
///
/// ─────────────────────────────────────────────────────────────────────────────
/// AFTER: Using `mdo!` do-notation (flat, readable, environment threaded implicitly)
/// ─────────────────────────────────────────────────────────────────────────────
fn compute_pricing() -> ConfigReader<String> {
    mdo! {
        ReaderKind;
        base <- get_base_rate();
        factor <- get_scaling_factor();
        adjusted <- compute_adjusted_rate(base, factor);
        let label = format!("Base: {:.2}, Factor: {:.2}, Adjusted: {:.2}", base, factor, adjusted);
        pure(label)
    }
}

/// An alternative version that uses `ask` to access the config directly.
fn compute_pricing_with_ask() -> ConfigReader<String> {
    mdo! {
        ReaderKind;
        cfg <- ask_config();
        let adjusted = cfg.base_rate * cfg.scaling_factor;
        let label = format!(
            "Config-based: Base={:.2}, Factor={:.2}, Adjusted={:.2}",
            cfg.base_rate, cfg.scaling_factor, adjusted
        );
        pure(label)
    }
}

/// A more complex multi-step calculation showing environment threading across many steps.
fn multi_step_pricing() -> ConfigReader<(f64, f64, f64)> {
    mdo! {
        ReaderKind;
        base <- get_base_rate();
        factor <- get_scaling_factor();
        adjusted <- compute_adjusted_rate(base, factor);
        pure((base, factor, adjusted))
    }
}

fn main() {
    println!("=== Do-notation with ReaderT: Environment Threading Example ===\n");

    // Create two different configurations to demonstrate environment threading.
    let config1 = AppConfig {
        base_rate: 10.0,
        scaling_factor: 1.5,
    };

    let config2 = AppConfig {
        base_rate: 20.0,
        scaling_factor: 2.0,
    };

    let config3 = AppConfig {
        base_rate: 5.5,
        scaling_factor: 3.2,
    };

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 1: Basic computation against config1
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 1: Basic do-notation computation");
    let computation = compute_pricing();
    let result1 = run_reader(&computation, config1.clone());
    println!("  Config: {:?}", config1);
    println!("  Result: {}", result1);
    assert_eq!(result1, "Base: 10.00, Factor: 1.50, Adjusted: 15.00");
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 2: Same computation against different config
    // Shows that the environment threading is correct — different inputs produce different results
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 2: Same computation against different config");
    let result2 = run_reader(&computation, config2.clone());
    println!("  Config: {:?}", config2);
    println!("  Result: {}", result2);
    assert_eq!(result2, "Base: 20.00, Factor: 2.00, Adjusted: 40.00");
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 3: Another config
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 3: Third configuration");
    let result3 = run_reader(&computation, config3.clone());
    println!("  Config: {:?}", config3);
    println!("  Result: {}", result3);
    assert_eq!(result3, "Base: 5.50, Factor: 3.20, Adjusted: 17.60");
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 4: Using `ask()` to access the full config
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 4: Using `ask()` to access the full environment");
    let computation_with_ask = compute_pricing_with_ask();
    let result4 = run_reader(&computation_with_ask, config1.clone());
    println!("  Config: {:?}", config1);
    println!("  Result: {}", result4);
    assert_eq!(
        result4,
        "Config-based: Base=10.00, Factor=1.50, Adjusted=15.00"
    );
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 5: Multi-step computation showing deep environment threading
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 5: Multi-step computation (3 bindings)");
    let multi_comp = multi_step_pricing();
    let (base, factor, adjusted) = run_reader(&multi_comp, config2.clone());
    println!("  Config: {:?}", config2);
    println!("  Base: {:.2}", base);
    println!("  Factor: {:.2}", factor);
    println!("  Adjusted (base * factor): {:.2}", adjusted);
    assert_eq!(base, 20.0);
    assert_eq!(factor, 2.0);
    assert_eq!(adjusted, 40.0);
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Summary: Show how environment threading works without explicit parameter passing
    // ─────────────────────────────────────────────────────────────────────────────
    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `ReaderT` demonstrates the power of monadic threading:");
    println!("  • Each step (get_base_rate, get_scaling_factor, etc.) silently receives");
    println!("    the same AppConfig from the environment.");
    println!("  • No explicit parameter passing required — the do-notation macro desugars");
    println!("    to nested `bind` calls that thread the environment automatically.");
    println!("  • This is especially powerful for dependency injection, configuration");
    println!("    management, and multi-step computations in complex applications.");
}
