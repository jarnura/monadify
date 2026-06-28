//! # ExceptT Monad Transformer for the `monadify` library
// Kind-based version is the default (mirrors `writer.rs`/`state.rs`).

pub mod kind {
    //! # Kind-based ExceptT Monad Transformer
    //!
    //! `ExceptT` (Except Transformer) augments an underlying monad (`MKind`, a
    //! Kind marker) with **short-circuiting error handling**. A computation
    //! `ExceptT<E, MKind, A>` is simply a wrapped inner value
    //! `MKind::Of<Result<A, E>>` — the produced value `A` or an error `E`.
    //!
    //! ## A hybrid of [`WriterT`](crate::transformers::writer) and [`StateT`](crate::transformers::state)
    //!
    //! `ExceptT` borrows its **carrier shape** from `WriterT`: the field
    //! [`run_except_t`](ExceptT::run_except_t) holds a *value*
    //! `MKind::Of<Result<A, E>>` directly (not an `Rc<dyn Fn>` like
    //! `ReaderT`/`StateT`), and [`Clone`] is hand-written, bounded on the
    //! projected inner type.
    //!
    //! But it belongs to the **`StateT` bound-family**: [`apply`](apply_kind::Apply::apply),
    //! [`bind`](monad_kind::Bind::bind), and [`join`](monad_kind::Monad::join)
    //! require the **inner** `MKind` to be a [`Bind`](monad_kind::Bind)/[`Monad`](monad_kind::Monad),
    //! not merely an [`Apply`](apply_kind::Apply). Short-circuiting on `Err` is a
    //! sequential data dependency — the result of the first computation must be
    //! inspected before the second is run or skipped — which only the inner
    //! monad's `bind` can express. (An inner-`Apply`-only `apply` that always ran
    //! the second computation would violate the Applicative–Monad consistency
    //! law; this matches Haskell's `instance (Monad m) => Applicative (ExceptT e m)`.)
    //!
    //! ## The lightest payload constraint of any transformer
    //!
    //! Unlike `WriterT` (whose log needs `W: Monoid`) and `StateT` (whose state
    //! needs `S: Clone`), `ExceptT` constrains its error type `E` only by
    //! `'static`. The error is propagated, never combined or duplicated. (A
    //! concrete `E: Clone` is required only *transitively*, where a concrete inner
    //! monad such as `Vec` makes `MKind::Of<Result<A, E>>: Clone` demand it.)
    //!
    //! The `Result<A, E>` order is fixed: `Result<value, error>` (Rust-native,
    //! value-biased like Haskell's `Right`). Transposing it silently breaks the
    //! monad laws.
    //!
    //! ## Key Components
    //! - [`ExceptT<E, MKind, A>`]: the computation, wrapping `MKind::Of<Result<A, E>>`.
    //! - [`ExceptTKind<E, MKind>`]: the Kind marker for `ExceptT`.
    //! - [`MonadError<E, A, MKind>`]: a trait providing `throw_error`,
    //!   `catch_error`, and `lift_either`.
    //! - [`Except<E, A>`]: alias for `ExceptT<E, IdentityKind, A>` (the plain Except monad).
    //!
    //! ## Example
    //! ```
    //! use monadify::transformers::except::{Except, ExceptT, ExceptTKind, MonadError};
    //! use monadify::IdentityKind;
    //! use monadify::Identity;
    //! use monadify::monad::kind::Bind;
    //!
    //! type Checked<A> = Except<String, A>;          // ExceptT<String, IdentityKind, A>
    //! type CheckedKind = ExceptTKind<String, IdentityKind>;
    //!
    //! // `Checked::throw` injects an error; the rest of the bind chain is skipped.
    //! let boom = |msg: &str| Checked::<i32>::throw(msg.to_string());
    //!
    //! let prog: Checked<i32> = CheckedKind::bind(boom("nope"), move |x| CheckedKind::bind(
    //!     Checked::ok(x + 1),
    //!     move |y| Checked::ok(y * 2),
    //! ));
    //! let Identity(res) = prog.run_except_t;
    //! assert_eq!(res, Err("nope".to_string())); // short-circuited
    //!
    //! // `.catch(..)` recovers from the error.
    //! let recovered: Checked<i32> = boom("bad").catch(|_e| Checked::ok(0));
    //! let Identity(res2) = recovered.run_except_t;
    //! assert_eq!(res2, Ok(0));
    //!
    //! // `from_result` embeds a pure `Result`.
    //! let from_ok: Checked<i32> = Checked::from_result(Ok(42));
    //! let Identity(res3) = from_ok.run_except_t;
    //! assert_eq!(res3, Ok(42));
    //!
    //! // For reference, the verbose trait form is still available for generic code:
    //! let verbose: Checked<i32> =
    //!     <CheckedKind as MonadError<String, i32, IdentityKind>>::lift_either(Ok(42));
    //! let Identity(res4) = verbose.run_except_t;
    //! assert_eq!(res4, Ok(42));
    //! ```

    use crate::applicative::kind as applicative_kind;
    use crate::apply::kind as apply_kind;
    use crate::function::RcFn; // Apply's function container (CFn removed in 0.2.0).
    use crate::functor::kind as functor_kind;
    use crate::identity::kind::IdentityKind;
    use crate::kind_based::kind::{Kind, Kind1};
    use crate::monad::kind as monad_kind;
    use std::marker::PhantomData;

    /// The `ExceptT` monad transformer for Kind-encoded types.
    ///
    /// `ExceptT<E, MKind, A>` wraps an inner value `MKind::Of<Result<A, E>>`: the
    /// produced value `A` or an error `E`, inside the inner monad. The value is
    /// stored in [`run_except_t`](Self::run_except_t).
    ///
    /// # Type Parameters
    /// - `E`: the error type (constrained only by `'static`).
    /// - `MKind`: the Kind marker for the inner monad (must implement [`Kind1`]).
    /// - `A`: the value produced on the success (`Ok`) branch.
    pub struct ExceptT<E, MKind: Kind1, A> {
        /// The wrapped inner computation: `MKind::Of<Result<A, E>>` (value-or-error).
        pub run_except_t: MKind::Of<Result<A, E>>,
        _phantom: PhantomData<(E, MKind, A)>,
    }

    // Manual `Clone` bounded on the projected inner type (mirrors `WriterT`): a
    // derived `Clone` would demand `E: Clone, MKind: Clone, A: Clone`, none of
    // which is the real requirement — only `MKind::Of<Result<A, E>>: Clone` is.
    impl<E, MKind: Kind1, A> Clone for ExceptT<E, MKind, A>
    where
        MKind::Of<Result<A, E>>: Clone,
    {
        fn clone(&self) -> Self {
            ExceptT {
                run_except_t: self.run_except_t.clone(),
                _phantom: PhantomData,
            }
        }
    }

    impl<E, MKind: Kind1, A> ExceptT<E, MKind, A> {
        /// Creates a new `ExceptT` from an inner value `MKind::Of<Result<A, E>>`.
        #[must_use]
        pub fn new(inner: MKind::Of<Result<A, E>>) -> Self {
            ExceptT {
                run_except_t: inner,
                _phantom: PhantomData,
            }
        }
    }

    /// The Kind marker for `ExceptT<E, MKind, _>`.
    ///
    /// Used to implement [`Functor`](functor_kind::Functor),
    /// [`Apply`](apply_kind::Apply), [`Applicative`](applicative_kind::Applicative),
    /// [`Bind`](monad_kind::Bind), and [`Monad`](monad_kind::Monad) for the
    /// `ExceptT` type constructor.
    ///
    /// # Type Parameters
    /// - `E`: the error type.
    /// - `MKind`: the Kind marker for the inner monad.
    #[derive(Default)]
    pub struct ExceptTKind<E, MKind: Kind1>(PhantomData<(E, MKind)>);

    impl<E, MKind: Kind1> Kind for ExceptTKind<E, MKind> {
        type Of<A> = ExceptT<E, MKind, A>;
    }
    // Kind1 is provided by the blanket impl in kind_based/kind.rs.

    /// A type alias for `ExceptT` with [`IdentityKind`] as the inner monad.
    /// This is the plain Except monad: `Except<E, A>` wraps `Identity<Result<A, E>>`.
    pub type Except<E, A> = ExceptT<E, IdentityKind, A>;

    // --- Kind Trait Implementations for ExceptTKind ---

    impl<E, MKind, A, B> functor_kind::Functor<A, B> for ExceptTKind<E, MKind>
    where
        E: 'static,
        MKind: functor_kind::Functor<Result<A, E>, Result<B, E>> + Kind1 + 'static,
        A: 'static,
        B: 'static,
        MKind::Of<Result<A, E>>: 'static,
        MKind::Of<Result<B, E>>: 'static,
    {
        /// Maps `A -> B` over the success branch, leaving any `Err` untouched.
        /// The mapping happens within the inner monad `MKind`.
        fn map(
            input: ExceptT<E, MKind, A>,
            func: impl FnMut(A) -> B + Clone + 'static,
        ) -> ExceptT<E, MKind, B> {
            let mut f = func;
            ExceptT::new(MKind::map(input.run_except_t, move |r: Result<A, E>| {
                r.map(&mut f)
            }))
        }
    }

    impl<E, MKind, A, B> apply_kind::Apply<A, B> for ExceptTKind<E, MKind>
    where
        E: 'static,
        A: 'static,
        B: 'static,
        // `Apply<A, B>: Functor<A, B>` (the inner `Functor<Result<A,E>,Result<B,E>>`
        // bound), plus the inner `Bind` that sequences the two computations to
        // short-circuit on `Err`, plus the inner `Applicative` for `pure(Err(e))`.
        MKind: functor_kind::Functor<Result<A, E>, Result<B, E>>
            + monad_kind::Bind<Result<RcFn<A, B>, E>, Result<B, E>>
            + applicative_kind::Applicative<Result<B, E>>
            + Kind1
            + 'static,
        MKind::Of<Result<A, E>>: Clone + 'static,
        MKind::Of<Result<B, E>>: 'static,
        MKind::Of<Result<RcFn<A, B>, E>>: 'static,
    {
        /// Applies a wrapped function to a wrapped value, **short-circuiting on
        /// `Err`**. Derived from the inner monad's `bind`: run the function
        /// computation; if it is `Err`, propagate it without touching the value
        /// computation; otherwise map the function over the value computation.
        fn apply(
            value_container: ExceptT<E, MKind, A>,
            function_container: ExceptT<E, MKind, RcFn<A, B>>,
        ) -> ExceptT<E, MKind, B> {
            let value_run = value_container.run_except_t;
            ExceptT::new(<MKind as monad_kind::Bind<
                Result<RcFn<A, B>, E>,
                Result<B, E>,
            >>::bind(
                function_container.run_except_t,
                move |rf: Result<RcFn<A, B>, E>| match rf {
                    Err(e) => MKind::pure(Err(e)),
                    Ok(g) => <MKind as functor_kind::Functor<Result<A, E>, Result<B, E>>>::map(
                        value_run.clone(),
                        move |ra: Result<A, E>| ra.map(|a| g.call(a)),
                    ),
                },
            ))
        }
    }

    impl<E, MKind, T> applicative_kind::Applicative<T> for ExceptTKind<E, MKind>
    where
        E: 'static,
        T: 'static, // `pure` consumes the value once — no `Clone` needed (like `WriterT`).
        // `Applicative<T>: Apply<T, T>`, so the full `Apply<T, T>` bound set is
        // dragged in here; `pure` itself needs only the inner `Applicative`.
        MKind: functor_kind::Functor<Result<T, E>, Result<T, E>>
            + monad_kind::Bind<Result<RcFn<T, T>, E>, Result<T, E>>
            + applicative_kind::Applicative<Result<T, E>>
            + Kind1
            + 'static,
        MKind::Of<Result<T, E>>: Clone + 'static,
        MKind::Of<Result<RcFn<T, T>, E>>: 'static,
    {
        /// Lifts a value `T` into `ExceptT` on the success branch:
        /// `MKind::pure(Ok(value))`.
        fn pure(value: T) -> ExceptT<E, MKind, T> {
            ExceptT::new(MKind::pure(Ok(value)))
        }
    }

    impl<E, MKind, A, B> monad_kind::Bind<A, B> for ExceptTKind<E, MKind>
    where
        E: 'static,
        A: 'static,
        B: 'static,
        // `Bind<A, B>: Apply<A, B>`, so the full `Apply<A, B>` bound set is dragged
        // in, plus the inner `Bind<Result<A,E>, Result<B,E>>` the `bind` body uses.
        MKind: functor_kind::Functor<Result<A, E>, Result<B, E>>
            + monad_kind::Bind<Result<A, E>, Result<B, E>>
            + monad_kind::Bind<Result<RcFn<A, B>, E>, Result<B, E>>
            + applicative_kind::Applicative<Result<B, E>>
            + Kind1
            + 'static,
        MKind::Of<Result<A, E>>: Clone + 'static,
        MKind::Of<Result<B, E>>: 'static,
        MKind::Of<Result<RcFn<A, B>, E>>: 'static,
    {
        /// Sequentially composes an `ExceptT` with a function producing the next
        /// `ExceptT`, **short-circuiting on `Err`**: if the first computation
        /// yields `Err(e)`, `func` is never called and `Err(e)` is propagated.
        fn bind(
            input: ExceptT<E, MKind, A>,
            func: impl FnMut(A) -> ExceptT<E, MKind, B> + Clone + 'static,
        ) -> ExceptT<E, MKind, B> {
            let mut f = func;
            ExceptT::new(
                <MKind as monad_kind::Bind<Result<A, E>, Result<B, E>>>::bind(
                    input.run_except_t,
                    move |ra: Result<A, E>| match ra {
                        Err(e) => MKind::pure(Err(e)),
                        Ok(a) => f(a).run_except_t,
                    },
                ),
            )
        }
    }

    impl<E, MKind, A> monad_kind::Monad<A> for ExceptTKind<E, MKind>
    where
        E: 'static,
        A: 'static,
        // `Monad<A>: Applicative<A>: Apply<A, A>`, so the full `Apply<A, A>` bound
        // set is dragged in, plus the inner `Bind` over `Result<ExceptT<…>, E>`.
        MKind: functor_kind::Functor<Result<A, E>, Result<A, E>>
            + monad_kind::Bind<Result<A, E>, Result<A, E>>
            + monad_kind::Bind<Result<RcFn<A, A>, E>, Result<A, E>>
            + monad_kind::Bind<Result<ExceptT<E, MKind, A>, E>, Result<A, E>>
            + applicative_kind::Applicative<Result<A, E>>
            + Kind1
            + 'static,
        MKind::Of<Result<A, E>>: Clone + 'static,
        MKind::Of<Result<RcFn<A, A>, E>>: 'static,
        MKind::Of<Result<ExceptT<E, MKind, A>, E>>: 'static,
    {
        /// Flattens a nested `ExceptT<E, MKind, ExceptT<E, MKind, A>>`: run the
        /// outer computation; if it is `Err`, propagate it, otherwise run the
        /// inner computation it produced.
        fn join(mma: ExceptT<E, MKind, ExceptT<E, MKind, A>>) -> ExceptT<E, MKind, A> {
            ExceptT::new(<MKind as monad_kind::Bind<
                Result<ExceptT<E, MKind, A>, E>,
                Result<A, E>,
            >>::bind(
                mma.run_except_t,
                move |r: Result<ExceptT<E, MKind, A>, E>| match r {
                    Err(e) => MKind::pure(Err(e)),
                    Ok(inner) => inner.run_except_t,
                },
            ))
        }
    }

    // --- Error-channel map and ergonomic inherent constructors/combinators ---

    impl<E, MKind: Kind1, A> ExceptT<E, MKind, A> {
        /// Maps the error channel `E -> E2`, leaving the success value unchanged:
        /// `Err(e) -> Err(f(e))`, `Ok(a) -> Ok(a)` (inner `Functor`).
        ///
        /// The Rust analog of Haskell's `withExceptT`.
        #[must_use]
        pub fn with_except_t<E2, F>(self, f: F) -> ExceptT<E2, MKind, A>
        where
            E: 'static,
            E2: 'static,
            A: 'static,
            MKind: functor_kind::Functor<Result<A, E>, Result<A, E2>> + 'static,
            MKind::Of<Result<A, E>>: 'static,
            MKind::Of<Result<A, E2>>: 'static,
            F: FnMut(E) -> E2 + Clone + 'static,
        {
            let mut f = f;
            ExceptT::new(MKind::map(self.run_except_t, move |r: Result<A, E>| {
                r.map_err(&mut f)
            }))
        }

        /// Lifts a success value onto the `Ok` branch — the ergonomic concrete form of
        /// `MonadError::lift_either(Ok(_))` (and of `Applicative::pure`). The generic
        /// `MonadError` trait remains for code generic over the inner monad.
        #[must_use]
        pub fn ok(value: A) -> Self
        where
            E: 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKind::Of<Result<A, E>>: 'static,
        {
            ExceptT::new(MKind::pure(Ok(value)))
        }

        /// Short-circuits with an error — the ergonomic concrete form of `MonadError::throw_error`.
        #[must_use]
        pub fn throw(error: E) -> Self
        where
            E: 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKind::Of<Result<A, E>>: 'static,
        {
            ExceptT::new(MKind::pure(Err(error)))
        }

        /// Embeds a pure `Result<A, E>` — the ergonomic concrete form of `MonadError::lift_either`.
        #[must_use]
        pub fn from_result(r: Result<A, E>) -> Self
        where
            E: 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKind::Of<Result<A, E>>: 'static,
        {
            ExceptT::new(MKind::pure(r))
        }

        /// Chainable recovery: runs `self`, and on `Err(e)` runs `handler(e)` instead — the
        /// method form of `MonadError::catch_error`.
        #[must_use]
        pub fn catch<F>(self, handler: F) -> Self
        where
            E: 'static,
            A: 'static,
            MKind: monad_kind::Bind<Result<A, E>, Result<A, E>>
                + applicative_kind::Applicative<Result<A, E>>
                + 'static,
            MKind::Of<Result<A, E>>: 'static,
            F: Fn(E) -> ExceptT<E, MKind, A> + Clone + 'static,
        {
            ExceptT::new(
                <MKind as monad_kind::Bind<Result<A, E>, Result<A, E>>>::bind(
                    self.run_except_t,
                    move |r: Result<A, E>| match r {
                        Ok(a) => MKind::pure(Ok(a)),
                        Err(e) => handler(e).run_except_t,
                    },
                ),
            )
        }
    }

    /// Trait for monads that support throwing and catching errors of type `E`.
    ///
    /// The Except analog of [`MonadReader`](crate::transformers::reader::MonadReader)
    /// (`ask`/`local`), [`MonadState`](crate::transformers::state::MonadState)
    /// (`get`/`put`/…), and [`MonadWriter`](crate::transformers::writer::MonadWriter)
    /// (`tell`/…). The primitive is [`throw_error`](Self::throw_error);
    /// [`catch_error`](Self::catch_error) recovers from a thrown error, and
    /// [`lift_either`](Self::lift_either) embeds a pure `Result`.
    ///
    /// `throw_error`/`lift_either` need only the inner [`Applicative`](applicative_kind::Applicative);
    /// `catch_error` needs the inner [`Bind`](monad_kind::Bind) (it must inspect
    /// the `Result` to decide whether to run the handler).
    ///
    /// # Type Parameters
    /// - `E`: the error type.
    /// - `A`: the value type carried by the computation.
    /// - `MKind`: the Kind marker for the inner monad.
    pub trait MonadError<E, A, MKind: Kind1>
    where
        Self: Sized,
    {
        /// Injects an error, short-circuiting: `MKind::pure(Err(e))`. The primitive.
        #[must_use]
        fn throw_error(e: E) -> ExceptT<E, MKind, A>
        where
            E: 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKind::Of<Result<A, E>>: 'static;

        /// Runs `m`; if it produced `Err(e)`, runs `handler(e)` instead;
        /// otherwise passes the `Ok` value through unchanged.
        #[must_use]
        fn catch_error<F>(m: ExceptT<E, MKind, A>, handler: F) -> ExceptT<E, MKind, A>
        where
            E: 'static,
            A: 'static,
            MKind: monad_kind::Bind<Result<A, E>, Result<A, E>>
                + applicative_kind::Applicative<Result<A, E>>
                + 'static,
            MKind::Of<Result<A, E>>: 'static,
            F: Fn(E) -> ExceptT<E, MKind, A> + Clone + 'static;

        /// Embeds a pure `Result<A, E>` directly: `MKind::pure(r)`.
        #[must_use]
        fn lift_either(r: Result<A, E>) -> ExceptT<E, MKind, A>
        where
            E: 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKind::Of<Result<A, E>>: 'static;
    }

    impl<E, MKindImpl, A> MonadError<E, A, MKindImpl> for ExceptTKind<E, MKindImpl>
    where
        E: 'static,
        A: 'static,
        MKindImpl: Kind1 + 'static,
    {
        fn throw_error(e: E) -> ExceptT<E, MKindImpl, A>
        where
            E: 'static,
            A: 'static,
            MKindImpl: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKindImpl::Of<Result<A, E>>: 'static,
        {
            ExceptT::new(MKindImpl::pure(Err(e)))
        }

        fn catch_error<F>(m: ExceptT<E, MKindImpl, A>, handler: F) -> ExceptT<E, MKindImpl, A>
        where
            E: 'static,
            A: 'static,
            MKindImpl: monad_kind::Bind<Result<A, E>, Result<A, E>>
                + applicative_kind::Applicative<Result<A, E>>
                + 'static,
            MKindImpl::Of<Result<A, E>>: 'static,
            F: Fn(E) -> ExceptT<E, MKindImpl, A> + Clone + 'static,
        {
            ExceptT::new(<MKindImpl as monad_kind::Bind<
                Result<A, E>,
                Result<A, E>,
            >>::bind(
                m.run_except_t,
                move |r: Result<A, E>| match r {
                    Ok(a) => MKindImpl::pure(Ok(a)),
                    Err(e) => handler(e).run_except_t,
                },
            ))
        }

        fn lift_either(r: Result<A, E>) -> ExceptT<E, MKindImpl, A>
        where
            E: 'static,
            A: 'static,
            MKindImpl: applicative_kind::Applicative<Result<A, E>> + 'static,
            MKindImpl::Of<Result<A, E>>: 'static,
        {
            ExceptT::new(MKindImpl::pure(r))
        }
    }
}

// Directly export Kind-based versions
pub use kind::{Except, ExceptT, ExceptTKind, MonadError};
