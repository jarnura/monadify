//! # Kind (Higher-Kinded Type) Infrastructure
//!
//! This module provides the core traits and marker types for simulating
//! Higher-Kinded Types (more generically, "Kinds") in Rust. This allows for
//! abstracting over type constructors like `Option`, `Vec`, `Result<_, E>`, etc.,
//! which is fundamental for defining generic functional programming traits like
//! `Functor`, `Applicative`, and `Monad` in the `monadify` library,
//! enabling them to operate over these different type constructors.
//!
//! The primary mechanism used is a combination of:
//! 1.  A central `Kind` trait with a Generic Associated Type (GAT) `Of<Arg>`.
//! 2.  Marker structs (e.g., `OptionKind`) that implement `Kind` and specify
//!     what `Of<Arg>` resolves to for their respective type constructor.
//!
//! This setup enables traits to be generic over the *marker type*, and through
//! the marker's `Of<Arg>` GAT, they can refer to the concrete type
//! (e.g., `Option<String>`, `Vec<i32>`).

use crate::function::{CFnOnce, RcFn};
use std::marker::PhantomData;

/// Represents a type constructor, often referred to as a Kind.
///
/// This trait is the cornerstone of simulating Higher-Kinded Types. It allows
/// abstracting over type constructors that take one type argument, such as
/// `Option<_>`, `Vec<_>`, or `Result<_, E>` (where `E` is fixed).
///
/// Implementors of this trait are typically lightweight "marker" structs
/// (e.g., [`OptionKind`], [`VecKind`]) that don't hold data themselves
/// but serve to identify the type constructor they represent.
pub trait Kind {
    /// The concrete type resulting from applying this Kind (type constructor)
    /// to a type argument `Arg`.
    ///
    /// For example:
    /// - If `Self` is [`OptionKind`], then `Self::Of<Arg>` is `Option<Arg>`.
    /// - If `Self` is [`VecKind`], then `Self::Of<Arg>` is `Vec<Arg>`.
    /// - If `Self` is [`ResultKind<MyError>`], then `Self::Of<Arg>` is `Result<Arg, MyError>`.
    type Of<Arg>: Sized;
}

// --- Marker Structs for common Kinds ---

/// Marker for the `Option` type constructor.
///
/// Implements [`Kind`] such that `OptionKind::Of<T>` resolves to `Option<T>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OptionKind;
impl Kind for OptionKind {
    type Of<Arg> = Option<Arg>;
}

/// Marker for the `Vec` type constructor.
///
/// Implements [`Kind`] such that `VecKind::Of<T>` resolves to `Vec<T>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VecKind;
impl Kind for VecKind {
    type Of<Arg> = Vec<Arg>;
}

/// Marker for the `Result<T, E>` type constructor, where `E` (the error type) is fixed.
///
/// `ResultKind<E>` acts as the constructor for `Result<_, E>`.
/// Implements [`Kind`] such that `ResultKind<E>::Of<T>` resolves to `Result<T, E>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ResultKind<E>(std::marker::PhantomData<E>);

impl<E> ResultKind<E> {
    /// Creates a new marker for `Result<_, E>`.
    /// This is primarily for type inference or explicit construction if needed,
    /// though often `Default::default()` or type inference is sufficient.
    pub const fn new() -> Self {
        ResultKind(std::marker::PhantomData)
    }
}

impl<E> Kind for ResultKind<E> {
    type Of<Arg> = Result<Arg, E>;
}

/// Kind Marker for `CFnOnce<X, _>`. `X` is the fixed input type of the function.
///
/// Implements [`Kind`] such that `CFnOnceKind<X>::Of<Output>` resolves to `CFnOnce<X, Output>`.
#[derive(Default)]
pub struct CFnOnceKind<X>(PhantomData<X>);

impl<X> Kind for CFnOnceKind<X> {
    type Of<Output> = CFnOnce<X, Output>;
}

/// Kind Marker for `RcFn<X, _>`. `X` is the fixed input type of the function.
///
/// Implements [`Kind`] such that `RcFnKind<X>::Of<Output>` resolves to `RcFn<X, Output>`.
///
/// The resolved type `RcFn<X, Output>` is `Clone` (O(1) `Rc` bump),
/// which enables `Applicative` laws over `VecKind` and `mdo!` do-notation blocks.
#[derive(Default)]
pub struct RcFnKind<X>(PhantomData<X>);

impl<X> Kind for RcFnKind<X> {
    type Of<Output> = RcFn<X, Output>;
}

// --- Arity Markers ---

/// Marks a `Kind` that effectively takes one type argument (e.g., `F<A>`).
///
/// In this library's current Kind simulation, all types implementing [`Kind`]
/// (which defines `type Of<Arg>`) inherently fit this "arity-1" concept.
/// This trait serves as a blanket implementation for all `T: Kind`, providing
/// a convenient way to specify this arity in trait bounds if needed, although
/// often just `T: Kind` is sufficient.
pub trait Kind1: Kind {}
impl<T: Kind> Kind1 for T {} // Blanket implementation

// If Kinds with more complex arities were needed, e.g., for Bifunctor `F<A, B>`:
// pub trait Kind2 {
//     type Of<Arg1, Arg2>: Sized;
// }
// For now, `Kind` with a single `Of<Arg>` GAT covers Functor, Applicative, Monad.

// The `concretize` function from the original sketch could be added here if useful.
// It would simply be an identity function on `Self::Of<Arg>`.
// pub trait KindConcretize: Kind {
//     fn concretize<Arg>(value: Self::Of<Arg>) -> Self::Of<Arg> {
//         value
//     }
// }
// impl<T: Kind> KindConcretize for T {}
