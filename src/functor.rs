pub mod kind {
    // Renamed from hkt to kind to align with the Kind trait
    //! # Kind-based Functor for the `monadify` library
    //!
    //! This module defines the `Functor` trait for types that implement the Kind pattern.
    //! A Functor is a type constructor `F` (represented by a `Kind` marker)
    //! that supports a `map` operation. This operation allows a function `A -> B`
    //! to be applied to a value or values within a structure `F::Of<A>` (e.g., `Option<A>`),
    //! resulting in a new structure `F::Of<B>` (e.g., `Option<B>`).
    //!
    //! The `Functor` trait is generic over:
    //! - `Self`: The Kind marker (e.g., [`OptionKind`]).
    //! - `A`: The input type of the value(s) within the Kind-encoded structure.
    //! - `B`: The output type after applying the mapping function.
    //!
    //! It relies on the [`Kind1`] trait from `crate::kind_based::kind` to relate the
    //! marker `Self` to its concrete type application `Self::Of<T>`.

    use crate::function::{CFnOnce, RcFn};
    use crate::kind_based::kind::{CFnOnceKind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind};
    use std::rc::Rc;

    /// Represents a type constructor that can be mapped over, using the Kind pattern.
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`], [`VecKind`])
    /// that implements [`Kind1`].
    ///
    /// The `map` operation takes an instance of `Self::Of<A>` (e.g., `Option<A>`)
    /// and a function `A -> B`, producing `Self::Of<B>` (e.g., `Option<B>`).
    ///
    /// ## Functor Laws
    /// Implementors must satisfy two laws:
    /// 1.  **Identity**: For any Kind marker `F` and value `x: F::Of<A>`,
    ///     `F::map(x, |a| a) == x`.
    /// 2.  **Composition**: For any Kind marker `F`, value `x: F::Of<A>`, and functions
    ///     `f: A -> B`, `g: B -> C`,
    ///     `F::map(F::map(x, f), g) == F::map(x, |a| g(f(a)))`.
    pub trait Functor<A, B>: Kind1 {
        // A is input type, B is output type for map
        /// Applies a function to a value (or values) within a Kind-encoded structure.
        ///
        /// # Type Parameters
        /// - `Self`: The Kind marker (e.g., [`OptionKind`]).
        /// - `A`: The type of the value(s) inside the input structure `Self::Of<A>`.
        /// - `B`: The type of the value(s) inside the output structure `Self::Of<B>`.
        ///
        /// # Parameters
        /// - `input`: The Kind-structured value, e.g., `Option<A>`, `Vec<A>`.
        /// - `func`: A function to apply to the inner value(s).
        ///   The `FnMut(A) -> B + Clone + 'static` bound is chosen for broad compatibility:
        ///   - `FnMut`: Allows mutation of captured state if needed, and covers `Fn` and `FnOnce`
        ///     that don't consume their captures by value on first call. Standard library `map`
        ///     methods (like `Option::map`, `Result::map`) often take `FnOnce`.
        ///   - `Clone`: Necessary for some implementations like [`RcFnKind`], where the
        ///     function might need to be cloned if the resulting structure can be "called"
        ///     multiple times.
        ///   - `'static`: Often required when functions are stored or returned within structures
        ///     like [`RcFn`] or [`CFnOnce`], especially if they don't borrow from the local scope.
        ///
        /// # Returns
        /// A new Kind-structured value `Self::Of<B>` containing the result(s) of
        /// applying `func`.
        fn map(input: Self::Of<A>, func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B>;
    }

    impl<A, B> Functor<A, B> for OptionKind {
        fn map(input: Self::Of<A>, func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B> {
            input.map(func) // Option::map takes FnOnce, FnMut is compatible
        }
    }

    impl<A, B, E> Functor<A, B> for ResultKind<E> {
        fn map(input: Self::Of<A>, func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B> {
            input.map(func) // Result::map takes FnOnce, FnMut is compatible
        }
    }

    impl<A, B> Functor<A, B> for VecKind {
        fn map(input: Self::Of<A>, func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B> {
            input.into_iter().map(func).collect()
        }
    }

    // Functor impl for RcFnKind (maps over the output type of RcFn)
    // A is the original output type, B is the new output type.
    impl<X, A, B> Functor<A, B> for RcFnKind<X>
    where
        X: 'static,
        A: 'static,
        B: 'static,
    {
        /// Maps a function `A -> B` over the output of an `RcFn<X, A>`,
        /// producing an `RcFn<X, B>`.
        ///
        /// The inner `Rc` is cloned (O(1)) to capture the original function in
        /// the new closure. `func` is `Clone`-d on each invocation to satisfy
        /// the `Fn` (not `FnMut`) requirement of `Rc<dyn Fn>`.
        fn map(input: Self::Of<A>, func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B> {
            let inner = input.0.clone();
            RcFn(Rc::new(move |x: X| func.clone()(inner(x))))
        }
    }

    // Functor impl for CFnOnceKind (maps over the output type of CFnOnce)
    impl<X, A, B> Functor<A, B> for CFnOnceKind<X>
    where
        X: 'static,
        A: 'static,
        B: 'static, // B must be 'static for CFnOnce<X,B> which is Self::Of<B>
    {
        fn map(input: Self::Of<A>, mut func: impl FnMut(A) -> B + Clone + 'static) -> Self::Of<B> // Added mut for func
        {
            CFnOnce::new(move |x: X| func(input.call_once(x)))
        }
    }
}

// Directly export Kind-based Functor
pub use kind::Functor; // Renamed from hkt to kind
                       // Note: RcFnKind and CFnOnceKind are defined in kind_based::kind
                       // and Functor implementations for them are in the kind module above.
                       // This re-export makes `crate::functor::Functor` point to the Kind-based one.
