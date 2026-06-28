pub mod kind {
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

    use crate::function::{CFnOnce, RcFn};
    use crate::functor::Functor;
    use crate::kind_based::kind::{
        CFnOnceKind, Kind, Kind1, OptionKind, RcFnKind, ResultKind, VecKind,
    };
    use std::rc::Rc;

    /// Represents a Kind-encoded type that can apply a wrapped function to a wrapped value.
    ///
    /// `Self` refers to the Kind marker type (e.g., [`OptionKind`]) that implements
    /// [`Kind1`] and [`Functor`].
    ///
    /// The `apply` method takes `Self::Of<A>` (e.g., `Option<A>`) and
    /// `Self::Of<RcFn<A, B>>` (e.g., `Option<RcFn<A, B>>`), and produces
    /// `Self::Of<B>` (e.g., `Option<B>`).
    ///
    /// ## Apply Laws
    /// A key law related to `apply` is compositional.
    pub trait Apply<A, B>: Functor<A, B>
    where
        Self: Sized + Kind1,
        A: 'static,
        B: 'static,
    {
        /// Applies a function wrapped in a Kind structure to a value wrapped in the same Kind structure.
        ///
        /// # Type Parameters
        /// - `Self`: The Kind marker.
        /// - `A`: The input type for the wrapped function `RcFn<A, B>`.
        /// - `B`: The result type of the wrapped function and the output Kind structure.
        ///
        /// # Parameters
        /// - `value_container`: The Kind-structured value `Self::Of<A>`.
        /// - `function_container`: The Kind-structured function `Self::Of<RcFn<A, B>>`.
        ///   The function is wrapped in [`RcFn`], which provides shared-ownership, Clone-able,
        ///   heap-allocated dispatch with `'static` bounds.
        ///
        /// # Returns
        /// A new Kind-structured value `Self::Of<B>`.
        #[must_use]
        fn apply(
            value_container: Self::Of<A>,
            function_container: Self::Of<RcFn<A, B>>,
        ) -> Self::Of<B>;
    }

    impl<A: 'static, B: 'static> Apply<A, B> for OptionKind {
        fn apply(
            value_container: Self::Of<A>,
            function_container: Self::Of<RcFn<A, B>>,
        ) -> Self::Of<B> {
            value_container.and_then(|val_a| function_container.map(|func_ab| func_ab.call(val_a)))
        }
    }

    impl<A: 'static, B: 'static, E: 'static + Clone> Apply<A, B> for ResultKind<E> {
        fn apply(
            value_container: Self::Of<A>,
            function_container: Self::Of<RcFn<A, B>>,
        ) -> Self::Of<B> {
            value_container.and_then(|val_a| function_container.map(|func_ab| func_ab.call(val_a)))
        }
    }

    impl<A: 'static + Clone, B: 'static> Apply<A, B> for VecKind {
        fn apply(
            value_container: Self::Of<A>,
            function_container: Self::Of<RcFn<A, B>>,
        ) -> Self::Of<B> {
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

    // Apply for CFnOnceKind<X>
    // F::Of<A> is CFnOnce<X, A>
    // F::Of<RcFn<A, B>> is CFnOnce<X, RcFn<A, B>>
    // Result is CFnOnce<X, B>
    impl<X, A, B> Apply<A, B> for CFnOnceKind<X>
    where
        X: 'static + Clone,
        A: 'static,
        B: 'static,
        Self: Functor<A, B>,
        Self: Kind<Of<A> = CFnOnce<X, A>>,
        Self: Kind<Of<RcFn<A, B>> = CFnOnce<X, RcFn<A, B>>>,
        Self: Kind<Of<B> = CFnOnce<X, B>>,
    {
        fn apply(
            value_container: Self::Of<A>,             // CFnOnce<X,A>
            function_container: Self::Of<RcFn<A, B>>, // CFnOnce<X, RcFn<A,B>>
        ) -> Self::Of<B> {
            // CFnOnce<X,B>
            CFnOnce::new(move |x_val: X| {
                let func_ab: RcFn<A, B> = function_container.call_once(x_val.clone());
                let val_a: A = value_container.call_once(x_val);
                func_ab.call(val_a)
            })
        }
    }

    // Apply for RcFnKind<X>
    // F::Of<A> is RcFn<X, A>
    // F::Of<RcFn<A, B>> is RcFn<X, RcFn<A, B>>
    // Result is RcFn<X, B>
    // This implements S f g x = (f x) (g x)
    impl<X, A, B> Apply<A, B> for RcFnKind<X>
    where
        X: 'static + Clone,
        A: 'static,
        B: 'static,
        Self: Functor<A, B>,
        Self: Kind<Of<A> = RcFn<X, A>>,
        Self: Kind<Of<RcFn<A, B>> = RcFn<X, RcFn<A, B>>>,
        Self: Kind<Of<B> = RcFn<X, B>>,
    {
        /// Applies an `RcFn<X, RcFn<A,B>>` (a function from environment to a function
        /// `A -> B`) to an `RcFn<X, A>` (a function from environment to `A`),
        /// producing `RcFn<X, B>`.  This is the S combinator: `(f x)(g x)`.
        fn apply(
            value_container: Self::Of<A>,             // RcFn<X, A>
            function_container: Self::Of<RcFn<A, B>>, // RcFn<X, RcFn<A,B>>
        ) -> Self::Of<B> {
            let f_rc = function_container.0.clone();
            let v_rc = value_container.0.clone();
            RcFn(Rc::new(move |x_val: X| {
                let func_ab: RcFn<A, B> = f_rc(x_val.clone());
                let val_a: A = v_rc(x_val);
                func_ab.call(val_a)
            }))
        }
    }

    /// Lifts a binary curried function to operate on Kind-encoded contexts.
    ///
    /// Given `func: A -> RcFn<B, C>`, `fa: F::Of<A>`, and `fb: F::Of<B>`,
    /// `lift2` produces `F::Of<C>`.
    #[must_use]
    pub fn lift2<F, A, B, C, FuncImpl>(
        func: FuncImpl, // A -> RcFn<B, C>
        fa: F::Of<A>,
        fb: F::Of<B>,
    ) -> F::Of<C>
    where
        F: Apply<B, C> + Functor<A, RcFn<B, C>> + Kind1,
        FuncImpl: Fn(A) -> RcFn<B, C> + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
    {
        let f_b_to_c_in_f = F::map(fa, func);
        F::apply(fb, f_b_to_c_in_f)
    }

    /// Lifts a ternary curried function to operate on Kind-encoded contexts.
    ///
    /// Given `func: A -> RcFn<B, RcFn<C, D>>`, `fa: F::Of<A>`, `fb: F::Of<B>`,
    /// and `fc: F::Of<C>`, `lift3` produces `F::Of<D>`.
    #[must_use]
    pub fn lift3<F, A, B, C, D, FuncImpl>(
        func: FuncImpl, // A -> RcFn<B, RcFn<C, D>>
        fa: F::Of<A>,
        fb: F::Of<B>,
        fc: F::Of<C>,
    ) -> F::Of<D>
    where
        F: Apply<C, D> + Apply<B, RcFn<C, D>> + Functor<A, RcFn<B, RcFn<C, D>>> + Kind1,
        FuncImpl: Fn(A) -> RcFn<B, RcFn<C, D>> + Clone + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
    {
        let f_b_to_c_to_d_in_f = F::map(fa, func);
        let f_c_to_d_in_f = <F as Apply<B, RcFn<C, D>>>::apply(fb, f_b_to_c_to_d_in_f);
        <F as Apply<C, D>>::apply(fc, f_c_to_d_in_f)
    }

    /// Combines two Kind-encoded actions, keeping only the result of the first.
    /// Often denoted as `<*`.
    #[must_use]
    pub fn apply_first<F, A, B>(fa: F::Of<A>, fb: F::Of<B>) -> F::Of<A>
    where
        F: Apply<B, A> + Functor<A, RcFn<B, A>> + Kind1,
        A: Copy + 'static,
        B: 'static,
    {
        let map_fn = |x: A| RcFn::new(move |_y: B| x);
        lift2::<F, A, B, A, _>(map_fn, fa, fb)
    }

    /// Combines two Kind-encoded actions, keeping only the result of the second.
    /// Often denoted as `*>`.
    #[must_use]
    pub fn apply_second<F, A, B>(fa: F::Of<A>, fb: F::Of<B>) -> F::Of<B>
    where
        F: Apply<B, B> + Functor<A, RcFn<B, B>> + Kind1,
        A: 'static,
        B: Copy + 'static,
    {
        let map_fn = |_: A| RcFn::new(|y: B| y);
        lift2::<F, A, B, B, _>(map_fn, fa, fb)
    }
}

// Directly export Kind-based Apply and related functions
pub use kind::*;
