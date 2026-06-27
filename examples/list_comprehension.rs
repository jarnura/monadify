//! Do-notation as list comprehension: Vec example.
//!
//! Demonstrates the use of `mdo!` with `VecKind` to express list comprehensions,
//! including Cartesian products and filtering with `guard`.
//!
//! Run with: `cargo run --example list_comprehension --features do-notation`

use monadify::{mdo, Applicative, VecKind};

/// ─────────────────────────────────────────────────────────────────────────────
/// BEFORE: Nested flat_map and filter chains (imperative-looking but cluttered)
/// ─────────────────────────────────────────────────────────────────────────────
#[allow(dead_code)]
fn pythagorean_triples_before(limit: i32) -> Vec<(i32, i32, i32)> {
    (1..=limit)
        .flat_map(move |a| {
            (1..=limit).flat_map(move |b| {
                (1..=limit)
                    .filter_map(move |c| {
                        if a * a + b * b == c * c && c <= limit {
                            Some((a, b, c))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect()
}

/// ─────────────────────────────────────────────────────────────────────────────
/// AFTER: Using `mdo!` do-notation (clean list-comprehension style)
/// ─────────────────────────────────────────────────────────────────────────────
fn pythagorean_triples_after(limit: i32) -> Vec<(i32, i32, i32)> {
    mdo! {
        VecKind;
        a <- (1..=limit).collect::<Vec<_>>();
        b <- (1..=limit).collect::<Vec<_>>();
        c <- (1..=limit).collect::<Vec<_>>();
        guard(a * a + b * b == c * c);
        VecKind::pure((a, b, c))
    }
}

/// List comprehension: all even numbers from 1 to n.
fn even_numbers(n: i32) -> Vec<i32> {
    mdo! {
        VecKind;
        x <- (1..=n).collect::<Vec<_>>();
        guard(x % 2 == 0);
        VecKind::pure(x)
    }
}

/// List comprehension: Cartesian product of two ranges, filtered.
/// Result: pairs (x, y) where x + y <= 10.
fn sum_pairs_under_10(max_x: i32, max_y: i32) -> Vec<(i32, i32)> {
    mdo! {
        VecKind;
        x <- (1..=max_x).collect::<Vec<_>>();
        y <- (1..=max_y).collect::<Vec<_>>();
        guard(x + y <= 10);
        VecKind::pure((x, y))
    }
}

/// List comprehension: all two-digit numbers with repeated digits (11, 22, 33, ..., 99).
fn repeated_digit_numbers() -> Vec<i32> {
    mdo! {
        VecKind;
        digit <- (1..=9).collect::<Vec<_>>();
        VecKind::pure(digit * 10 + digit)
    }
}

/// List comprehension: points on a grid within a certain distance from origin.
fn points_near_origin(max_x: i32, max_y: i32, max_distance_sq: i32) -> Vec<(i32, i32)> {
    mdo! {
        VecKind;
        x <- (-max_x..=max_x).collect::<Vec<_>>();
        y <- (-max_y..=max_y).collect::<Vec<_>>();
        guard(x * x + y * y <= max_distance_sq);
        VecKind::pure((x, y))
    }
}

/// Complex comprehension: multiples of 3 and 5 (but not both).
fn multiples_of_3_xor_5(n: i32) -> Vec<i32> {
    mdo! {
        VecKind;
        x <- (1..=n).collect::<Vec<_>>();
        guard((x % 3 == 0 && x % 5 != 0) || (x % 3 != 0 && x % 5 == 0));
        VecKind::pure(x)
    }
}

fn main() {
    println!("=== Do-notation as List Comprehension: VecKind Example ===\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 1: Even numbers from 1 to 10
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 1: Even numbers from 1 to 10");
    let evens = even_numbers(10);
    println!("  Result: {:?}", evens);
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 2: Pairs (x, y) where x + y <= 10, from ranges [1,5] × [1,6]
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 2: Pairs (x, y) where x + y <= 10");
    let pairs = sum_pairs_under_10(5, 6);
    println!("  Result: {:?}", pairs);
    // (1,1) through (1,6): all valid (1+6=7 ≤ 10)
    // (2,1) through (2,6): all valid (2+6=8 ≤ 10)
    // (3,1) through (3,6): all valid (3+6=9 ≤ 10)
    // (4,1) through (4,6): all valid (4+6=10 ≤ 10)
    // (5,1) through (5,5): 5+5=10 ✓, but 5+6=11 ✗
    let expected = vec![
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (1, 6),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (3, 1),
        (3, 2),
        (3, 3),
        (3, 4),
        (3, 5),
        (3, 6),
        (4, 1),
        (4, 2),
        (4, 3),
        (4, 4),
        (4, 5),
        (4, 6),
        (5, 1),
        (5, 2),
        (5, 3),
        (5, 4),
        (5, 5),
    ];
    assert_eq!(pairs, expected);
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 3: Repeated digit numbers (11, 22, 33, ..., 99)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 3: Repeated digit numbers");
    let repeated = repeated_digit_numbers();
    println!("  Result: {:?}", repeated);
    assert_eq!(repeated, vec![11, 22, 33, 44, 55, 66, 77, 88, 99]);
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 4: Points near origin (within distance 5)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 4: Points near origin (within distance² <= 25)");
    let points = points_near_origin(5, 5, 25);
    println!("  Count: {}", points.len());
    println!("  Sample points: {:?}", &points[0..10.min(points.len())]);
    // Points within distance 5: (0,0), (±1,0), (0,±1), (±2,0), etc.
    // Should include roughly a circle of radius 5
    assert!(points.len() > 20); // Rough check
    assert!(points.contains(&(0, 0)));
    assert!(points.contains(&(5, 0)));
    assert!(points.contains(&(3, 4))); // 3² + 4² = 25
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 5: Multiples of 3 XOR 5 from 1 to 30
    // (multiples of 3 but not 5, OR multiples of 5 but not 3)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 5: Multiples of 3 XOR 5 (1 to 30)");
    let xor_multiples = multiples_of_3_xor_5(30);
    println!("  Result: {:?}", xor_multiples);
    // Multiples of 3: 3, 6, 9, 12, 15, 18, 21, 24, 27, 30
    // Multiples of 5: 5, 10, 15, 20, 25, 30
    // XOR: 3, 5, 6, 9, 10, 12, 18, 20, 21, 24, 25, 27 (excluding 15, 30 which are both)
    let expected_xor = vec![3, 5, 6, 9, 10, 12, 18, 20, 21, 24, 25, 27];
    assert_eq!(xor_multiples, expected_xor);
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 6: Pythagorean triples with low limit (3, 4, 5)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 6: Pythagorean triples (limit=5)");
    let triples = pythagorean_triples_after(5);
    println!("  Result: {:?}", triples);
    // a² + b² = c² with a,b,c ∈ [1,5]
    // Only (3,4,5) and (4,3,5) in this range
    assert!(triples.contains(&(3, 4, 5)) || triples.contains(&(4, 3, 5)));
    println!("  ✓ PASSED\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // Test 7: Pythagorean triples with larger limit
    // ─────────────────────────────────────────────────────────────────────────────
    println!("Test 7: Pythagorean triples (limit=13)");
    let triples_large = pythagorean_triples_after(13);
    println!("  Result: {:?}", triples_large);
    // Should find (3,4,5), (4,3,5), (5,12,13), (6,8,10), (8,6,10), (12,5,13) etc.
    assert!(!triples_large.is_empty());
    println!("  Count: {}", triples_large.len());
    println!("  ✓ PASSED\n");

    println!("=== All tests passed! ===");
    println!("\nKey insight: `mdo!` with `VecKind` expresses list comprehensions");
    println!("as readable, imperative sequences instead of nested flat_map/filter chains.");
    println!("The `guard` statement filters elements just like list comprehension guards");
    println!("in languages like Python, Haskell, or Scala.");
}
