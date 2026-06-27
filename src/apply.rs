pub mod kind {
    // Renamed from hkt to kind
    //! # Kind-based Apply for the `monadify` library
    //!
    //! This module defines the `Apply` trait for Kind-encoded types, which extends `Functor`.
    //! `Apply` provides the `apply` method (often denoted as `<*>`), allowing sequential
    //! application of a Kind-wrapped function to a Kind-wrapped value.
    //!
    //! If you have `F::Of<A>` (a wrapped value) and `F::Of<A -> B>` (a wrapped function),
    //! `apply` combines them to produce `F::Of<B>`.
    //!
    //! The `Apply` trait is generic over:
    //! - `Self`: The Kind marker (e.g., [`OptionKind`]).
    //! - `A`: The input type of the function `A -> B` and the type of value in `Self::Of<A>`.
    //! - `B`: The output type of the function `A -> B` and the type of value in `Self::Of<B>`.

    use crate::function::{CFn, CFnOnce, RcFn};
    use crate::functor::Functor; // Kind-based Functor
    use crate::kind_based::kind::{
        CFnKind, CFnOnceKind, Kind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind,
    };
    use std::rc::Rc;

    /// Represents a Kind-encoded type that can apply a wrapped function to a wrapped value.
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`]) that implements
    /// [`Kind1`] and [`Functor`].
    ///
    /// The `apply` method takes `Self::Of<A>` (e.g., `Option<A>`) and
    /// `Self::Of<CFn<A, B>>` (e.g., `Option<CFn<A, B>>`), and produces
    /// `Self::Of<B>` (e.g., `Option<B>`).
    ///
    /// ## Apply Laws
    /// (Often defined in terms of `Applicative` which builds on `Apply`)
    /// A key law related to `apply` is compositional:
    /// `apply(apply(v, compose_pure_fn), pure_g_fn) == apply(v, apply(pure_f_fn, pure_compose_fn))`
    /// where `compose_pure_fn` is `pure(.)` or `pure(|f| |g| |x| f(g(x)))`.
    /// More commonly, laws are expressed with `Applicative`.
    pub trait Apply<A, B>: Functor<A, B>
    where
        Self: Sized + Kind1, // Changed HKT1 to Kind1
        A: 'static,
        B: 'static,
    {
        /// Applies a function wrapped in a Kind structure to a value wrapped in the same Kind structure.
        ///
        /// # Type Parameters
        /// - `Self`: The Kind marker.
        /// - `A`: The input type for the wrapped function `CFn<A, B>`.
        /// - `B`: The result type of the wrapped function and the output Kind structure.
        ///
        /// # Parameters
        /// - `value_container`: The Kind-structured value `Self::Of<A>`.
        /// - `function_container`: The Kind-structured function `Self::Of<CFn<A, B>>`.
        ///   Note: The function itself is wrapped in [`CFn`], which handles dynamic dispatch
        ///   and necessary `'static` bounds for the function it wraps.
        ///
        /// # Returns
        /// A new Kind-structured value `Self::Of<B>`.
        fn apply(
            value_container: Self::Of<A>,            // Changed Applied to Of
            function_container: Self::Of<CFn<A, B>>, // Changed Applied to Of
        ) -> Self::Of<B>; // Changed Applied to Of
    }

    impl<A: 'static, B: 'static> Apply<A, B> for OptionKind {
        // Changed OptionHKTMarker to OptionKind
        fn apply(
            value_container: Self::Of<A>,            // Changed Applied to Of
            function_container: Self::Of<CFn<A, B>>, // Changed Applied to Of
        ) -> Self::Of<B> {
            // Changed Applied to Of
            value_container.and_then(|val_a| function_container.map(|func_ab| func_ab.call(val_a)))
        }
    }

    impl<A: 'static, B: 'static, E: 'static + Clone> Apply<A, B> for ResultKind<E> {
        // Changed ResultHKTMarker to ResultKind
        fn apply(
            value_container: Self::Of<A>,            // Changed Applied to Of
            function_container: Self::Of<CFn<A, B>>, // Changed Applied to Of
        ) -> Self::Of<B> {
            // Changed Applied to Of
            value_container.and_then(|val_a| function_container.map(|func_ab| func_ab.call(val_a)))
        }
    }

    impl<A: 'static + Clone, B: 'static> Apply<A, B> for VecKind {
        // Changed VecHKTMarker to VecKind
        fn apply(
            value_container: Self::Of<A>,            // Changed Applied to Of
            function_container: Self::Of<CFn<A, B>>, // Changed Applied to Of
        ) -> Self::Of<B> {
            // Changed Applied to Of
            function_container
                .into_iter()
                .flat_map(|f_fn| {
                    value_container
                        .iter()
                        .map(move |val_a| f_fn.call(val_a.clone()))
                })
                .collect()
        }
    }

    // Apply for CFnKind<X>
    // F::Of<A> is CFn<X, A>
    // F::Of<CFn<A, B>> is CFn<X, CFn<A, B>>
    // Result is CFn<X, B>
    // This implements S f g x = (f x) (g x)
    impl<X, A, B> Apply<A, B> for CFnKind<X>
    // Changed CFnHKTMarker to CFnKind
    where
        X: 'static + Clone, // Clone for x_val in the closure
        A: 'static,
        B: 'static,
        Self: Functor<A, B>,           // Ensure Functor constraint is met
        Self: Kind<Of<A> = CFn<X, A>>, // HKT -> Kind, Applied -> Of
        Self: Kind<Of<CFn<A, B>> = CFn<X, CFn<A, B>>>, // HKT -> Kind, Applied -> Of
        Self: Kind<Of<B> = CFn<X, B>>, // HKT -> Kind, Applied -> Of
                                       // Removed: CFn<X, CFn<A, B>>: Fn(X) -> CFn<A, B>,
                                       // Removed: CFn<X, A>: Fn(X) -> A,
                                       // The .call method on CFn struct does not require CFn itself to be Fn.
                                       // A: 'static is from Apply trait. X: 'static + Clone for closure. B: 'static from Apply trait.
    {
        fn apply(
            value_container: Self::Of<A>,            // This is c_x_a. Applied -> Of
            function_container: Self::Of<CFn<A, B>>, // This is c_x_fab. Applied -> Of
        ) -> Self::Of<B> {
            // Applied -> Of
            // Self::Of<A> is CFn<X, A>
            // Self::Of<CFn<A, B>> is CFn<X, CFn<A, B>>
            CFn::new(move |x_val: X| {
                let func_ab = function_container.call(x_val.clone());
                let val_a = value_container.call(x_val);
                func_ab.call(val_a)
            })
        }
    }

    // Apply for CFnOnceKind<X>
    // Similar to CFnKind, but uses call_once and produces CFnOnce
    impl<X, A, B> Apply<A, B> for CFnOnceKind<X>
    // Changed CFnOnceHKTMarker to CFnOnceKind
    where
        X: 'static + Clone, // Clone for x_val in the closure
        A: 'static,
        B: 'static,
        Self: Functor<A, B>,
        Self: Kind<Of<A> = CFnOnce<X, A>>, // HKT -> Kind, Applied -> Of
        Self: Kind<Of<CFn<A, B>> = CFnOnce<X, CFn<A, B>>>, // HKT -> Kind, Applied -> Of
        Self: Kind<Of<B> = CFnOnce<X, B>>, // HKT -> Kind, Applied -> Of
                                           // Comments about FnOnce bounds and GATs remain relevant.
    {
        fn apply(
            value_container: Self::Of<A>,            // CFnOnce<X,A>. Applied -> Of
            function_container: Self::Of<CFn<A, B>>, // CFnOnce<X, CFn<A,B>>. Applied -> Of
        ) -> Self::Of<B> {
            // CFnOnce<X,B>. Applied -> Of
            CFnOnce::new(move |x_val: X| {
                // Self::Of<CFn<A,B>> is CFnOnce<X, CFn<A,B>>
                // Self::Of<A> is CFnOnce<X,A>
                let func_ab = function_container.call_once(x_val.clone()); // func_ab is CFn<A,B>
                let val_a = value_container.call_once(x_val); // val_a is A
                func_ab.call(val_a)
            })
        }
    }

    // Apply for RcFnKind<X>
    // The function container is RcFn<X, CFn<A,B>> (outer Rc-backed, inner CFn),
    // mirroring CFnKind where the container is CFn<X, CFn<A,B>>.
    impl<X, A, B> Apply<A, B> for RcFnKind<X>
    where
        X: 'static + Clone,
        A: 'static,
        B: 'static,
        Self: Functor<A, B>,
        Self: Kind<Of<A> = RcFn<X, A>>,
        Self: Kind<Of<CFn<A, B>> = RcFn<X, CFn<A, B>>>,
        Self: Kind<Of<B> = RcFn<X, B>>,
    {
        /// Applies an `RcFn<X, CFn<A,B>>` (a function from environment to a function
        /// `A -> B`) to an `RcFn<X, A>` (a function from environment to `A`),
        /// producing `RcFn<X, B>`.  This is the S combinator: `(f x)(g x)`.
        fn apply(
            value_container: Self::Of<A>,            // RcFn<X, A>
            function_container: Self::Of<CFn<A, B>>, // RcFn<X, CFn<A,B>>
        ) -> Self::Of<B> {
            let f_rc = function_container.0.clone();
            let v_rc = value_container.0.clone();
            RcFn(Rc::new(move |x_val: X| {
                let func_ab: CFn<A, B> = f_rc(x_val.clone());
                let val_a: A = v_rc(x_val);
                func_ab.call(val_a)
            }))
        }
    }

    /// A variant of [`Apply`] where the function container holds [`RcFn<A, B>`]
    /// instead of [`CFn<A, B>`].
    ///
    /// This is needed to implement [`crate::applicative::kind::lift_a1_rc`], which
    /// wraps a plain `A -> B` function in `RcFn` (not `CFn`) so that the resulting
    /// value satisfies `T: Clone` — a requirement of `VecKind`'s `Applicative<T>`.
    ///
    /// Implemented for [`OptionKind`], [`ResultKind<E>`], and [`VecKind`].
    pub trait ApplyRc<A, B>: Kind1 {
        /// Applies a Kind-wrapped `RcFn<A, B>` to a Kind-wrapped `A`, producing `B`.
        fn apply_rc(value: Self::Of<A>, func: Self::Of<RcFn<A, B>>) -> Self::Of<B>;
    }

    impl<A: 'static, B: 'static> ApplyRc<A, B> for OptionKind {
        /// Applies `Option<RcFn<A,B>>` to `Option<A>`.
        fn apply_rc(value: Self::Of<A>, func: Self::Of<RcFn<A, B>>) -> Self::Of<B> {
            value.and_then(|val_a| func.map(|f| f.call(val_a)))
        }
    }

    impl<A: 'static, B: 'static, E: 'static + Clone> ApplyRc<A, B> for ResultKind<E> {
        /// Applies `Result<RcFn<A,B>, E>` to `Result<A, E>`.
        fn apply_rc(value: Self::Of<A>, func: Self::Of<RcFn<A, B>>) -> Self::Of<B> {
            value.and_then(|val_a| func.map(|f| f.call(val_a)))
        }
    }

    impl<A: 'static + Clone, B: 'static> ApplyRc<A, B> for VecKind {
        /// Applies `Vec<RcFn<A,B>>` to `Vec<A>`, producing the Cartesian product.
        ///
        /// Each `RcFn` in `func` is applied to every element of `value`.  Because
        /// `RcFn` is `Clone`, iterating over `value` multiple times works without
        /// additional allocation.
        fn apply_rc(value: Self::Of<A>, func: Self::Of<RcFn<A, B>>) -> Self::Of<B> {
            func.into_iter()
                .flat_map(|f_fn| value.iter().map(move |val_a| f_fn.call(val_a.clone())))
                .collect()
        }
    }

    impl<X, A, B> ApplyRc<A, B> for RcFnKind<X>
    where
        X: 'static + Clone,
        A: 'static,
        B: 'static,
    {
        /// Applies `RcFn<X, RcFn<A,B>>` to `RcFn<X, A>`, producing `RcFn<X, B>`.
        ///
        /// This is the S-combinator (`apply_rc(v, f)(x) = f(x)(v(x))`) expressed
        /// through `RcFn` wrappers at every layer.  Because both the outer carrier
        /// and the inner function wrapper are `RcFn` (and therefore `Clone`), no
        /// `Box`-to-`Rc` conversion is needed — both `Rc` handles are cheaply shared.
        fn apply_rc(value: Self::Of<A>, func: Self::Of<RcFn<A, B>>) -> Self::Of<B> {
            let f_rc = func.0.clone();
            let v_rc = value.0.clone();
            RcFn(Rc::new(move |x_val: X| {
                let inner_fn: RcFn<A, B> = f_rc(x_val.clone());
                let val_a: A = v_rc(x_val);
                inner_fn.call(val_a)
            }))
        }
    }

    /// Lifts a binary curried function to operate on Kind-encoded contexts.
    ///
    /// Given `func: A -> (B -> C)` (represented as `A -> CFn<B, C>`),
    /// `fa: F::Of<A>`, and `fb: F::Of<B>`, `lift2` produces `F::Of<C>`.
    /// It's equivalent to `apply(map(fa, func), fb)` but expressed using the `Apply` trait's `apply` method.
    pub fn lift2<F, A, B, C, FuncImpl>(
        func: FuncImpl, // A -> CFn<B, C>
        fa: F::Of<A>,   // Changed Applied to Of
        fb: F::Of<B>,   // Changed Applied to Of
    ) -> F::Of<C>
    // Changed Applied to Of
    where
        F: Apply<B, C> + Functor<A, CFn<B, C>> + Kind1, // Changed HKT1 to Kind1
        FuncImpl: Fn(A) -> CFn<B, C> + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        let f_b_to_c_in_f = F::map(fa, func);
        F::apply(fb, f_b_to_c_in_f)
    }

    /// Lifts a ternary curried function to operate on Kind-encoded contexts.
    ///
    /// Given `func: A -> (B -> (C -> D))` (represented as `A -> CFn<B, CFn<C, D>>`),
    /// `fa: F::Of<A>`, `fb: F::Of<B>`, and `fc: F::Of<C>`, `lift3` produces `F::Of<D>`.
    pub fn lift3<F, A, B, C, D, FuncImpl>(
        func: FuncImpl, // A -> CFn<B, CFn<C, D>>
        fa: F::Of<A>,   // Changed Applied to Of
        fb: F::Of<B>,   // Changed Applied to Of
        fc: F::Of<C>,   // Changed Applied to Of
    ) -> F::Of<D>
    // Changed Applied to Of
    where
        F: Apply<C, D> + Apply<B, CFn<C, D>> + Functor<A, CFn<B, CFn<C, D>>> + Kind1, // Changed HKT1 to Kind1
        FuncImpl: Fn(A) -> CFn<B, CFn<C, D>> + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
    {
        let f_b_to_c_to_d_in_f = F::map(fa, func);
        let f_c_to_d_in_f = <F as Apply<B, CFn<C, D>>>::apply(fb, f_b_to_c_to_d_in_f);
        <F as Apply<C, D>>::apply(fc, f_c_to_d_in_f)
    }

    /// Combines two Kind-encoded actions, keeping only the result of the first.
    /// Often denoted as `<*`.
    pub fn apply_first<F, A, B>(
        fa: F::Of<A>, // Changed Applied to Of
        fb: F::Of<B>, // Changed Applied to Of
    ) -> F::Of<A>
    // Changed Applied to Of
    where
        F: Apply<B, A> + Functor<A, CFn<B, A>> + Kind1, // Changed HKT1 to Kind1
        A: Copy + 'static,
        B: 'static,
    {
        let map_fn = |x: A| CFn::new(move |_y: B| x);
        lift2::<F, A, B, A, _>(map_fn, fa, fb)
    }

    /// Combines two Kind-encoded actions, keeping only the result of the second.
    /// Often denoted as `*>`.
    pub fn apply_second<F, A, B>(
        fa: F::Of<A>, // Changed Applied to Of
        fb: F::Of<B>, // Changed Applied to Of
    ) -> F::Of<B>
    // Changed Applied to Of
    where
        F: Apply<B, B> + Functor<A, CFn<B, B>> + Kind1, // Changed HKT1 to Kind1
        A: 'static,
        B: Copy + 'static,
    {
        let map_fn = |_: A| CFn::new(|y: B| y);
        lift2::<F, A, B, B, _>(map_fn, fa, fb)
    }
}

// Directly export Kind-based Apply and related functions
pub use kind::*; // Renamed from hkt to kind
