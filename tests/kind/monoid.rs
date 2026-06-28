//! Law tests for the `Semigroup`/`Monoid` instances (`()`, `String`, `Vec<T>`).
//!
//! Verifies associativity of `combine` and left/right identity of `empty`.

use monadify::monoid::{Monoid, Semigroup};

// ── () : the trivial monoid ──────────────────────────────────────────────────

#[test]
fn unit_monoid_is_trivial() {
    #[allow(clippy::unit_arg, clippy::let_unit_value)]
    let combined = ().combine(());
    assert_eq!(combined, ());
    assert_eq!(<() as Monoid>::empty(), ());
}

// ── String under concatenation ──────────────────────────────────────────────

#[test]
fn string_semigroup_associativity() {
    let a = "foo".to_string();
    let b = "bar".to_string();
    let c = "baz".to_string();
    let left = a.clone().combine(b.clone()).combine(c.clone());
    let right = a.combine(b.combine(c));
    assert_eq!(left, right);
    assert_eq!(left, "foobarbaz");
}

#[test]
fn string_monoid_identity() {
    let x = "hello".to_string();
    assert_eq!(String::empty().combine(x.clone()), x);
    assert_eq!(x.clone().combine(String::empty()), x);
}

// ── Vec<T> under concatenation ──────────────────────────────────────────────

#[test]
fn vec_semigroup_associativity() {
    let a = vec![1, 2];
    let b = vec![3];
    let c = vec![4, 5];
    let left = a.clone().combine(b.clone()).combine(c.clone());
    let right = a.combine(b.combine(c));
    assert_eq!(left, right);
    assert_eq!(left, vec![1, 2, 3, 4, 5]);
}

#[test]
fn vec_monoid_identity() {
    let x = vec!['a', 'b', 'c'];
    assert_eq!(Vec::empty().combine(x.clone()), x);
    assert_eq!(x.clone().combine(Vec::empty()), x);
}
