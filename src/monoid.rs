//! # Semigroup and Monoid
//!
//! Algebraic structures for *combinable* values. A [`Semigroup`] has an
//! associative binary operation [`combine`](Semigroup::combine) (Haskell's
//! `<>` / `mappend`); a [`Monoid`] additionally has an identity element
//! [`empty`](Monoid::empty) (Haskell's `mempty`).
//!
//! These power the [`WriterT`](crate::transformers::writer) transformer, whose
//! log type must accumulate monoidally: `tell` appends with `combine`, and
//! `pure` starts the log at `empty`.
//!
//! ## Laws
//!
//! `Semigroup` — **associativity**:
//! ```text
//! a.combine(b).combine(c) == a.combine(b.combine(c))
//! ```
//! `Monoid` — **left and right identity**:
//! ```text
//! Monoid::empty().combine(x) == x
//! x.combine(Monoid::empty()) == x
//! ```
//!
//! ## Provided instances
//! - `()` — the trivial monoid (one element, the no-op log).
//! - `String` — concatenation, identity is `""`.
//! - `Vec<T>` — concatenation, identity is `[]`.
//!
//! ```
//! use monadify::monoid::{Semigroup, Monoid};
//!
//! assert_eq!("foo".to_string().combine("bar".to_string()), "foobar");
//! assert_eq!(vec![1, 2].combine(vec![3]), vec![1, 2, 3]);
//! assert_eq!(<String as Monoid>::empty().combine("x".to_string()), "x");
//! ```

/// A type with an associative binary operation, [`combine`](Self::combine).
///
/// Analogous to Haskell's `Semigroup` (`<>`). Implementations must be
/// **associative**: `a.combine(b).combine(c) == a.combine(b.combine(c))`.
pub trait Semigroup {
    /// Combines two values associatively. Consumes both operands so the
    /// implementation can reuse their allocations (e.g. `String`/`Vec`).
    #[must_use]
    fn combine(self, other: Self) -> Self;
}

/// A [`Semigroup`] with an identity element, [`empty`](Self::empty).
///
/// Analogous to Haskell's `Monoid` (`mempty`). The identity must satisfy
/// `empty().combine(x) == x` and `x.combine(empty()) == x`.
pub trait Monoid: Semigroup {
    /// The identity element: combining it with any value yields that value.
    #[must_use]
    fn empty() -> Self;
}

// --- Trivial monoid: () ---

impl Semigroup for () {
    fn combine(self, _other: ()) {}
}
impl Monoid for () {
    fn empty() {}
}

// --- String under concatenation ---

impl Semigroup for String {
    fn combine(mut self, other: String) -> String {
        self.push_str(&other);
        self
    }
}
impl Monoid for String {
    fn empty() -> String {
        String::new()
    }
}

// --- Vec<T> under concatenation ---

impl<T> Semigroup for Vec<T> {
    fn combine(mut self, mut other: Vec<T>) -> Vec<T> {
        self.append(&mut other);
        self
    }
}
impl<T> Monoid for Vec<T> {
    fn empty() -> Vec<T> {
        Vec::new()
    }
}
