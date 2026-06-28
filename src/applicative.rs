pub mod kind {
    //! # Kind-based Applicative Functor for the `monadify` library
    //!
    //! This module defines the `Applicative` trait for Kind-encoded types, which extends `Apply`.
    //! An Applicative Functor allows lifting a normal value into the Kind context
    //! (via `pure`) and applying a wrapped function to a wrapped value (via `apply`
    //! from the `Apply` supertrait).
    //!
    //! The `Applicative` trait is generic over:
    //! - `Self`: The Kind marker (e.g., [`OptionKind`]).
    //! - `T`: The type of the value being lifted by `pure`.
    //!
    //! ## Example
    //!
    //! ```
    //! use monadify::applicative::kind::{Applicative, lift_a1};
    //! use monadify::apply::kind::Apply;
    //! use monadify::kind_based::kind::OptionKind;
    //! use monadify::function::RcFn;
    //!
    //! // Using pure and apply directly
    //! let val_opt: Option<i32> = OptionKind::pure(10); // Some(10)
    //! let fn_opt: Option<RcFn<i32, i32>> = OptionKind::pure(RcFn::new(|x| x + 1)); // Some(RcFn)
    //!
    //! // Need to specify the marker for apply
    //! let result_opt: Option<i32> = OptionKind::apply(val_opt, fn_opt);
    //! assert_eq!(result_opt, Some(11));
    //!
    //! // Using lift_a1 (which uses pure and apply internally)
    //! let val_opt2: Option<i32> = Some(20);
    //! // Specify the Kind marker for lift_a1 if it cannot be inferred
    //! let result_opt2: Option<i32> = lift_a1::<OptionKind, _, _, _>(|x: i32| x * 2, val_opt2);
    //! assert_eq!(result_opt2, Some(40));
    //! ```
    //!
    //! `Applicative` builds upon `Apply` by adding the `pure` method. This allows
    //! functions and values to be lifted into the context before application.

    use crate::apply::kind::Apply;
    use crate::function::{CFnOnce, RcFn};
    use crate::kind_based::kind::{
        CFnOnceKind, Kind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind,
    };
    use std::rc::Rc;

    /// Represents a Kind-encoded type that is an Applicative Functor.
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`]) that implements
    /// [`Kind1`] and [`Apply`].
    ///
    /// The primary method provided by `Applicative` is `pure`, which takes a regular
    /// value `T` and lifts it into the Kind context, producing `Self::Of<T>`
    /// (e.g., `pure(10)` for `OptionKind` yields `Some(10)`).
    ///
    /// ## Example of `pure`
    ///
    /// ```
    /// use monadify::applicative::kind::Applicative;
    /// use monadify::kind_based::kind::{OptionKind, VecKind};
    ///
    /// // For Option
    /// let val_opt: Option<i32> = OptionKind::pure(10);
    /// assert_eq!(val_opt, Some(10));
    ///
    /// // For Vec (requires T: Clone for pure)
    /// let val_vec: Vec<String> = VecKind::pure("hello".to_string());
    /// assert_eq!(val_vec, vec!["hello".to_string()]);
    /// ```
    ///
    /// ## Applicative Laws
    /// Implementors must satisfy several laws:
    /// 1.  **Identity**: `apply(v, pure(identity_fn)) == v`
    /// 2.  **Homomorphism**: `apply(pure(x), pure(f_fn)) == pure(f(x))`
    /// 3.  **Interchange**: `apply(pure(y), u) == apply(u, pure(|f_fn| f_fn(y)))`
    /// 4.  **Composition (derived)**: `map f x == apply(x, pure(f))` (often shown as `lift_a1`)
    pub trait Applicative<T>: Apply<T, T>
    where
        Self: Sized + Kind1,
        T: 'static,
    {
        /// Lifts a value into the applicative context.
        ///
        /// # Parameters
        /// - `value`: The value of type `T` to be lifted.
        ///
        /// # Returns
        /// The value wrapped in the Kind applicative structure, `Self::Of<T>`.
        #[must_use]
        fn pure(value: T) -> Self::Of<T>;
    }

    impl<T: 'static> Applicative<T> for OptionKind {
        /// Lifts a value `T` into `Some(T)`.
        fn pure(value: T) -> Self::Of<T> {
            Some(value)
        }
    }

    impl<T: 'static, E: 'static + Clone> Applicative<T> for ResultKind<E> {
        /// Lifts a value `T` into `Ok(T)`.
        fn pure(value: T) -> Self::Of<T> {
            Ok(value)
        }
    }

    impl<T: 'static + Clone> Applicative<T> for VecKind {
        /// Lifts a value `T` into `vec![T]`.
        ///
        /// The `T: Clone` bound on this `impl` block is due to `Vec`'s `pure`
        /// creating a new vector with the element.
        fn pure(value: T) -> Self::Of<T> {
            vec![value]
        }
    }

    // Applicative for RcFnKind<X> — generic Clone case.
    // Lifts a value `T: Clone` into `RcFn<X, T>` which always returns `value.clone()`.
    impl<X, T> Applicative<T> for RcFnKind<X>
    where
        X: 'static,
        T: 'static + Clone,
        Self: Apply<T, T>,
        Self: Kind<Of<T> = RcFn<X, T>>,
    {
        /// Lifts a value `T` into an `RcFn<X, T>` (a function `X -> T`).
        ///
        /// The resulting function, when called with any input of type `X`,
        /// ignores that input and always returns a clone of the original `value`.
        ///
        /// Requires `T: Clone` because the lifted value is cloned by the returned function.
        fn pure(value: T) -> Self::Of<T> {
            RcFn(Rc::new(move |_x: X| value.clone()))
        }
    }

    // Applicative for CFnOnceKind
    // Lifts a value `T` into `CFnOnce<X, T>`
    impl<X, T> Applicative<T> for CFnOnceKind<X>
    where
        X: 'static,
        T: 'static + Clone,
        Self: Apply<T, T>,
        Self: Kind<Of<T> = CFnOnce<X, T>>,
    {
        /// Lifts a value `T` into a `CFnOnce<X, T>` (a function `X -> T` called once).
        ///
        /// The resulting function, when called with any input of type `X`,
        /// will ignore that input and return a clone of the original `value`.
        ///
        /// Requires `T: Clone` as the lifted value is cloned by the returned function.
        fn pure(value: T) -> Self::Of<T> {
            CFnOnce::new(move |_x: X| value.clone())
        }
    }

    /// Lifts a unary function `A -> B` to operate on Kind `Applicative` values: `F::Of<A> -> F::Of<B>`.
    /// This is `map` defined via `pure` and `apply`: `map f fa == apply(fa, pure(RcFn::new(f)))`.
    ///
    /// # Parameters
    /// - `F`: The Kind marker, must implement `Applicative<RcFn<A,B>>` and `Apply<A,B>`.
    /// - `func`: The function `A -> B`.
    /// - `fa`: The applicative value `F::Of<A>`.
    ///
    /// # Returns
    /// The result `F::Of<B>`.
    ///
    /// ## Example
    ///
    /// ```
    /// use monadify::applicative::kind::lift_a1;
    /// use monadify::kind_based::kind::{OptionKind, VecKind};
    ///
    /// // Using lift_a1 with Option
    /// let opt_val: Option<i32> = Some(5);
    /// let lifted_opt: Option<String> = lift_a1::<OptionKind, _, _, _>(
    ///     |x: i32| (x * 2).to_string(),
    ///     opt_val
    /// );
    /// assert_eq!(lifted_opt, Some("10".to_string()));
    ///
    /// // Using lift_a1 with Vec — enabled via RcFn (which IS Clone).
    /// let vec_val: Vec<i32> = vec![1, 2, 3];
    /// let lifted_vec: Vec<bool> = lift_a1::<VecKind, _, _, _>(
    ///     |x: i32| x % 2 == 0,
    ///     vec_val
    /// );
    /// assert_eq!(lifted_vec, vec![false, true, false]);
    /// ```
    #[must_use]
    pub fn lift_a1<F, A, B, FuncImpl>(func: FuncImpl, fa: F::Of<A>) -> F::Of<B>
    where
        F: Applicative<RcFn<A, B>> + Apply<A, B> + Kind1,
        FuncImpl: Fn(A) -> B + 'static,
        A: 'static,
        B: 'static,
    {
        // 1. Lift the function `func: A -> B` into the context using `RcFn`.
        //    `F::pure(RcFn::new(func))` results in `F::Of<RcFn<A, B>>`.
        //    This requires `F` to be `Applicative` for the type `RcFn<A, B>`.
        let f_in_context: F::Of<RcFn<A, B>> = F::pure(RcFn::new(func));

        // 2. Apply the wrapped function to the wrapped value.
        F::apply(fa, f_in_context)
    }
}

// Directly export Kind-based Applicative and related functions
pub use kind::*;
