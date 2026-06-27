pub mod kind {
    // Renamed from hkt to kind
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
    //! use monadify::applicative::kind::{Applicative, lift_a1}; // Applicative for ::pure
    //! use monadify::apply::kind::Apply; // for .apply() method
    //! use monadify::kind_based::kind::OptionKind;
    //! use monadify::function::CFn;
    //!
    //! // Using pure and apply directly
    //! let val_opt: Option<i32> = OptionKind::pure(10); // Some(10)
    //! let fn_opt: Option<CFn<i32, i32>> = OptionKind::pure(CFn::new(|x| x + 1)); // Some(CFn)
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
    //! A common pattern, often called `lift_a1` or similar, is equivalent to
    //! `map` from `Functor` but implemented using `pure` and `apply`:
    //! `map f fa == apply(fa, pure(f))`. Note: The order of arguments in `apply` can vary;
    //! this `monadify` library's `apply` takes `(value_context, function_context)`.
    //! The `lift_a1` function in this module demonstrates this pattern.

    use crate::apply::kind::{Apply, ApplyRc}; // Kind-based Apply
    use crate::function::{CFn, CFnOnce, RcFn};
    use crate::kind_based::kind::{
        CFnKind, CFnOnceKind, Kind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind,
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
    ///     (Note: The exact formulation of composition can vary, often involving `apply` and `pure`.)
    pub trait Applicative<T>: Apply<T, T>
    where
        Self: Sized + Kind1, // Changed HKT1 to Kind1
        T: 'static,
    {
        /// Lifts a value into the applicative context.
        ///
        /// # Parameters
        /// - `value`: The value of type `T` to be lifted.
        ///   The `T: 'static` bound is common. Many `pure` implementations also require `T: Clone`
        ///   (e.g., for [`CFnKind`], [`VecKind`]) if the `value` needs to be cloned
        ///   into the new context, especially if the context itself might be "called" or
        ///   iterated multiple times. This can make some applicative laws involving `pure`
        ///   of non-`Clone` function types (like `CFn`) untestable.
        ///
        /// # Returns
        /// The value wrapped in the Kind applicative structure, `Self::Of<T>`.
        fn pure(value: T) -> Self::Of<T>; // Changed Applied to Of
    }

    impl<T: 'static> Applicative<T> for OptionKind {
        // Changed OptionHKTMarker to OptionKind
        /// Lifts a value `T` into `Some(T)`.
        fn pure(value: T) -> Self::Of<T> {
            // Self::Of<T> is Option<T>
            Some(value)
        }
    }

    impl<T: 'static, E: 'static + Clone> Applicative<T> for ResultKind<E> {
        // Changed ResultHKTMarker to ResultKind
        /// Lifts a value `T` into `Ok(T)`.
        fn pure(value: T) -> Self::Of<T> {
            // Self::Of<T> is Result<T, E>
            Ok(value)
        }
    }

    impl<T: 'static + Clone> Applicative<T> for VecKind {
        // Changed VecHKTMarker to VecKind
        /// Lifts a value `T` into `vec![T]`.
        ///
        /// The `T: Clone` bound on this `impl` block is due to `Vec`'s `pure`
        /// creating a new vector with the element.
        fn pure(value: T) -> Self::Of<T> {
            // Self::Of<T> is Vec<T>
            vec![value]
        }
    }

    // Applicative for CFnKind
    // Lifts a value `T` into `CFn<X, T>` which always returns `value.clone()`
    impl<X, T> Applicative<T> for CFnKind<X>
    // Changed CFnHKTMarker to CFnKind
    where
        X: 'static,
        T: 'static + Clone,            // T needs to be Clone for the closure
        Self: Apply<T, T>,             // Ensure Apply<T,T> for CFnKind<X> is defined
        Self: Kind<Of<T> = CFn<X, T>>, // Changed HKT to Kind, Applied to Of
    {
        /// Lifts a value `T` into a `CFn<X, T>` (a function `X -> T`).
        ///
        /// The resulting function, when called with any input of type `X`,
        /// will ignore that input and always return a clone of the original `value`.
        ///
        /// Requires `T: Clone` because the lifted value is cloned by the returned function.
        fn pure(value: T) -> Self::Of<T> {
            // Changed Applied to Of
            // Self::Of<T> is CFn<X, T> as per Kind1 impl for CFnKind
            CFn::new(move |_x: X| value.clone())
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

    // Applicative for RcFnKind<X> — specialized CFn case.
    //
    // `CFn<A,B>` is not `Clone` (it wraps `Box<dyn Fn>`), so the generic
    // `Applicative<T: Clone>` impl above does not cover it. This impl provides
    // `pure` for `CFn<A,B>` by converting the inner `Box` to `Rc` (enabling
    // O(1) cloning) and creating a new delegating `CFn` wrapper on each call.
    //
    // Coherence note: the two impls do NOT overlap because `CFn<A,B>` does not
    // implement `Clone` in this crate, and — due to the orphan rules — no
    // downstream crate can add `impl Clone for CFn<A,B>` either.
    impl<X, A, B> Applicative<CFn<A, B>> for RcFnKind<X>
    where
        X: 'static,
        A: 'static,
        B: 'static,
        Self: Apply<CFn<A, B>, CFn<A, B>>,
        Self: Kind<Of<CFn<A, B>> = RcFn<X, CFn<A, B>>>,
    {
        /// Lifts a `CFn<A,B>` into `RcFn<X, CFn<A,B>>`.
        ///
        /// Because `CFn` is not `Clone`, the inner `Box<dyn Fn(A)->B>` is first
        /// converted to `Rc<dyn Fn(A)->B>`. On each call to the resulting `RcFn`,
        /// a new `CFn` is created that delegates to the shared `Rc`, costing only
        /// one `Rc` reference-count bump per call.
        fn pure(value: CFn<A, B>) -> RcFn<X, CFn<A, B>> {
            // Convert Box → Rc so the underlying function can be shared cheaply.
            let shared: Rc<dyn Fn(A) -> B + 'static> = Rc::from(value.0);
            RcFn(Rc::new(move |_x: X| {
                // Clone the Rc (O(1)) and wrap in a new CFn that delegates.
                let rc_clone = Rc::clone(&shared);
                CFn(Box::new(move |a: A| (rc_clone.as_ref())(a)))
            }))
        }
    }

    // Applicative for CFnOnceKind
    // Lifts a value `T` into `CFnOnce<X, T>`
    impl<X, T> Applicative<T> for CFnOnceKind<X>
    // Changed CFnOnceHKTMarker to CFnOnceKind
    where
        X: 'static,
        T: 'static + Clone,
        Self: Apply<T, T>,
        Self: Kind<Of<T> = CFnOnce<X, T>>, // Changed HKT to Kind, Applied to Of
    {
        /// Lifts a value `T` into a `CFnOnce<X, T>` (a function `X -> T` called once).
        ///
        /// The resulting function, when called with any input of type `X`,
        /// will ignore that input and return a clone of the original `value`.
        ///
        /// Requires `T: Clone` as the lifted value is cloned by the returned function.
        fn pure(value: T) -> Self::Of<T> {
            // Changed Applied to Of
            // Self::Of<T> is CFnOnce<X, T> as per Kind1 impl for CFnOnceKind
            CFnOnce::new(move |_x: X| value.clone())
        }
    }

    /// Lifts a unary function `A -> B` to operate on Kind `Applicative` values: `F::Of<A> -> F::Of<B>`.
    /// This is `map` defined via `pure` and `apply`: `map f fa == apply(fa, pure(CFn::new(f)))`.
    ///
    /// # Parameters
    /// - `F`: The Kind marker, must implement `Applicative<CFn<A,B>>` and `Apply<A,B>`.
    /// - `func`: The function `A -> B`.
    /// - `fa`: The applicative value `F::Of<A>`.
    ///
    /// # Returns
    /// The result `F::Of<B>`.
    ///
    /// ## Example
    ///
    /// ```
    /// use monadify::applicative::kind::{lift_a1, lift_a1_rc};
    /// use monadify::kind_based::kind::{OptionKind, VecKind};
    ///
    /// // Using lift_a1 with Option
    /// let opt_val: Option<i32> = Some(5);
    /// // Provide the Kind marker via turbofish if type inference needs help
    /// let lifted_opt: Option<String> = lift_a1::<OptionKind, _, _, _>(
    ///     |x: i32| (x * 2).to_string(),
    ///     opt_val
    /// );
    /// assert_eq!(lifted_opt, Some("10".to_string()));
    ///
    /// // Using lift_a1_rc with Vec — re-enabled via RcFn (which IS Clone).
    /// // `lift_a1` over VecKind required `CFn<A,B>: Clone`, which `CFn` cannot satisfy.
    /// // `lift_a1_rc` uses `RcFn<A,B>` instead, which derives `Clone`.
    /// let vec_val: Vec<i32> = vec![1, 2, 3];
    /// let lifted_vec: Vec<bool> = lift_a1_rc::<VecKind, _, _, _>(
    ///     |x: i32| x % 2 == 0,
    ///     vec_val
    /// );
    /// assert_eq!(lifted_vec, vec![false, true, false]);
    /// ```
    pub fn lift_a1<F, A, B, FuncImpl>(
        func: FuncImpl,
        fa: F::Of<A>, // Changed Applied to Of
    ) -> F::Of<B>
    // Changed Applied to Of
    where
        F: Applicative<CFn<A, B>> + Apply<A, B> + Kind1, // Changed HKT1 to Kind1
        FuncImpl: Fn(A) -> B + 'static,
        A: 'static,
        B: 'static,
        CFn<A, B>: 'static, // Ensure the lifted function type is 'static
    {
        // 1. Lift the function `func: A -> B` into the context using `CFn`.
        //    `F::pure(CFn::new(func))` results in `F::Of<CFn<A, B>>`.
        //    This requires `F` to be `Applicative` for the type `CFn<A, B>`.
        let f_in_context: F::Of<CFn<A, B>> = F::pure(CFn::new(func)); // Changed Applied to Of

        // 2. Apply the wrapped function to the wrapped value.
        //    `F::apply(fa, f_in_context)` where `fa` is `F::Of<A>`.
        //    This requires `F` to be `Apply<A, B>`.
        F::apply(fa, f_in_context)
    }

    /// Lifts a unary function `A -> B` to operate on Kind `Applicative` values using
    /// [`RcFn`] as the function container.
    ///
    /// This is the `RcFn`-backed sibling of [`lift_a1`].  [`lift_a1`] builds
    /// `F::pure(CFn::new(func))`, requiring `CFn<A,B>: Clone` — a bound `CFn`
    /// cannot satisfy.  `lift_a1_rc` builds `F::pure(RcFn::new(func))` instead;
    /// [`RcFn`] derives `Clone`, so `VecKind::Applicative<RcFn<A,B>>` compiles.
    ///
    /// # Parameters
    /// - `F`: The Kind marker; must implement `Applicative<RcFn<A,B>>` and `ApplyRc<A,B>`.
    /// - `func`: The function `A -> B` to lift.
    /// - `fa`: The applicative value `F::Of<A>`.
    ///
    /// # Returns
    /// The result `F::Of<B>`.
    ///
    /// # Example
    ///
    /// ```
    /// use monadify::applicative::kind::lift_a1_rc;
    /// use monadify::kind_based::kind::VecKind;
    ///
    /// let fa: Vec<i32> = vec![1, 2, 3];
    /// let result: Vec<bool> = lift_a1_rc::<VecKind, _, _, _>(|x: i32| x % 2 == 0, fa);
    /// assert_eq!(result, vec![false, true, false]);
    /// ```
    pub fn lift_a1_rc<F, A, B, FuncImpl>(func: FuncImpl, fa: F::Of<A>) -> F::Of<B>
    where
        F: Applicative<RcFn<A, B>> + ApplyRc<A, B> + Kind1,
        FuncImpl: Fn(A) -> B + 'static,
        A: 'static,
        B: 'static,
    {
        // 1. Wrap the function in RcFn (Clone-able) and lift it into the context.
        let fc: F::Of<RcFn<A, B>> = F::pure(RcFn::new(func));
        // 2. Apply the wrapped function to the wrapped value via apply_rc.
        F::apply_rc(fa, fc)
    }
}

// Directly export Kind-based Applicative and related functions
pub use kind::*; // Renamed from hkt to kind
