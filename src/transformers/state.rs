//! # StateT Monad Transformer for the `monadify` library
// Kind-based version is the default (mirrors `reader.rs`).

pub mod kind {
    //! # Kind-based StateT Monad Transformer
    //!
    //! This module provides the Kind-based implementation of the `StateT` monad
    //! transformer. `StateT` (State Transformer) threads a **mutable state** of
    //! type `S` through an underlying monad (`MKind`, a Kind marker).
    //!
    //! A computation `StateT<S, MKind, A>` is a function
    //! `S -> MKind::Of<(A, S)>`: given a starting state it produces, inside the
    //! inner monad, a pair `(value, new_state)`.
    //!
    //! ## Relationship to [`ReaderT`](crate::transformers::reader)
    //!
    //! `StateT` mirrors `ReaderT`'s structure, but **threads** state rather than
    //! broadcasting a read-only environment. This causes four deliberate
    //! divergences from `ReaderT`:
    //!
    //! 1. **Inner `Bind`/`Monad`, not just `Apply`.** [`apply`](apply_kind::Apply::apply),
    //!    [`bind`](monad_kind::Bind::bind), and [`join`](monad_kind::Monad::join)
    //!    require the **inner** `MKind` to be `Bind`/`Monad`, because step *n+1*
    //!    depends on the state produced by step *n* — a genuine sequential data
    //!    dependency. `ReaderT::apply` needs only an inner `Apply` (the
    //!    environment is broadcast read-only).
    //! 2. **Every inner bound is over the pair `(A, S)`**, never bare `A`.
    //! 3. [`apply`](apply_kind::Apply::apply) additionally requires `A: Clone`
    //!    (the argument value is produced inside the inner monad and must be
    //!    duplicated to feed the function).
    //! 4. **Three runners** — [`run_state_t`](StateT::run_state_t) (the raw
    //!    field), [`eval_state_t`](StateT::eval_state_t) (value only), and
    //!    [`exec_state_t`](StateT::exec_state_t) (final state only).
    //!
    //! The tuple order is `(A, S) = (value, new_state)`; it is fixed and never
    //! reordered (transposing silently breaks the monad and `MonadState` laws).
    //!
    //! ## Key Components
    //! - [`StateT<S, MKind, A>`]: the computation `S -> MKind::Of<(A, S)>`.
    //! - [`StateTKind<S, MKind>`]: the Kind marker for `StateT`.
    //! - [`MonadState<S, A, MKind>`]: a trait providing `state`, `get`, `put`,
    //!   `modify`, and `gets`.
    //! - [`State<S, A>`]: alias for `StateT<S, IdentityKind, A>` (the plain State monad).
    //!
    //! ## Example
    //! ```
    //! use monadify::transformers::state::{State, StateT, StateTKind};
    //! use monadify::IdentityKind;
    //! use monadify::Identity;
    //! use monadify::applicative::kind::Applicative;
    //! use monadify::monad::kind::Bind;
    //!
    //! type Counter<A> = State<i32, A>;            // StateT<i32, IdentityKind, A>
    //! type CounterKind = StateTKind<i32, IdentityKind>;
    //!
    //! // `state`: a pure state transition. Return the old value, increment the state.
    //! let tick: Counter<i32> =
    //!     CounterKind::state(|s| (s, s + 1));
    //! let Identity((v, s)) = (tick.run_state_t)(10);
    //! assert_eq!((v, s), (10, 11));
    //!
    //! // `get` then `put`: read the state, store `x + 5`, return the original `x`.
    //! let prog: Counter<i32> = CounterKind::bind(
    //!     CounterKind::get(),
    //!     |x| {
    //!         CounterKind::bind(
    //!             CounterKind::put(x + 5),
    //!             move |_| CounterKind::pure(x),
    //!         )
    //!     },
    //! );
    //! let Identity((v, s)) = (prog.run_state_t)(1);
    //! assert_eq!((v, s), (1, 6));
    //!
    //! // Runners: project just the value or just the final state.
    //! let again: Counter<i32> =
    //!     CounterKind::state(|s| (s * 2, s + 100));
    //! assert_eq!(again.clone().eval_state_t(3), Identity(6));
    //! assert_eq!(again.exec_state_t(3), Identity(103));
    //! ```

    use crate::applicative::kind as applicative_kind;
    use crate::apply::kind as apply_kind;
    use crate::function::RcFn; // Apply's function container (CFn removed in 0.2.0).
    use crate::functor::kind as functor_kind;
    use crate::identity::kind::IdentityKind;
    use crate::kind_based::kind::{Kind, Kind1};
    use crate::monad::kind as monad_kind;
    use std::marker::PhantomData;
    use std::rc::Rc;

    /// The `StateT` monad transformer for Kind-encoded types.
    ///
    /// `StateT<S, MKind, A>` is a computation that takes a state `S` and yields,
    /// inside the inner monad `MKind`, a pair `(A, S)` — the produced value and
    /// the new state. The computation is stored in [`run_state_t`](Self::run_state_t),
    /// a function `S -> MKind::Of<(A, S)>`.
    ///
    /// # Type Parameters
    /// - `S`: the threaded state type.
    /// - `MKind`: the Kind marker for the inner monad (must implement [`Kind1`]).
    /// - `A`: the value produced alongside the new state.
    #[derive(Clone)]
    pub struct StateT<S, MKind: Kind1, A> {
        /// The core computation: `S -> MKind::Of<(A, S)>` (value-then-new-state).
        // The boxed-`Fn` carrier is inherent to the encoding (mirrors ReaderT).
        #[allow(clippy::type_complexity)]
        pub run_state_t: Rc<dyn Fn(S) -> MKind::Of<(A, S)> + 'static>,
        _phantom: PhantomData<(S, MKind, A)>,
    }

    impl<S, MKind: Kind1, A> StateT<S, MKind, A> {
        /// Creates a new `StateT` from a function `S -> MKind::Of<(A, S)>`.
        #[must_use]
        pub fn new<F>(f: F) -> Self
        where
            F: Fn(S) -> MKind::Of<(A, S)> + 'static,
        {
            StateT {
                run_state_t: Rc::new(f),
                _phantom: PhantomData,
            }
        }
    }

    /// The Kind marker for `StateT<S, MKind, _>`.
    ///
    /// Used to implement [`Functor`](functor_kind::Functor),
    /// [`Apply`](apply_kind::Apply), [`Applicative`](applicative_kind::Applicative),
    /// [`Bind`](monad_kind::Bind), and [`Monad`](monad_kind::Monad) for the
    /// `StateT` type constructor.
    ///
    /// # Type Parameters
    /// - `S`: the state type.
    /// - `MKind`: the Kind marker for the inner monad.
    #[derive(Default)]
    pub struct StateTKind<S, MKind: Kind1>(PhantomData<(S, MKind)>);

    impl<S, MKind: Kind1> Kind for StateTKind<S, MKind> {
        type Of<A> = StateT<S, MKind, A>;
    }
    // Kind1 is provided by the blanket impl in kind_based/kind.rs.

    /// A type alias for `StateT` with [`IdentityKind`] as the inner monad.
    /// This is the plain State monad: `State<S, A>` is a computation
    /// `S -> Identity<(A, S)>`.
    pub type State<S, A> = StateT<S, IdentityKind, A>;

    // --- Kind Trait Implementations for StateTKind ---

    impl<S, MKind, A, B> functor_kind::Functor<A, B> for StateTKind<S, MKind>
    where
        S: Clone + 'static,
        MKind: functor_kind::Functor<(A, S), (B, S)> + Kind1 + 'static,
        A: 'static,
        B: 'static,
        MKind::Of<(A, S)>: 'static,
        MKind::Of<(B, S)>: 'static,
    {
        /// Maps `A -> B` over the produced value, threading the state unchanged.
        /// The mapping happens within the inner monad `MKind`.
        fn map(
            input: StateT<S, MKind, A>,
            func: impl FnMut(A) -> B + Clone + 'static,
        ) -> StateT<S, MKind, B> {
            let run = input.run_state_t.clone();
            StateT::new(move |s: S| {
                let m_pair = run(s); // MKind::Of<(A, S)>
                let mut f = func.clone();
                MKind::map(m_pair, move |(a, s2)| (f(a), s2))
            })
        }
    }

    impl<S, MKind, T> applicative_kind::Applicative<T> for StateTKind<S, MKind>
    where
        S: Clone + 'static,
        T: Clone + 'static,
        // `Applicative<T>: Apply<T, T>`, so the full `Apply<T, T>` bound set is
        // dragged in here (auto-satisfied for every concrete inner monad).
        MKind: functor_kind::Functor<(T, S), (T, S)>
            + monad_kind::Bind<(T, S), (T, S)>
            + monad_kind::Bind<(RcFn<T, T>, S), (T, S)>
            + applicative_kind::Applicative<(T, S)>
            + Kind1
            + 'static,
        MKind::Of<(T, S)>: 'static,
        MKind::Of<(RcFn<T, T>, S)>: 'static,
    {
        /// Lifts a value `T` into `StateT`, threading the incoming state
        /// unchanged: `s -> MKind::pure((value, s))`.
        fn pure(value: T) -> StateT<S, MKind, T> {
            StateT::new(move |s: S| MKind::pure((value.clone(), s)))
        }
    }

    impl<S, MKind, A, B> apply_kind::Apply<A, B> for StateTKind<S, MKind>
    where
        S: Clone + 'static,
        A: Clone + 'static, // DIVERGENCE 3: value is duplicated to feed the function.
        B: 'static,
        // `Apply<A, B>: Functor<A, B>` (the Functor bound), plus the inner-`Bind`
        // bounds the state-threaded `apply` body needs.
        MKind: functor_kind::Functor<(A, S), (B, S)>
            + monad_kind::Bind<(A, S), (B, S)>
            + monad_kind::Bind<(RcFn<A, B>, S), (B, S)>
            + applicative_kind::Applicative<(B, S)>
            + Kind1
            + 'static,
        MKind::Of<(A, S)>: 'static,
        MKind::Of<(B, S)>: 'static,
        MKind::Of<(RcFn<A, B>, S)>: 'static,
    {
        /// Applies a wrapped function to a wrapped value, threading state
        /// sequentially: run the value computation (`s -> s1`), then the
        /// function computation (`s1 -> s2`), then `pure` the applied result.
        ///
        /// Because state must be threaded, `apply` is derived from the inner
        /// monad's `bind`/`pure` (not its `apply`) — hence the inner `Bind`
        /// bounds. The function container is [`RcFn`].
        fn apply(
            value_container: StateT<S, MKind, A>,
            function_container: StateT<S, MKind, RcFn<A, B>>,
        ) -> StateT<S, MKind, B> {
            let value_run = value_container.run_state_t.clone();
            let func_run = function_container.run_state_t.clone();
            StateT::new(move |s: S| {
                let func_run = func_run.clone();
                // 1. run the value computation: s -> (a, s1)
                <MKind as monad_kind::Bind<(A, S), (B, S)>>::bind(value_run(s), move |(a, s1)| {
                    let func_run = func_run.clone();
                    // 2. run the function computation from s1: s1 -> (g, s2)
                    <MKind as monad_kind::Bind<(RcFn<A, B>, S), (B, S)>>::bind(
                        func_run(s1),
                        move |(g, s2)| MKind::pure((g.call(a.clone()), s2)), // 3. apply, thread s2
                    )
                })
            })
        }
    }

    impl<S, MKind, A, B> monad_kind::Bind<A, B> for StateTKind<S, MKind>
    where
        S: Clone + 'static,
        A: Clone + 'static, // inherited via the `Bind<A, B>: Apply<A, B>` supertrait.
        B: 'static,
        // `Bind<A, B>: Apply<A, B>`, so the full `Apply<A, B>` bound set is dragged
        // in here on top of the `Bind<(A, S), (B, S)>` the `bind` body uses.
        MKind: functor_kind::Functor<(A, S), (B, S)>
            + monad_kind::Bind<(A, S), (B, S)>
            + monad_kind::Bind<(RcFn<A, B>, S), (B, S)>
            + applicative_kind::Applicative<(B, S)>
            + Kind1
            + 'static,
        MKind::Of<(A, S)>: 'static,
        MKind::Of<(B, S)>: 'static,
        MKind::Of<(RcFn<A, B>, S)>: 'static,
    {
        /// Sequentially composes a `StateT` with a function producing the next
        /// `StateT`, **threading the state**: the first computation's output
        /// state `s1` is fed to the computation produced by `func`.
        fn bind(
            input: StateT<S, MKind, A>,
            func: impl FnMut(A) -> StateT<S, MKind, B> + Clone + 'static,
        ) -> StateT<S, MKind, B> {
            let run = input.run_state_t.clone();
            StateT::new(move |s: S| {
                let mut f = func.clone();
                MKind::bind(run(s), move |(a, s1)| {
                    let next: StateT<S, MKind, B> = f(a);
                    (next.run_state_t)(s1)
                })
            })
        }
    }

    impl<S, MKind, A> monad_kind::Monad<A> for StateTKind<S, MKind>
    where
        S: Clone + 'static,
        A: Clone + 'static,
        // `Monad<A>: Applicative<A>: Apply<A, A>`, so the full `Apply<A, A>` bound
        // set is dragged in, plus the `Bind<(StateT<…>, S), (A, S)>` `join` uses.
        MKind: functor_kind::Functor<(A, S), (A, S)>
            + monad_kind::Bind<(A, S), (A, S)>
            + monad_kind::Bind<(RcFn<A, A>, S), (A, S)>
            + applicative_kind::Applicative<(A, S)>
            + monad_kind::Bind<(StateT<S, MKind, A>, S), (A, S)>
            + Kind1
            + 'static,
        MKind::Of<(A, S)>: 'static,
        MKind::Of<(RcFn<A, A>, S)>: 'static,
        MKind::Of<(StateT<S, MKind, A>, S)>: 'static,
    {
        /// Flattens a nested `StateT<S, MKind, StateT<S, MKind, A>>`: run the
        /// outer computation to obtain the inner one and the threaded state
        /// `s1`, then run the inner computation from `s1`.
        fn join(mma: StateT<S, MKind, StateT<S, MKind, A>>) -> StateT<S, MKind, A> {
            StateT::new(move |s: S| {
                <MKind as monad_kind::Bind<(StateT<S, MKind, A>, S), (A, S)>>::bind(
                    (mma.run_state_t)(s),
                    move |(inner, s1)| (inner.run_state_t)(s1),
                )
            })
        }
    }

    // --- Runners (value / state projections) ---

    impl<S, MKind: Kind1, A> StateT<S, MKind, A> {
        /// Runs the computation and keeps only the produced value, discarding
        /// the final state: `MKind::Of<A>` (inner `Functor`, projecting `fst`).
        #[must_use]
        pub fn eval_state_t(self, s: S) -> MKind::Of<A>
        where
            S: Clone + 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, S), A> + 'static,
            MKind::Of<(A, S)>: 'static,
        {
            MKind::map((self.run_state_t)(s), |(a, _s)| a)
        }

        /// Runs the computation and keeps only the final state, discarding the
        /// value: `MKind::Of<S>` (inner `Functor`, projecting `snd`).
        #[must_use]
        pub fn exec_state_t(self, s: S) -> MKind::Of<S>
        where
            S: Clone + 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, S), S> + 'static,
            MKind::Of<(A, S)>: 'static,
        {
            MKind::map((self.run_state_t)(s), |(_a, s2)| s2)
        }
    }

    /// Trait for monads that thread a mutable state `S`.
    ///
    /// The analog of [`MonadReader`](crate::transformers::reader::MonadReader)
    /// for state. The primitive is [`state`](Self::state); the rest derive from
    /// it. Every operation ends in a single `MKind::pure(...)`, so each needs
    /// only the inner [`Applicative`](applicative_kind::Applicative) bound — no
    /// inner `Bind`/`Monad` is required for the `MonadState` surface itself.
    ///
    /// # Type Parameters
    /// - `S`: the state type.
    /// - `A`: the value type produced by [`state`](Self::state)/[`gets`](Self::gets).
    /// - `MKind`: the Kind marker for the inner monad.
    pub trait MonadState<S, A, MKind: Kind1>
    where
        Self: Sized,
    {
        /// Embeds a pure state transition `S -> (A, S)`. The primitive from which
        /// `get`/`put`/`modify`/`gets` derive.
        #[must_use]
        fn state<F>(f: F) -> StateT<S, MKind, A>
        where
            S: Clone + 'static,
            A: 'static,
            MKind: applicative_kind::Applicative<(A, S)> + 'static,
            MKind::Of<(A, S)>: 'static,
            F: Fn(S) -> (A, S) + 'static;

        /// Reads the whole state as the value, leaving it unchanged: `s -> (s, s)`.
        #[must_use]
        fn get() -> StateT<S, MKind, S>
        where
            S: Clone + 'static,
            MKind: applicative_kind::Applicative<(S, S)> + 'static,
            MKind::Of<(S, S)>: 'static;

        /// Replaces the state, returning unit: `_ -> ((), new_state)`.
        #[must_use]
        fn put(new_state: S) -> StateT<S, MKind, ()>
        where
            S: Clone + 'static,
            MKind: applicative_kind::Applicative<((), S)> + 'static,
            MKind::Of<((), S)>: 'static;

        /// Applies `f` to the state, returning unit: `s -> ((), f(s))`.
        #[must_use]
        fn modify<F>(f: F) -> StateT<S, MKind, ()>
        where
            S: Clone + 'static,
            MKind: applicative_kind::Applicative<((), S)> + 'static,
            MKind::Of<((), S)>: 'static,
            F: Fn(S) -> S + 'static;

        /// Projects the state through `f` as the value, leaving the state
        /// unchanged: `s -> (f(s), s)`.
        #[must_use]
        fn gets<F, B>(f: F) -> StateT<S, MKind, B>
        where
            S: Clone + 'static,
            B: 'static,
            MKind: applicative_kind::Applicative<(B, S)> + 'static,
            MKind::Of<(B, S)>: 'static,
            F: Fn(S) -> B + 'static;
    }

    impl<S, MKindImpl, A> MonadState<S, A, MKindImpl> for StateTKind<S, MKindImpl>
    where
        S: 'static,
        A: 'static,
        MKindImpl: Kind1 + 'static,
    {
        fn state<F>(f: F) -> StateT<S, MKindImpl, A>
        where
            S: Clone + 'static,
            A: 'static,
            MKindImpl: applicative_kind::Applicative<(A, S)> + 'static,
            MKindImpl::Of<(A, S)>: 'static,
            F: Fn(S) -> (A, S) + 'static,
        {
            StateT::new(move |s: S| {
                let (a, s2) = f(s);
                MKindImpl::pure((a, s2))
            })
        }

        fn get() -> StateT<S, MKindImpl, S>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<(S, S)> + 'static,
            MKindImpl::Of<(S, S)>: 'static,
        {
            StateT::new(move |s: S| MKindImpl::pure((s.clone(), s)))
        }

        fn put(new_state: S) -> StateT<S, MKindImpl, ()>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<((), S)> + 'static,
            MKindImpl::Of<((), S)>: 'static,
        {
            StateT::new(move |_s: S| MKindImpl::pure(((), new_state.clone())))
        }

        fn modify<F>(f: F) -> StateT<S, MKindImpl, ()>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<((), S)> + 'static,
            MKindImpl::Of<((), S)>: 'static,
            F: Fn(S) -> S + 'static,
        {
            StateT::new(move |s: S| MKindImpl::pure(((), f(s))))
        }

        fn gets<F, B>(f: F) -> StateT<S, MKindImpl, B>
        where
            S: Clone + 'static,
            B: 'static,
            MKindImpl: applicative_kind::Applicative<(B, S)> + 'static,
            MKindImpl::Of<(B, S)>: 'static,
            F: Fn(S) -> B + 'static,
        {
            StateT::new(move |s: S| MKindImpl::pure((f(s.clone()), s)))
        }
    }

    // --- Ergonomic inherent associated functions on the Kind marker ---
    //
    // These replicate the five [`MonadState`] surface methods directly on the
    // `StateTKind<S, MKindImpl>` marker, so callers can write
    // `StateTKind::<i32, IdentityKind>::get()` instead of the verbose
    // `<StateTKind<i32, IdentityKind> as MonadState<i32, i32, IdentityKind>>::get()`.
    // Each method simply delegates to the corresponding `MonadState` impl with
    // identical bounds. The `MonadState` trait itself is unchanged and remains
    // the right choice for code that is generic over the transformer.

    impl<S, MKindImpl: Kind1> StateTKind<S, MKindImpl> {
        /// Embeds a pure state transition `f: S -> (A, S)` — the ergonomic
        /// concrete form of [`MonadState::state`].
        ///
        /// Callers can write `StateTKind::<S, M>::state(f)` instead of
        /// `<StateTKind<S, M> as MonadState<S, A, M>>::state(f)`.
        /// The generic [`MonadState`] trait stays for code that is generic
        /// over the transformer.
        #[must_use]
        pub fn state<F, A>(f: F) -> StateT<S, MKindImpl, A>
        where
            S: Clone + 'static,
            A: 'static,
            MKindImpl: applicative_kind::Applicative<(A, S)> + 'static,
            MKindImpl::Of<(A, S)>: 'static,
            F: Fn(S) -> (A, S) + 'static,
        {
            <Self as MonadState<S, A, MKindImpl>>::state(f)
        }

        /// Reads the whole state as the value, leaving it unchanged
        /// (`s -> (s, s)`) — the ergonomic concrete form of [`MonadState::get`].
        ///
        /// Callers can write `StateTKind::<S, M>::get()` instead of
        /// `<StateTKind<S, M> as MonadState<S, S, M>>::get()`.
        /// The generic [`MonadState`] trait stays for code that is generic
        /// over the transformer.
        #[must_use]
        pub fn get() -> StateT<S, MKindImpl, S>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<(S, S)> + 'static,
            MKindImpl::Of<(S, S)>: 'static,
        {
            <Self as MonadState<S, S, MKindImpl>>::get()
        }

        /// Replaces the state with `new_state`, returning unit
        /// (`_ -> ((), new_state)`) — the ergonomic concrete form of
        /// [`MonadState::put`].
        ///
        /// Callers can write `StateTKind::<S, M>::put(s)` instead of
        /// `<StateTKind<S, M> as MonadState<S, (), M>>::put(s)`.
        /// The generic [`MonadState`] trait stays for code that is generic
        /// over the transformer.
        #[must_use]
        pub fn put(new_state: S) -> StateT<S, MKindImpl, ()>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<((), S)> + 'static,
            MKindImpl::Of<((), S)>: 'static,
        {
            <Self as MonadState<S, (), MKindImpl>>::put(new_state)
        }

        /// Applies `f` to the current state and stores the result, returning
        /// unit (`s -> ((), f(s))`) — the ergonomic concrete form of
        /// [`MonadState::modify`].
        ///
        /// Callers can write `StateTKind::<S, M>::modify(f)` instead of
        /// `<StateTKind<S, M> as MonadState<S, (), M>>::modify(f)`.
        /// The generic [`MonadState`] trait stays for code that is generic
        /// over the transformer.
        #[must_use]
        pub fn modify<F>(f: F) -> StateT<S, MKindImpl, ()>
        where
            S: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<((), S)> + 'static,
            MKindImpl::Of<((), S)>: 'static,
            F: Fn(S) -> S + 'static,
        {
            <Self as MonadState<S, (), MKindImpl>>::modify(f)
        }

        /// Projects the state through `f` as the value, leaving the state
        /// unchanged (`s -> (f(s), s)`) — the ergonomic concrete form of
        /// [`MonadState::gets`].
        ///
        /// Callers can write `StateTKind::<S, M>::gets(f)` instead of
        /// `<StateTKind<S, M> as MonadState<S, B, M>>::gets(f)`.
        /// The generic [`MonadState`] trait stays for code that is generic
        /// over the transformer.
        #[must_use]
        pub fn gets<F, B>(f: F) -> StateT<S, MKindImpl, B>
        where
            S: Clone + 'static,
            B: 'static,
            MKindImpl: applicative_kind::Applicative<(B, S)> + 'static,
            MKindImpl::Of<(B, S)>: 'static,
            F: Fn(S) -> B + 'static,
        {
            <Self as MonadState<S, B, MKindImpl>>::gets(f)
        }
    }
}

// Directly export Kind-based versions
pub use kind::{MonadState, State, StateT, StateTKind};
