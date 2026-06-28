//! Config loader example demonstrating `ExceptT` for heterogeneous parse error unification.
//!
//! Two raw string fields — port and retries — are parsed independently. Each parse
//! produces an `ExceptT<ParseIntError, IdentityKind, _>`. The error channel is then
//! remapped via `with_except_t` into a domain-specific `ConfigError`. The two
//! `Checked<_>` computations are sequenced with `bind` so the first bad field
//! short-circuits and the second is never attempted.
//!
//! Run with: `cargo run --example except_config_loader --features do-notation`

use monadify::monad::kind::Bind;
use monadify::transformers::except::{Except, ExceptT, ExceptTKind, MonadError};
use monadify::{Identity, IdentityKind};
use std::num::ParseIntError;

/// Domain error distinguishing which config field failed to parse.
#[derive(Clone, PartialEq, Debug)]
enum ConfigError {
    BadPort(String),
    BadRetries(String),
}

/// Fully parsed configuration.
#[derive(PartialEq, Debug)]
struct Config {
    port: u16,
    retries: u32,
}

/// `Except<ConfigError, A>` = `ExceptT<ConfigError, IdentityKind, A>`.
type Checked<A> = Except<ConfigError, A>;

/// Kind marker alias used for sequencing `Checked<_>` computations with `bind`.
type CKind = ExceptTKind<ConfigError, IdentityKind>;

/// Parse `port_s` as `u16` and `retries_s` as `u32`, unifying any
/// `ParseIntError` into `ConfigError` via `with_except_t`.
///
/// The first bad field short-circuits: if `port_s` fails, `retries_s` is
/// never parsed.
fn load(port_s: &str, retries_s: &str) -> Checked<Config> {
    // Build ExceptT<ParseIntError, IdentityKind, u16> from the parse result, then
    // remap the error channel ParseIntError -> ConfigError::BadPort.
    let port_raw: ExceptT<ParseIntError, IdentityKind, u16> =
        <ExceptTKind<ParseIntError, IdentityKind> as MonadError<
            ParseIntError,
            u16,
            IdentityKind,
        >>::lift_either(port_s.parse::<u16>());
    let port_c: Checked<u16> = port_raw.with_except_t(|e| ConfigError::BadPort(e.to_string()));

    // Same for retries, remapping to ConfigError::BadRetries.
    let retries_raw: ExceptT<ParseIntError, IdentityKind, u32> =
        <ExceptTKind<ParseIntError, IdentityKind> as MonadError<
            ParseIntError,
            u32,
            IdentityKind,
        >>::lift_either(retries_s.parse::<u32>());
    let retries_c: Checked<u32> =
        retries_raw.with_except_t(|e| ConfigError::BadRetries(e.to_string()));

    // Sequence: port_c >>= \port -> retries_c >>= \retries -> Ok(Config { .. })
    // Short-circuit: Err in port_c skips the inner bind entirely.
    CKind::bind(port_c, move |port| {
        CKind::bind(retries_c.clone(), move |retries| {
            <CKind as MonadError<ConfigError, Config, IdentityKind>>::lift_either(Ok(Config {
                port,
                retries,
            }))
        })
    })
}

fn main() {
    println!("=== ExceptT Config Loader: unifying parse errors via with_except_t ===\n");

    // Test 1: both fields valid.
    let Identity(res1) = load("8080", "3").run_except_t;
    assert_eq!(
        res1,
        Ok(Config {
            port: 8080,
            retries: 3
        }),
        "Test 1 failed"
    );
    println!("load(\"8080\", \"3\")      => {:?}  PASSED", res1);

    // Test 2: bad port (non-numeric string) => BadPort; retries never parsed.
    let Identity(res2) = load("notaport", "3").run_except_t;
    assert!(
        matches!(res2, Err(ConfigError::BadPort(_))),
        "Test 2 failed: expected Err(BadPort(_)), got {:?}",
        res2
    );
    println!("load(\"notaport\", \"3\")  => {:?}  PASSED", res2);

    // Test 3: port ok, bad retries => BadRetries.
    let Identity(res3) = load("8080", "xx").run_except_t;
    assert!(
        matches!(res3, Err(ConfigError::BadRetries(_))),
        "Test 3 failed: expected Err(BadRetries(_)), got {:?}",
        res3
    );
    println!("load(\"8080\", \"xx\")     => {:?}  PASSED", res3);

    // Test 4: port overflow (99999 > u16::MAX 65535) => ParseIntError => BadPort.
    let Identity(res4) = load("99999", "3").run_except_t;
    assert!(
        matches!(res4, Err(ConfigError::BadPort(_))),
        "Test 4 failed: expected Err(BadPort(_)), got {:?}",
        res4
    );
    println!("load(\"99999\", \"3\")     => {:?}  PASSED", res4);

    println!("\n=== All assertions passed! ===");
    println!("\nKey insight:");
    println!("  * lift_either      wraps a Result into ExceptT<ParseIntError, IdentityKind, _>.");
    println!("  * with_except_t    remaps the error channel: ParseIntError -> ConfigError.");
    println!("  * bind             sequences Checked<_> steps, short-circuiting on Err.");
    println!("  * run_except_t     is a field exposing Identity<Result<Config, ConfigError>>.");
}
