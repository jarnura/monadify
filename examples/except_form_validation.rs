//! Except-monad demo: user-registration form validation.
//!
//! Demonstrates how `Except<ValidationError, _>` short-circuits on the FIRST
//! invalid field in a user-registration form. Three validation steps are
//! sequenced with `mdo!`; the moment one step calls `throw_error`, all later
//! steps are skipped and the error propagates — later checks never run.
//!
//! ## Design notes
//!
//! Each `check_*` helper is a standalone function whose result is an owned
//! `Checked<_>` value. `check_password` accepts `(username, email)` by value so
//! those strings become bind-result *parameters* of the depth-1 closure rather
//! than captured state of the depth-0 closure — this is the key to keeping every
//! generated `move` closure `FnMut` despite the values being non-`Copy`.
//!
//! Run with:
//! ```text
//! cargo run --quiet --example except_form_validation --features do-notation
//! ```

use monadify::identity::{Identity, IdentityKind};
use monadify::mdo;
use monadify::transformers::except::{Except, ExceptTKind, MonadError};

// ── Domain types ──────────────────────────────────────────────────────────────

/// Possible validation failures for a user-registration form.
#[derive(Clone, PartialEq, Debug)]
enum ValidationError {
    /// The username field was left empty.
    EmptyUsername,
    /// The password is shorter than 8 characters.
    PasswordTooShort,
    /// The email address does not contain `@`.
    InvalidEmail,
}

/// A successfully validated and constructed user.
#[derive(Debug, Clone, PartialEq)]
struct User {
    username: String,
    email: String,
}

// ── Kind aliases ──────────────────────────────────────────────────────────────

/// `Except<ValidationError, A>` = `ExceptT<ValidationError, IdentityKind, A>`.
/// Unwrap a value via `let Identity(res) = prog.run_except_t`.
type Checked<A> = Except<ValidationError, A>;

/// Kind marker for the `mdo!` macro first argument.
type CKind = ExceptTKind<ValidationError, IdentityKind>;

// ── Validation steps ──────────────────────────────────────────────────────────

/// Step 1 — username must be non-empty.
///
/// Returns the validated username so step 2 can receive it as a bind-result
/// parameter rather than a captured variable.
fn check_username(username: String) -> Checked<String> {
    if username.is_empty() {
        <CKind as MonadError<ValidationError, String, IdentityKind>>::throw_error(
            ValidationError::EmptyUsername,
        )
    } else {
        <CKind as MonadError<ValidationError, String, IdentityKind>>::lift_either(Ok(username))
    }
}

/// Step 2 — password must be at least 8 characters.
///
/// Accepts `username` and `email` by value and carries them forward as
/// `Ok((username, email))` on success. This threading pattern keeps both strings
/// as bind-result *parameters* of the depth-1 closure, avoiding a captured-state
/// move that would turn the depth-0 closure into `FnOnce`.
fn check_password(username: String, password: String, email: String) -> Checked<(String, String)> {
    if password.len() < 8 {
        <CKind as MonadError<ValidationError, (String, String), IdentityKind>>::throw_error(
            ValidationError::PasswordTooShort,
        )
    } else {
        <CKind as MonadError<ValidationError, (String, String), IdentityKind>>::lift_either(Ok((
            username, email,
        )))
    }
}

/// Step 3 — email must contain `@`.
///
/// Builds the final `User` on success.
fn check_email(username: String, email: String) -> Checked<User> {
    if !email.contains('@') {
        <CKind as MonadError<ValidationError, User, IdentityKind>>::throw_error(
            ValidationError::InvalidEmail,
        )
    } else {
        <CKind as MonadError<ValidationError, User, IdentityKind>>::lift_either(Ok(User {
            username,
            email,
        }))
    }
}

// ── Top-level validator ───────────────────────────────────────────────────────

/// Validates a registration form in field order: username -> password -> email.
///
/// The `mdo!` block sequences three `Except` computations. If any step calls
/// `throw_error`, the `ExceptT::bind` implementation propagates the error
/// without invoking the remaining steps.
///
/// `password` and `email` are cloned in the bind expression for step 2 so
/// they remain in the depth-0 closure as borrows (`.clone()`) rather than
/// consumed moves — this keeps the depth-0 continuation `FnMut + Clone`.
fn validate_form(username: String, password: String, email: String) -> Checked<User> {
    mdo! {
        CKind;
        un        <- check_username(username);
        (un2, em) <- check_password(un, password.clone(), email.clone());
        user      <- check_email(un2, em);
        pure(user)
    }
}

// ── Helpers for running ───────────────────────────────────────────────────────

/// Runs a `Checked<User>` computation and returns the inner `Result`.
fn run(prog: Checked<User>) -> Result<User, ValidationError> {
    let Identity(res) = prog.run_except_t;
    res
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Except-monad: User-Registration Form Validation ===\n");

    // ── Test 1: all fields valid ──────────────────────────────────────────────
    println!("Test 1: valid form — expects Ok(User)");
    let result = run(validate_form(
        "alice".to_string(),
        "supersecret".to_string(),
        "alice@example.com".to_string(),
    ));
    assert_eq!(
        result,
        Ok(User {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        }),
        "valid form must produce Ok(User)"
    );
    println!("  Ok(User {{ username: \"alice\", email: \"alice@example.com\" }})");
    println!("  PASSED\n");

    // ── Test 2: empty username — short-circuits at step 1 ────────────────────
    println!("Test 2: empty username — expects Err(EmptyUsername)");
    let result = run(validate_form(
        "".to_string(),
        "supersecret".to_string(),
        "alice@example.com".to_string(),
    ));
    assert_eq!(
        result,
        Err(ValidationError::EmptyUsername),
        "empty username must short-circuit with EmptyUsername"
    );
    println!("  Err(EmptyUsername) -- password and email checks never ran");
    println!("  PASSED\n");

    // ── Test 3: short password — short-circuits at step 2 ────────────────────
    println!("Test 3: valid username, short password — expects Err(PasswordTooShort)");
    let result = run(validate_form(
        "alice".to_string(),
        "abc".to_string(),
        "alice@example.com".to_string(),
    ));
    assert_eq!(
        result,
        Err(ValidationError::PasswordTooShort),
        "short password must short-circuit with PasswordTooShort"
    );
    println!("  Err(PasswordTooShort) -- email check never ran");
    println!("  PASSED\n");

    // ── Test 4: bad email — short-circuits at step 3 ─────────────────────────
    println!("Test 4: valid username and password, bad email — expects Err(InvalidEmail)");
    let result = run(validate_form(
        "alice".to_string(),
        "supersecret".to_string(),
        "notanemail".to_string(),
    ));
    assert_eq!(
        result,
        Err(ValidationError::InvalidEmail),
        "invalid email must short-circuit with InvalidEmail"
    );
    println!("  Err(InvalidEmail)");
    println!("  PASSED\n");

    println!("=== All assertions passed! ===\n");
    println!("Key insight: `mdo!` with `Except<ValidationError, _>` sequences validation:");
    println!("  * `throw_error(e)`      injects an error; all later `bind` steps are skipped.");
    println!("  * `lift_either(Ok(x))`  lifts a success value into the Except monad.");
    println!("  * `ExceptT::bind`       propagates `Err` without running the continuation.");
    println!("  * `run_except_t`        is a VALUE field; unwrap via `let Identity(res) = prog.run_except_t`.");
    println!("  * Each step carries forward the data the next step needs as its bind-result");
    println!("    parameter, keeping all generated `move` closures `FnMut + Clone`.");
}
