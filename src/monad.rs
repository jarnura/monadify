pub mod kind {
    // Renamed from hkt to kind
    //! # Kind-based Monad and Bind for the `monadify` library
    //!
    //! This module defines the `Monad` and `Bind` traits for Kind-encoded types.
    //!
    //! - [`Bind`]: Provides the `bind` method (often called `flatMap` or `>>=`),
    //!   which allows sequencing operations that return a monadic value. It extends [`Apply`].
    //! - [`Monad`]: Extends [`Applicative`] (and thus `Bind` via `Applicative`'s supertrait `Apply`)
    //!   and adds the `join` method, which flattens a nested monadic structure (e.g., `F::Of<F::Of<A>>` to `F::Of<A>`).
    //!   Alternatively, a monad can be defined by `pure` (from `Applicative`) and `bind`.
    //!
    //! ## Example
    //!
    //! ```
    //! use monadify::monad::kind::{Monad, Bind};
    //! use monadify::applicative::kind::Applicative; // For pure
    //! use monadify::kind_based::kind::OptionKind;
    //!
    //! // Using bind
    //! let opt_val: Option<i32> = Some(5);
    //! let bind_fn = |x: i32| if x > 0 { Some(x * 2) } else { None };
    //! let result_bind: Option<i32> = OptionKind::bind(opt_val, bind_fn);
    //! assert_eq!(result_bind, Some(10));
    //!
    //! let opt_val_none: Option<i32> = Some(-5);
    //! let result_bind_none: Option<i32> = OptionKind::bind(opt_val_none, bind_fn);
    //! assert_eq!(result_bind_none, None);
    //!
    //! // Using join
    //! let nested_opt: Option<Option<String>> = Some(Some("hello".to_string()));
    //! let joined_opt: Option<String> = OptionKind::join(nested_opt);
    //! assert_eq!(joined_opt, Some("hello".to_string()));
    //!
    //! let nested_none_inner: Option<Option<String>> = Some(None);
    //! let joined_none_inner: Option<String> = OptionKind::join(nested_none_inner);
    //! assert_eq!(joined_none_inner, None);
    //!
    //! let nested_none_outer: Option<Option<String>> = None;
    //! let joined_none_outer: Option<String> = OptionKind::join(nested_none_outer);
    //! assert_eq!(joined_none_outer, None);
    //! ```

    use crate::applicative::kind::Applicative; // Kind-based Applicative
    use crate::apply::kind::Apply; // Kind-based Apply
    use crate::function::{CFnOnce, RcFn};
    use crate::kind_based::kind::{
        CFnOnceKind, Kind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind,
    };
    use std::rc::Rc;

    /// Kind-based `Monad` trait.
    ///
    /// A `Monad` allows for sequencing computations within a context. It extends
    /// [`Applicative`]. The key additional operation is `join`, which flattens
    /// nested monadic structures.
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`]) that implements
    /// [`Kind1`] and [`Applicative`].
    /// `A` is the type of the value held within the monadic context (e.g., the `T` in `Option<T>`).
    ///
    /// ## Monad Laws
    /// Implementors must satisfy several laws:
    /// 1.  **Left Identity**: `bind(pure(x), f) == f(x)`
    /// 2.  **Right Identity**: `bind(m, pure) == m`
    /// 3.  **Associativity**: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`
    ///
    /// These laws can also be expressed using `join`, `pure`, and `map` (from `Functor`,
    /// which `Applicative` extends):
    /// 1.  `join(pure(m)) == m`
    /// 2.  `join(map(m, pure)) == m`
    /// 3.  `join(map(mma, join)) == join(join(mma))` (if `map` is defined for `F::Of<F::Of<F::Of<A>>>`)
    ///     or more commonly: `join(map(mmma, |mma| join(mma))) == join(join(mmma))`
    pub trait Monad<A>: Applicative<A>
    // Monad holds type A
    where
        Self: Sized + Kind1, // Self is the Kind Marker
        A: 'static,
    {
        /// Flattens a nested monadic structure.
        ///
        /// For a Kind `F`, `join` takes `F::Of<F::Of<A>>` and returns `F::Of<A>`.
        ///
        /// # Example
        /// ```
        /// use monadify::monad::kind::Monad;
        /// use monadify::kind_based::kind::OptionKind;
        ///
        /// let nested: Option<Option<i32>> = Some(Some(10));
        /// let flat: Option<i32> = OptionKind::join(nested);
        /// assert_eq!(flat, Some(10));
        ///
        /// let nested_none: Option<Option<i32>> = Some(None);
        /// assert_eq!(OptionKind::join(nested_none), None);
        /// ```
        #[must_use]
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A>; // Changed Applied to Of
    }

    /// Kind-based `Bind` trait (equivalent to `flatMap` or Haskell's `>>=`).
    ///
    /// `Bind` allows sequencing operations where each operation takes a normal value
    /// and returns a value wrapped in the Kind context. It extends [`Apply`].
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`]).
    /// `A` is the type of the value within the input Kind context `Self::Of<A>`.
    /// `B` is the type of the value within the output Kind context `Self::Of<B>`
    /// that the provided function `func` returns.
    ///
    /// ## Example
    /// ```
    /// use monadify::monad::kind::Bind;
    /// use monadify::kind_based::kind::OptionKind;
    ///
    /// let opt_val: Option<i32> = Some(5);
    ///
    /// // Function that returns an Option
    /// let half = |x: i32| -> Option<f64> {
    ///     if x % 2 == 0 { Some((x as f64) / 2.0) } else { None }
    /// };
    ///
    /// let result: Option<f64> = OptionKind::bind(opt_val, half); // Fails as 5 is odd
    /// assert_eq!(result, None);
    ///
    /// let result_even: Option<f64> = OptionKind::bind(Some(10), half); // Succeeds
    /// assert_eq!(result_even, Some(5.0));
    /// ```
    pub trait Bind<A, B>: Apply<A, B>
    where
        Self: Sized + Kind1, // Changed HKT1 to Kind1
        A: 'static,
        B: 'static,
    {
        /// Sequentially composes computations within the Kind context.
        ///
        /// Takes a value in context (`Self::Of<A>`) and a function (`A -> Self::Of<B>`).
        /// It applies the function to the unwrapped value (if present/valid) and returns
        /// the resulting context `Self::Of<B>`.
        #[must_use]
        fn bind(
            input: Self::Of<A>,
            func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B>; // Changed Applied to Of
    }

    // --- Bind Implementations ---

    impl<A: 'static, B: 'static> Bind<A, B> for OptionKind {
        // Changed OptionHKTMarker to OptionKind
        /// For `Option`, `bind` is equivalent to `Option::and_then`.
        /// If `input` is `Some(a)`, it applies `func` to `a`.
        /// If `input` is `None`, it returns `None`.
        fn bind(
            input: Self::Of<A>,
            func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B> {
            // Changed Applied to Of
            input.and_then(func)
        }
    }

    impl<A: 'static, B: 'static, E: 'static + Clone> Bind<A, B> for ResultKind<E> {
        // Changed ResultHKTMarker to ResultKind
        /// For `Result`, `bind` is equivalent to `Result::and_then`.
        /// If `input` is `Ok(a)`, it applies `func` to `a`.
        /// If `input` is `Err(e)`, it propagates the `Err(e)`.
        fn bind(
            input: Self::Of<A>,
            func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B> {
            // Changed Applied to Of
            input.and_then(func)
        }
    }

    impl<A: 'static + Clone, B: 'static> Bind<A, B> for VecKind {
        // Changed VecHKTMarker to VecKind
        /// For `Vec`, `bind` applies `func` to each element and flattens the results.
        /// This is equivalent to `Vec::into_iter().flat_map(func).collect()`.
        fn bind(
            input: Self::Of<A>,
            func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B> {
            // Changed Applied to Of
            input.into_iter().flat_map(func).collect()
        }
    }

    // Bind for RcFnKind<R> (Kleisli composition for R -> _)
    // input: RcFn<R, A>; func: A -> RcFn<R, B>; result: RcFn<R, B>
    impl<R, A, B: 'static> Bind<A, B> for RcFnKind<R>
    where
        R: 'static + Clone,
        A: 'static,
        Self: Apply<A, B>,
        Self: Kind<Of<A> = RcFn<R, A>>,
        Self: Kind<Of<B> = RcFn<R, B>>,
    {
        /// Implements Kleisli composition for `RcFn<R, A>` and `A -> RcFn<R, B>`.
        ///
        /// The resulting `RcFn<R, B>`, when called with `r: R`:
        /// 1. Calls `input(r.clone())` to get `a: A`.
        /// 2. Calls `func(a)` to get `rcfn_r_b: RcFn<R, B>`.
        /// 3. Calls `rcfn_r_b(r)` to get `b: B`.
        fn bind(
            input: Self::Of<A>,
            func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B> {
            let inner = input.0.clone();
            RcFn(Rc::new(move |r: R| {
                let a_val = inner(r.clone());
                let mut func_clone = func.clone();
                let rcfn_r_b: RcFn<R, B> = func_clone(a_val);
                rcfn_r_b.call(r)
            }))
        }
    }

    impl<R, A, B: 'static> Bind<A, B> for CFnOnceKind<R>
    // Changed CFnOnceHKTMarker to CFnOnceKind
    where
        R: 'static + Clone,
        A: 'static,
        Self: Apply<A, B>,
        Self: Kind<Of<A> = CFnOnce<R, A>>, // Changed HKT to Kind, Applied to Of
        Self: Kind<Of<B> = CFnOnce<R, B>>, // Changed HKT to Kind, Applied to Of
    {
        /// Implements Kleisli composition for functions `R -> A` (once) and `A -> (R -> B)` (once).
        ///
        /// Implements Kleisli composition for `CFnOnce`: analogous to `RcFnKind::bind` but for single-use closures.
        /// The resulting function `R -> B` can also only be called once.
        fn bind(
            input: Self::Of<A>,
            mut func: impl FnMut(A) -> Self::Of<B> + Clone + 'static,
        ) -> Self::Of<B> {
            // Changed Applied to Of
            let concrete_input = input; // CFnOnce<R,A>
            CFnOnce::new(move |r: R| {
                let a_val = concrete_input.call_once(r.clone());
                let cfn_once_r_b = func(a_val); // CFnOnce<R,B>
                cfn_once_r_b.call_once(r)
            })
        }
    }

    // --- Monad Implementations ---

    impl<A: 'static> Monad<A> for OptionKind {
        // Changed OptionHKTMarker to OptionKind
        /// Flattens `Option<Option<A>>` to `Option<A>`.
        /// `Some(Some(a))` becomes `Some(a)`.
        /// `Some(None)` becomes `None`.
        /// `None` becomes `None`.
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A> {
            // mma is Option<Option<A>>. Changed Applied to Of
            mma.flatten()
        }
    }

    impl<A: 'static, E: 'static + Clone> Monad<A> for ResultKind<E> {
        // Changed ResultHKTMarker to ResultKind
        /// Flattens `Result<Result<A, E>, E>` to `Result<A, E>`.
        /// `Ok(Ok(a))` becomes `Ok(a)`.
        /// `Ok(Err(e))` becomes `Err(e)`.
        /// `Err(e)` becomes `Err(e)`.
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A> {
            // mma is Result<Result<A,E>, E>. Changed Applied to Of
            mma.and_then(core::convert::identity)
        }
    }

    impl<A: 'static + Clone> Monad<A> for VecKind {
        // Changed VecHKTMarker to VecKind
        /// Flattens `Vec<Vec<A>>` to `Vec<A>`.
        /// `vec![vec![1, 2], vec![3]]` becomes `vec![1, 2, 3]`.
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A> {
            // mma is Vec<Vec<A>>. Changed Applied to Of
            mma.into_iter().flatten().collect()
        }
    }

    impl<R, A> Monad<A> for RcFnKind<R>
    where
        R: 'static + Clone,
        A: 'static + Clone, // required by Applicative<A> supertrait for RcFnKind<R>
    {
        /// Flattens `RcFn<R, RcFn<R, A>>` to `RcFn<R, A>`.
        ///
        /// Implemented via `bind` with the identity function `|ma: RcFn<R,A>| ma`.
        /// Because `RcFn` is `Clone`, the inner type `A = RcFn<R, A>` satisfies
        /// the `Clone` constraint (unlike the old `Box`-backed approach which blocked this).
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A> {
            <Self as Bind<Self::Of<A>, A>>::bind(mma, |ma: Self::Of<A>| ma)
        }
    }

    impl<R, A> Monad<A> for CFnOnceKind<R>
    // Changed CFnOnceHKTMarker to CFnOnceKind
    where
        R: 'static + Clone,
        A: 'static + Clone, // From Applicative supertrait for CFnOnceKind<R>
    {
        /// Flattens `CFnOnce<R, CFnOnce<R, A>>` to `CFnOnce<R, A>`.
        ///
        /// Analogous to `RcFnKind::join`, but for single-use `CFnOnce` closures.
        fn join(mma: Self::Of<Self::Of<A>>) -> Self::Of<A> {
            // mma is CFnOnce<R, CFnOnce<R,A>>. Changed Applied to Of
            <Self as Bind<Self::Of<A>, A>>::bind(mma, |ma: Self::Of<A>| ma) // Changed Applied to Of
        }
    }

    /// Helper function for `Bind::bind`.
    ///
    /// This allows calling `bind(func, ma)` instead of `F::bind(ma, func)`.
    ///
    /// # Example
    /// ```
    /// use monadify::monad::kind::bind; // The helper function
    /// use monadify::kind_based::kind::OptionKind;
    ///
    /// let opt_val: Option<i32> = Some(5);
    /// let half = |x: i32| if x % 2 == 0 { Some((x as f64) / 2.0) } else { None };
    ///
    /// // Note: Type of F (OptionKind) might need to be inferred or specified
    /// let result: Option<f64> = bind::<OptionKind, _, _, _>(half, opt_val);
    /// assert_eq!(result, None);
    /// ```
    #[must_use]
    pub fn bind<F, A, B, FuncImpl>(
        func: FuncImpl,
        ma: F::Of<A>, // Changed Applied to Of
    ) -> F::Of<B>
    // Changed Applied to Of
    where
        F: Bind<A, B> + Kind1, // F is the KindMarker. Changed HKT1 to Kind1
        FuncImpl: FnMut(A) -> F::Of<B> + Clone + 'static, // Changed Applied to Of
        A: 'static,
        B: 'static, // B needs to be 'static for F::Of<B>
    {
        F::bind(ma, func)
    }
}

// Directly export Kind-based Bind, Monad, and helper bind
pub use kind::{bind, Bind, Monad}; // Renamed from hkt to kind
                                   // Note: join is a method on the Monad trait in the kind module.
