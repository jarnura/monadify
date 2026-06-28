//! # WriterT Monad Transformer for the `monadify` library
// Kind-based version is the default (mirrors `reader.rs`/`state.rs`).

pub mod kind {
    //! # Kind-based WriterT Monad Transformer
    //!
    //! `WriterT` (Writer Transformer) augments an underlying monad (`MKind`, a
    //! Kind marker) with a **monoidal output log** of type `W`. A computation
    //! `WriterT<W, MKind, A>` is simply a wrapped inner value
    //! `MKind::Of<(A, W)>` — the produced value paired with an accumulated log.
    //!
    //! ## Relationship to [`ReaderT`](crate::transformers::reader) and [`StateT`](crate::transformers::state)
    //!
    //! `WriterT` belongs to the **`ReaderT` family**: [`apply`](apply_kind::Apply::apply)
    //! needs only an inner [`Apply`](apply_kind::Apply), because two independent
    //! computations' logs are simply *combined* — there is no sequential data
    //! dependency like `StateT`'s threaded state. The new ingredient is that the
    //! log type `W` must be a [`Monoid`]: [`pure`](applicative_kind::Applicative::pure)
    //! seeds the log with [`Monoid::empty`], and every sequencing step
    //! [`combine`](crate::monoid::Semigroup::combine)s the two logs.
    //!
    //! Unlike `ReaderT`/`StateT`, the carrier is a **value**, not a function, so
    //! there is no `run_*` closure to call — [`run_writer_t`](WriterT::run_writer_t)
    //! is the inner `MKind::Of<(A, W)>` directly. The tuple order `(A, W) =
    //! (value, log)` is fixed and never reordered.
    //!
    //! ## Key Components
    //! - [`WriterT<W, MKind, A>`]: the computation, wrapping `MKind::Of<(A, W)>`.
    //! - [`WriterTKind<W, MKind>`]: the Kind marker for `WriterT`.
    //! - [`MonadWriter<W, A, MKind>`]: a trait providing `tell`, `writer`,
    //!   `listen`, and `censor`.
    //! - [`Writer<W, A>`]: alias for `WriterT<W, IdentityKind, A>` (the plain Writer monad).
    //!
    //! ## Example
    //! ```
    //! use monadify::transformers::writer::{Writer, WriterT, WriterTKind, MonadWriter};
    //! use monadify::IdentityKind;
    //! use monadify::Identity;
    //! use monadify::monad::kind::Bind;
    //!
    //! type Logged<A> = Writer<String, A>;          // WriterT<String, IdentityKind, A>
    //! type LoggedKind = WriterTKind<String, IdentityKind>;
    //!
    //! // `tell` appends to the log and yields unit.
    //! let step = |msg: &str| LoggedKind::tell(msg.to_string());
    //!
    //! // Sequence two log-writes; the logs concatenate monoidally.
    //! let prog: Logged<()> = LoggedKind::bind(step("hello "), move |_| step("world"));
    //! let Identity(((), log)) = prog.run_writer_t;
    //! assert_eq!(log, "hello world");
    //!
    //! // `exec_writer_t` keeps only the log.
    //! let prog2: Logged<()> = LoggedKind::bind(step("a"), move |_| step("b"));
    //! assert_eq!(prog2.exec_writer_t(), Identity("ab".to_string()));
    //! ```

    use crate::applicative::kind as applicative_kind;
    use crate::apply::kind as apply_kind;
    use crate::function::RcFn; // Apply's function container (CFn removed in 0.2.0).
    use crate::functor::kind as functor_kind;
    use crate::identity::kind::IdentityKind;
    use crate::kind_based::kind::{Kind, Kind1};
    use crate::monad::kind as monad_kind;
    use crate::monoid::Monoid;
    use std::marker::PhantomData;

    /// The `WriterT` monad transformer for Kind-encoded types.
    ///
    /// `WriterT<W, MKind, A>` wraps an inner value `MKind::Of<(A, W)>`: the
    /// produced value `A` paired with an accumulated monoidal log `W`. The value
    /// is stored in [`run_writer_t`](Self::run_writer_t).
    ///
    /// # Type Parameters
    /// - `W`: the log type (must be a [`Monoid`] for the
    ///   `Applicative`/`Monad` instances).
    /// - `MKind`: the Kind marker for the inner monad (must implement [`Kind1`]).
    /// - `A`: the value produced alongside the log.
    pub struct WriterT<W, MKind: Kind1, A> {
        /// The wrapped inner computation: `MKind::Of<(A, W)>` (value-then-log).
        pub run_writer_t: MKind::Of<(A, W)>,
        _phantom: PhantomData<(W, MKind, A)>,
    }

    // Manual `Clone` bounded on the projected inner type (mirrors `RcFn`): a
    // derived `Clone` would demand `W: Clone, MKind: Clone, A: Clone`, none of
    // which is the real requirement — only `MKind::Of<(A, W)>: Clone` is.
    impl<W, MKind: Kind1, A> Clone for WriterT<W, MKind, A>
    where
        MKind::Of<(A, W)>: Clone,
    {
        fn clone(&self) -> Self {
            WriterT {
                run_writer_t: self.run_writer_t.clone(),
                _phantom: PhantomData,
            }
        }
    }

    impl<W, MKind: Kind1, A> WriterT<W, MKind, A> {
        /// Creates a new `WriterT` from an inner value `MKind::Of<(A, W)>`.
        #[must_use]
        pub fn new(inner: MKind::Of<(A, W)>) -> Self {
            WriterT {
                run_writer_t: inner,
                _phantom: PhantomData,
            }
        }
    }

    /// The Kind marker for `WriterT<W, MKind, _>`.
    ///
    /// Used to implement [`Functor`](functor_kind::Functor),
    /// [`Apply`](apply_kind::Apply), [`Applicative`](applicative_kind::Applicative),
    /// [`Bind`](monad_kind::Bind), and [`Monad`](monad_kind::Monad) for the
    /// `WriterT` type constructor.
    ///
    /// # Type Parameters
    /// - `W`: the log type.
    /// - `MKind`: the Kind marker for the inner monad.
    #[derive(Default)]
    pub struct WriterTKind<W, MKind: Kind1>(PhantomData<(W, MKind)>);

    impl<W, MKind: Kind1> Kind for WriterTKind<W, MKind> {
        type Of<A> = WriterT<W, MKind, A>;
    }
    // Kind1 is provided by the blanket impl in kind_based/kind.rs.

    impl<W, MKindImpl: Kind1> WriterTKind<W, MKindImpl> {
        /// Appends `w` to the log and produces unit — the ergonomic alternative to
        /// `<WriterTKind<W, MKindImpl> as MonadWriter<W, (), MKindImpl>>::tell(w)`.
        ///
        /// Avoids the UFCS syntax for the common case. Generic code that is
        /// parameterized over the concrete Kind marker should continue to use the
        /// trait form.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::writer::{Writer, WriterTKind};
        /// use monadify::identity::{Identity, IdentityKind};
        ///
        /// let w: Writer<String, ()> =
        ///     WriterTKind::<String, IdentityKind>::tell("hello".to_string());
        /// let Identity(((), log)) = w.run_writer_t;
        /// assert_eq!(log, "hello");
        /// ```
        #[must_use]
        pub fn tell(w: W) -> WriterT<W, MKindImpl, ()>
        where
            W: 'static,
            MKindImpl: applicative_kind::Applicative<((), W)> + 'static,
            MKindImpl::Of<((), W)>: 'static,
        {
            <Self as MonadWriter<W, (), MKindImpl>>::tell(w)
        }

        /// Embeds a `(value, log)` pair directly — the ergonomic alternative to
        /// `<WriterTKind<W, MKindImpl> as MonadWriter<W, A, MKindImpl>>::writer(value, log)`.
        ///
        /// Avoids the UFCS syntax for the common case. Generic code that is
        /// parameterized over the concrete Kind marker should continue to use the
        /// trait form.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::writer::{Writer, WriterTKind};
        /// use monadify::identity::{Identity, IdentityKind};
        ///
        /// let w: Writer<String, i32> =
        ///     WriterTKind::<String, IdentityKind>::writer(42, "note".to_string());
        /// let Identity((val, log)) = w.run_writer_t;
        /// assert_eq!((val, log), (42, "note".to_string()));
        /// ```
        #[must_use]
        pub fn writer<A>(value: A, log: W) -> WriterT<W, MKindImpl, A>
        where
            W: 'static,
            A: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<(A, W)> + 'static,
            MKindImpl::Of<(A, W)>: 'static,
        {
            <Self as MonadWriter<W, A, MKindImpl>>::writer(value, log)
        }
    }

    /// A type alias for `WriterT` with [`IdentityKind`] as the inner monad.
    /// This is the plain Writer monad: `Writer<W, A>` wraps `Identity<(A, W)>`.
    pub type Writer<W, A> = WriterT<W, IdentityKind, A>;

    // --- Kind Trait Implementations for WriterTKind ---

    impl<W, MKind, A, B> functor_kind::Functor<A, B> for WriterTKind<W, MKind>
    where
        W: 'static,
        MKind: functor_kind::Functor<(A, W), (B, W)> + Kind1 + 'static,
        A: 'static,
        B: 'static,
        MKind::Of<(A, W)>: 'static,
        MKind::Of<(B, W)>: 'static,
    {
        /// Maps `A -> B` over the produced value, leaving the log untouched.
        /// The mapping happens within the inner monad `MKind`.
        fn map(
            input: WriterT<W, MKind, A>,
            func: impl FnMut(A) -> B + Clone + 'static,
        ) -> WriterT<W, MKind, B> {
            let mut f = func;
            WriterT::new(MKind::map(input.run_writer_t, move |(a, w)| (f(a), w)))
        }
    }

    impl<W, MKind, A, B> apply_kind::Apply<A, B> for WriterTKind<W, MKind>
    where
        W: Monoid + Clone + 'static,
        A: 'static,
        B: 'static,
        // `Apply<A, B>: Functor<A, B>` (the inner `Functor<(A,W),(B,W)>` bound),
        // plus the inner `Functor` that lifts the function pair into a function
        // on pairs, plus the inner `Apply` that combines the two computations.
        MKind: functor_kind::Functor<(A, W), (B, W)>
            + functor_kind::Functor<(RcFn<A, B>, W), RcFn<(A, W), (B, W)>>
            + apply_kind::Apply<(A, W), (B, W)>
            + Kind1
            + 'static,
        MKind::Of<(A, W)>: 'static,
        MKind::Of<(B, W)>: 'static,
        MKind::Of<(RcFn<A, B>, W)>: 'static,
        MKind::Of<RcFn<(A, W), (B, W)>>: 'static,
    {
        /// Applies a wrapped function to a wrapped value, **combining the logs**.
        /// The two computations are independent (no data dependency), so this is
        /// derived from the inner monad's `apply`; the result log is
        /// `func_log.combine(value_log)`.
        fn apply(
            value_container: WriterT<W, MKind, A>,
            function_container: WriterT<W, MKind, RcFn<A, B>>,
        ) -> WriterT<W, MKind, B> {
            // Lift the function pair `(g, w1)` into a function on value pairs
            // `(a, w2) -> (g(a), w1 <> w2)`, then apply it inside `MKind`.
            #[allow(clippy::type_complexity)]
            let m_lifted: MKind::Of<RcFn<(A, W), (B, W)>> = MKind::map(
                function_container.run_writer_t,
                |(g, w1): (RcFn<A, B>, W)| {
                    RcFn::new(move |(a, w2): (A, W)| (g.call(a), w1.clone().combine(w2)))
                },
            );
            WriterT::new(MKind::apply(value_container.run_writer_t, m_lifted))
        }
    }

    impl<W, MKind, T> applicative_kind::Applicative<T> for WriterTKind<W, MKind>
    where
        W: Monoid + Clone + 'static,
        T: 'static, // `pure` consumes the value once — no `Clone` needed (unlike `ReaderT`).
        // `Applicative<T>: Apply<T, T>`, so the full `Apply<T, T>` bound set is
        // dragged in here; `pure` itself needs the inner `Applicative<(T, W)>`.
        MKind: functor_kind::Functor<(T, W), (T, W)>
            + functor_kind::Functor<(RcFn<T, T>, W), RcFn<(T, W), (T, W)>>
            + apply_kind::Apply<(T, W), (T, W)>
            + applicative_kind::Applicative<(T, W)>
            + Kind1
            + 'static,
        MKind::Of<(T, W)>: 'static,
        MKind::Of<(RcFn<T, T>, W)>: 'static,
        MKind::Of<RcFn<(T, W), (T, W)>>: 'static,
    {
        /// Lifts a value `T` into `WriterT` with an **empty** log:
        /// `MKind::pure((value, Monoid::empty()))`.
        fn pure(value: T) -> WriterT<W, MKind, T> {
            WriterT::new(MKind::pure((value, W::empty())))
        }
    }

    impl<W, MKind, A, B> monad_kind::Bind<A, B> for WriterTKind<W, MKind>
    where
        W: Monoid + Clone + 'static,
        A: 'static,
        B: 'static,
        // `Bind<A, B>: Apply<A, B>`, so the full `Apply<A, B>` bound set is
        // dragged in, plus the inner `Bind`/`Functor` the `bind` body uses to
        // sequence the two computations and combine their logs.
        MKind: functor_kind::Functor<(A, W), (B, W)>
            + functor_kind::Functor<(RcFn<A, B>, W), RcFn<(A, W), (B, W)>>
            + apply_kind::Apply<(A, W), (B, W)>
            + monad_kind::Bind<(A, W), (B, W)>
            + functor_kind::Functor<(B, W), (B, W)>
            + Kind1
            + 'static,
        MKind::Of<(A, W)>: 'static,
        MKind::Of<(B, W)>: 'static,
        MKind::Of<(RcFn<A, B>, W)>: 'static,
        MKind::Of<RcFn<(A, W), (B, W)>>: 'static,
    {
        /// Sequentially composes a `WriterT` with a function producing the next
        /// `WriterT`, **combining the logs**: the first computation's log `w1`
        /// is prepended to the second's log `w2`, giving `w1.combine(w2)`.
        fn bind(
            input: WriterT<W, MKind, A>,
            func: impl FnMut(A) -> WriterT<W, MKind, B> + Clone + 'static,
        ) -> WriterT<W, MKind, B> {
            let mut f = func;
            WriterT::new(MKind::bind(input.run_writer_t, move |(a, w1): (A, W)| {
                let next: WriterT<W, MKind, B> = f(a);
                // `w1` is already owned here; move it straight into the inner
                // closure, which clones per call (the inner map may run >1 time).
                MKind::map(next.run_writer_t, move |(b, w2): (B, W)| {
                    (b, w1.clone().combine(w2))
                })
            }))
        }
    }

    impl<W, MKind, A> monad_kind::Monad<A> for WriterTKind<W, MKind>
    where
        W: Monoid + Clone + 'static,
        A: Clone + 'static,
        // `Monad<A>: Applicative<A>: Apply<A, A>`, so the full `Apply<A, A>`
        // bound set is dragged in, plus the inner `Bind`/`Functor` `join` uses.
        MKind: functor_kind::Functor<(A, W), (A, W)>
            + functor_kind::Functor<(RcFn<A, A>, W), RcFn<(A, W), (A, W)>>
            + apply_kind::Apply<(A, W), (A, W)>
            + applicative_kind::Applicative<(A, W)>
            + monad_kind::Bind<(WriterT<W, MKind, A>, W), (A, W)>
            + Kind1
            + 'static,
        MKind::Of<(A, W)>: 'static,
        MKind::Of<(RcFn<A, A>, W)>: 'static,
        MKind::Of<RcFn<(A, W), (A, W)>>: 'static,
        MKind::Of<(WriterT<W, MKind, A>, W)>: 'static,
    {
        /// Flattens a nested `WriterT<W, MKind, WriterT<W, MKind, A>>`: run the
        /// outer computation to obtain the inner one and the outer log `w1`,
        /// then combine `w1` with the inner computation's log `w2`.
        fn join(mma: WriterT<W, MKind, WriterT<W, MKind, A>>) -> WriterT<W, MKind, A> {
            WriterT::new(<MKind as monad_kind::Bind<
                (WriterT<W, MKind, A>, W),
                (A, W),
            >>::bind(
                mma.run_writer_t,
                move |(inner, w1): (WriterT<W, MKind, A>, W)| {
                    <MKind as functor_kind::Functor<(A, W), (A, W)>>::map(
                        inner.run_writer_t,
                        move |(a, w2): (A, W)| (a, w1.clone().combine(w2)),
                    )
                },
            ))
        }
    }

    // --- Runner (log projection) ---

    impl<W, MKind: Kind1, A> WriterT<W, MKind, A> {
        /// Runs the computation and keeps only the produced value, discarding
        /// the log: `MKind::Of<A>` (inner `Functor`, projecting `fst`).
        #[must_use]
        pub fn eval_writer_t(self) -> MKind::Of<A>
        where
            W: 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), A> + 'static,
            MKind::Of<(A, W)>: 'static,
        {
            MKind::map(self.run_writer_t, |(a, _w)| a)
        }

        /// Runs the computation and keeps only the accumulated log, discarding
        /// the value: `MKind::Of<W>` (inner `Functor`, projecting `snd`).
        #[must_use]
        pub fn exec_writer_t(self) -> MKind::Of<W>
        where
            W: 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), W> + 'static,
            MKind::Of<(A, W)>: 'static,
        {
            MKind::map(self.run_writer_t, |(_a, w)| w)
        }

        /// Exposes the accumulated log alongside the value, leaving it in place —
        /// the chainable method form of
        /// `<WriterTKind<W, MKind> as MonadWriter<W, A, MKind>>::listen(self)`.
        ///
        /// The produced value becomes `(A, W)` and the log remains `W`.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::writer::{Writer, WriterTKind, MonadWriter};
        /// use monadify::identity::{Identity, IdentityKind};
        ///
        /// let w: Writer<String, ()> =
        ///     WriterTKind::<String, IdentityKind>::tell("xyz".to_string());
        /// let Identity((((), exposed), log)) = w.listen().run_writer_t;
        /// assert_eq!(exposed, "xyz");
        /// assert_eq!(log, "xyz");
        /// ```
        #[must_use]
        pub fn listen(self) -> WriterT<W, MKind, (A, W)>
        where
            W: Clone + 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), ((A, W), W)> + 'static,
            MKind::Of<(A, W)>: 'static,
            MKind::Of<((A, W), W)>: 'static,
        {
            <WriterTKind<W, MKind> as MonadWriter<W, A, MKind>>::listen(self)
        }

        /// Rewrites the log with `f`, leaving the value unchanged — the chainable
        /// method form of
        /// `<WriterTKind<W, MKind> as MonadWriter<W, A, MKind>>::censor(f, self)`.
        ///
        /// The result is `(a, f(w))`.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::writer::{Writer, WriterTKind, MonadWriter};
        /// use monadify::identity::{Identity, IdentityKind};
        ///
        /// let w: Writer<String, ()> =
        ///     WriterTKind::<String, IdentityKind>::tell("quiet".to_string());
        /// let Identity(((), log)) = w.censor(|s: String| s.to_uppercase()).run_writer_t;
        /// assert_eq!(log, "QUIET");
        /// ```
        #[must_use]
        pub fn censor<F>(self, f: F) -> Self
        where
            W: 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), (A, W)> + 'static,
            MKind::Of<(A, W)>: 'static,
            F: Fn(W) -> W + Clone + 'static,
        {
            <WriterTKind<W, MKind> as MonadWriter<W, A, MKind>>::censor(f, self)
        }
    }

    /// Trait for monads that accumulate a monoidal output log `W`.
    ///
    /// The Writer analog of [`MonadReader`](crate::transformers::reader::MonadReader)
    /// (`ask`/`local`) and [`MonadState`](crate::transformers::state::MonadState)
    /// (`get`/`put`/…). The primitive is [`tell`](Self::tell); [`writer`](Self::writer)
    /// is the general constructor, while [`listen`](Self::listen) and
    /// [`censor`](Self::censor) inspect and rewrite the log.
    ///
    /// `tell`/`writer` need only the inner [`Applicative`](applicative_kind::Applicative);
    /// `listen`/`censor` need only the inner [`Functor`](functor_kind::Functor).
    ///
    /// # Type Parameters
    /// - `W`: the log type.
    /// - `A`: the value type carried by the computation.
    /// - `MKind`: the Kind marker for the inner monad.
    pub trait MonadWriter<W, A, MKind: Kind1>
    where
        Self: Sized,
    {
        /// Appends `w` to the log, producing unit: `((), w)`. The primitive.
        #[must_use]
        fn tell(w: W) -> WriterT<W, MKind, ()>
        where
            W: 'static,
            MKind: applicative_kind::Applicative<((), W)> + 'static,
            MKind::Of<((), W)>: 'static;

        /// The general constructor: embeds a `(value, log)` pair directly.
        #[must_use]
        fn writer(value: A, log: W) -> WriterT<W, MKind, A>
        where
            W: 'static,
            A: Clone + 'static,
            MKind: applicative_kind::Applicative<(A, W)> + 'static,
            MKind::Of<(A, W)>: 'static;

        /// Exposes the accumulated log alongside the value, leaving it in place:
        /// turns a `WriterT<W, M, A>` into `WriterT<W, M, (A, W)>` with the same log.
        #[must_use]
        fn listen(m: WriterT<W, MKind, A>) -> WriterT<W, MKind, (A, W)>
        where
            W: Clone + 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), ((A, W), W)> + 'static,
            MKind::Of<(A, W)>: 'static,
            MKind::Of<((A, W), W)>: 'static;

        /// Rewrites the log with `f`, leaving the value unchanged: `(a, f(w))`.
        /// (The ergonomic specialization of Haskell's `pass`.)
        #[must_use]
        fn censor<F>(f: F, m: WriterT<W, MKind, A>) -> WriterT<W, MKind, A>
        where
            W: 'static,
            A: 'static,
            MKind: functor_kind::Functor<(A, W), (A, W)> + 'static,
            MKind::Of<(A, W)>: 'static,
            F: Fn(W) -> W + Clone + 'static;
    }

    impl<W, MKindImpl, A> MonadWriter<W, A, MKindImpl> for WriterTKind<W, MKindImpl>
    where
        W: 'static,
        A: 'static,
        MKindImpl: Kind1 + 'static,
    {
        fn tell(w: W) -> WriterT<W, MKindImpl, ()>
        where
            W: 'static,
            MKindImpl: applicative_kind::Applicative<((), W)> + 'static,
            MKindImpl::Of<((), W)>: 'static,
        {
            WriterT::new(MKindImpl::pure(((), w)))
        }

        fn writer(value: A, log: W) -> WriterT<W, MKindImpl, A>
        where
            W: 'static,
            A: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<(A, W)> + 'static,
            MKindImpl::Of<(A, W)>: 'static,
        {
            WriterT::new(MKindImpl::pure((value, log)))
        }

        fn listen(m: WriterT<W, MKindImpl, A>) -> WriterT<W, MKindImpl, (A, W)>
        where
            W: Clone + 'static,
            A: 'static,
            MKindImpl: functor_kind::Functor<(A, W), ((A, W), W)> + 'static,
            MKindImpl::Of<(A, W)>: 'static,
            MKindImpl::Of<((A, W), W)>: 'static,
        {
            WriterT::new(MKindImpl::map(m.run_writer_t, |(a, w): (A, W)| {
                ((a, w.clone()), w)
            }))
        }

        fn censor<F>(f: F, m: WriterT<W, MKindImpl, A>) -> WriterT<W, MKindImpl, A>
        where
            W: 'static,
            A: 'static,
            MKindImpl: functor_kind::Functor<(A, W), (A, W)> + 'static,
            MKindImpl::Of<(A, W)>: 'static,
            F: Fn(W) -> W + Clone + 'static,
        {
            WriterT::new(MKindImpl::map(m.run_writer_t, move |(a, w): (A, W)| {
                (a, f(w))
            }))
        }
    }
}

// Directly export Kind-based versions
pub use kind::{MonadWriter, Writer, WriterT, WriterTKind};
