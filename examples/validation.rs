//! Do-notation validation pipeline example.
//!
//! Demonstrates the power of `mdo!` for short-circuit validation:
//! a sequence of fallible operations using Option/Result.
//!
//! Run with: `cargo run --example validation --features do-notation`

use monadify::{mdo, OptionKind, ResultKind};

/// Check if a number is in a valid range.
fn validate_range(n: i32, min: i32, max: i32) -> Option<i32> {
    if n >= min && n <= max {
        Some(n)
    } else {
        None
    }
}

/// Check if a number is positive.
fn is_positive(n: i32) -> Option<i32> {
    if n > 0 {
        Some(n)
    } else {
        None
    }
}

/// Represents validation output.
#[derive(Debug, Clone, PartialEq)]
struct ValidationResult {
    original: i32,
    squared: i32,
    sum: i32,
}

/// ─────────────────────────────────────────────────────────────────────────────
/// BEFORE: Nested and_then / manual bind chains (hard to read)
/// ─────────────────────────────────────────────────────────────────────────────
#[allow(dead_code)]
fn process_numbers_before(a: i32, b: i32) -> Option<ValidationResult> {
    is_positive(a).and_then(|a_val| {
        is_positive(b).and_then(|b_val| {
            validate_range(a_val, 1, 100).and_then(|a_checked| {
                validate_range(b_val, 1, 100).map(|b_checked| ValidationResult {
                    original: a_checked + b_checked,
                    squared: a_checked * a_checked + b_checked * b_checked,
                    sum: a_checked + b_checked,
                })
            })
        })
    })
}

/// ─────────────────────────────────────────────────────────────────────────────
/// AFTER: Using `mdo!` do-notation (flat, imperative style, easy to read)
/// ─────────────────────────────────────────────────────────────────────────────
fn process_numbers_after(a: i32, b: i32) -> Option<ValidationResult> {
    mdo! {
        OptionKind;
        a_val <- is_positive(a);
        b_val <- is_positive(b);
        a_checked <- validate_range(a_val, 1, 100);
        b_checked <- validate_range(b_val, 1, 100);
        pure(ValidationResult {
            original: a_checked + b_checked,
            squared: a_checked * a_checked + b_checked * b_checked,
            sum: a_checked + b_checked,
        })
    }
}

/// Result-based version with error messages.
fn process_numbers_with_errors(a: i32, b: i32) -> Result<ValidationResult, String> {
    mdo! {
        ResultKind::<String>;
        a_val <- if a > 0 {
            Ok(a)
        } else {
            Err("First number must be positive".to_string())
        };
        b_val <- if b > 0 {
            Ok(b)
        } else {
            Err("Second number must be positive".to_string())
        };
        a_checked <- if a_val <= 100 {
            Ok(a_val)
        } else {
            Err(format!("First number {} exceeds max 100", a_val))
        };
        b_checked <- if b_val <= 100 {
            Ok(b_val)
        } else {
            Err(format!("Second number {} exceeds max 100", b_val))
        };
        pure(ValidationResult {
            original: a_checked + b_checked,
            squared: a_checked * a_checked + b_checked * b_checked,
            sum: a_checked + b_checked,
        })
    }
}

fn main() {
    println!("=== Do-notation Validation Pipeline Example ===\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // SUCCESS CASE: All validations pass
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 1: Valid numbers (both positive and in range)");
    let result1 = process_numbers_after(5, 7);
    println!("  process_numbers_after(5, 7): {:?}", result1);
    assert_eq!(
        result1,
        Some(ValidationResult {
            original: 12,
            squared: 74, // 5² + 7² = 25 + 49 = 74
            sum: 12,
        })
    );
    println!("  ✓ SUCCESS\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // FAILURE CASE 1: First number negative (short-circuits immediately)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 2: First number negative (short-circuit at first step)");
    let result2 = process_numbers_after(-5, 7);
    println!("  process_numbers_after(-5, 7): {:?}", result2);
    assert_eq!(result2, None);
    println!("  ✓ SHORT-CIRCUIT: First validation failed, remaining steps skipped\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // FAILURE CASE 2: Second number negative (short-circuits at second step)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 3: Second number negative (short-circuit at second step)");
    let result3 = process_numbers_after(5, -3);
    println!("  process_numbers_after(5, -3): {:?}", result3);
    assert_eq!(result3, None);
    println!("  ✓ SHORT-CIRCUIT: Second validation failed after first succeeded\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // FAILURE CASE 3: First number out of range (short-circuits at third step)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 4: First number exceeds range (short-circuit at third step)");
    let result4 = process_numbers_after(150, 7);
    println!("  process_numbers_after(150, 7): {:?}", result4);
    assert_eq!(result4, None);
    println!("  ✓ SHORT-CIRCUIT: Range check failed after positivity checks passed\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // FAILURE CASE 4: Second number out of range (short-circuits at fourth step)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 5: Second number exceeds range (short-circuit at fourth step)");
    let result5 = process_numbers_after(5, 200);
    println!("  process_numbers_after(5, 200): {:?}", result5);
    assert_eq!(result5, None);
    println!("  ✓ SHORT-CIRCUIT: Second range check failed\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // RESULT-BASED: With detailed error messages
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 6: Result-based version with error messages");
    let result6a = process_numbers_with_errors(10, 20);
    println!("  Success case (10, 20): {:?}", result6a);
    assert!(result6a.is_ok());

    let result6b = process_numbers_with_errors(-5, 20);
    println!("  First number negative: {:?}", result6b);
    assert_eq!(result6b, Err("First number must be positive".to_string()));

    let result6c = process_numbers_with_errors(10, -3);
    println!("  Second number negative: {:?}", result6c);
    assert_eq!(result6c, Err("Second number must be positive".to_string()));

    let result6d = process_numbers_with_errors(150, 20);
    println!("  First number out of range: {:?}", result6d);
    assert_eq!(
        result6d,
        Err("First number 150 exceeds max 100".to_string())
    );

    let result6e = process_numbers_with_errors(10, 101);
    println!("  Second number out of range: {:?}", result6e);
    assert_eq!(
        result6e,
        Err("Second number 101 exceeds max 100".to_string())
    );
    println!("  ✓ ALL ERROR CASES PASSED\n");

    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` makes validation pipelines readable and maintainable");
    println!("by eliminating deeply nested closures and using imperative sequencing.");
    println!("When one step fails, the entire computation short-circuits to None/Err");
    println!("without executing the remaining steps.");
}
