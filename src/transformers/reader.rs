//! # ReaderT Monad Transformer for the `monadify` library
// Kind-based version is now default.

pub mod kind {
    // Renamed from hkt to kind
    //! # Kind-based ReaderT Monad Transformer
    //!
    //! This module provides the Kind-based implementation of the `ReaderT` monad transformer
    //! for the `monadify` library.
    //! `ReaderT` (Reader Transformer) adds a read-only environment (of type `R`)
    //! to an underlying monad (represented by `MKind`, a Kind marker).
    //!
    //! Computations of type `ReaderT<R, MKind, A>` are essentially functions
    //! of the form `R -> MKind::Of<A>`, where `MKind::Of<A>` is the
    //! actual inner monadic type (e.g., `Option<A>`, `Result<A, E>`).
    //!
    //! ## Key Components
    //! - [`ReaderT<R, MKind, A>`]: The main struct representing a computation
    //!   that depends on an environment `R` and results in `MKind::Of<A>`.
    //! - [`ReaderTKind<R, MKind>`]: The Kind marker for `ReaderT`.
    //! - [`MonadReader<REnv, AVal, MKind>`]: A trait providing `ask` (to get the
    //!   environment) and `local` (to run a computation with a modified environment).
    //! - [`Reader<R, A>`]: A type alias for `ReaderT<R, IdentityKind, A>`,
    //!   representing a simple Reader monad (not a transformer).
    //!
    //! ## Example
    //! ```
    //! use monadify::transformers::reader::kind::{ReaderT, ReaderTKind, MonadReader, Reader};
    //! use monadify::kind_based::kind::OptionKind; // OptionKind from kind_based::kind
    //! use monadify::IdentityKind; // IdentityKind re-exported from crate::identity
    //! use monadify::functor::kind::Functor;
    //! use monadify::applicative::kind::Applicative;
    //! use monadify::monad::kind::{Bind, Monad};
    //! use std::rc::Rc;
    //!
    //! // Define a configuration environment
    //! #[derive(Clone)]
    //! struct Config {
    //!     greeting: String,
    //!     count: i32,
    //! }
    //!
    //! // Type alias for ReaderT with Option as inner monad and Config as environment
    //! type ConfigReaderOption<A> = ReaderT<Config, OptionKind, A>;
    //! type ConfigReaderOptionKind = ReaderTKind<Config, OptionKind>;
    //!
    //! // 1. Using 'ask' to get the environment (ergonomic inherent form)
    //! let ask_for_greeting_op: ConfigReaderOption<Config> = ConfigReaderOptionKind::ask();
    //! let ask_for_greeting: ConfigReaderOption<String> = <ConfigReaderOptionKind as Functor<Config, String>>::map(
    //!     ask_for_greeting_op,
    //!     |config: Config| config.greeting
    //! );
    //! let config1 = Config { greeting: "Hello".to_string(), count: 5 };
    //! assert_eq!((ask_for_greeting.run_reader_t)(config1.clone()), Some("Hello".to_string()));
    //!
    //! // 2. Using 'pure' and 'map'
    //! let pure_val: ConfigReaderOption<i32> = ConfigReaderOptionKind::pure(10);
    //! let mapped_val: ConfigReaderOption<i32> = <ConfigReaderOptionKind as Functor<i32, i32>>::map(pure_val, |x| x * 2);
    //! assert_eq!((mapped_val.run_reader_t)(config1.clone()), Some(20));
    //!
    //! // 3. Using 'bind'
    //! let ask_op_for_bind: ConfigReaderOption<Config> = ConfigReaderOptionKind::ask();
    //! let computation: ConfigReaderOption<String> = ConfigReaderOptionKind::bind(
    //!     ask_op_for_bind, // Gets Config
    //!     |config: Config| {
    //!         if config.count > 3 {
    //!             ConfigReaderOptionKind::pure(format!("{} times {}", config.greeting, config.count))
    //!         } else {
    //!             ReaderT::new(|_cfg| None) // Create a None in the context
    //!         }
    //!     }
    //! );
    //! assert_eq!(
    //!     (computation.run_reader_t)(config1.clone()),
    //!     Some("Hello times 5".to_string())
    //! );
    //!
    //! // 4. Using 'local' to modify the environment for a sub-computation
    //! let local_computation: ConfigReaderOption<String> = ConfigReaderOptionKind::local(
    //!     |mut cfg: Config| { cfg.greeting = "Hola".to_string(); cfg.count = 2; cfg },
    //!     computation.clone() // The computation from step 3
    //! );
    //! // The original computation would yield Some("Hello times 5") with config1.
    //! // The local_computation modifies env, so inner computation sees count=2, resulting in None.
    //! assert_eq!((local_computation.run_reader_t)(config1.clone()), None);
    //!
    //! // 5. Using 'join' (example with Reader<R, Reader<R, A>>)
    //! type SimpleReader<A> = Reader<Config, A>; // ReaderT<Config, IdentityKind, A>
    //! type SimpleReaderKind = ReaderTKind<Config, IdentityKind>;
    //! use monadify::Kind; // For Kind::Of
    //!
    //! let val_in_id: <IdentityKind as Kind>::Of<SimpleReader<String>> =
    //!     IdentityKind::pure(ReaderT::new(|cfg: Config| IdentityKind::pure(cfg.greeting)));
    //!
    //! let nested_reader: SimpleReader<SimpleReader<String>> =
    //!     ReaderT::new(move |_cfg: Config| val_in_id.clone());
    //!
    //! let joined_reader: SimpleReader<String> = SimpleReaderKind::join(nested_reader);
    //! assert_eq!((joined_reader.run_reader_t)(config1.clone()), IdentityKind::pure("Hello".to_string()));
    //! ```

    use crate::applicative::kind as applicative_kind; // Renamed hkt to kind
    use crate::apply::kind as apply_kind; // Renamed hkt to kind
    use crate::function::RcFn; // For Apply's function container type
    use crate::functor::kind as functor_kind; // Renamed hkt to kind
    use crate::identity::kind::IdentityKind;
    use crate::kind_based::kind::{Kind, Kind1}; // Changed HKT, HKT1 to Kind, Kind1
    use crate::monad::kind as monad_kind; // Renamed hkt to kind
    use std::marker::PhantomData;
    use std::rc::Rc; // Changed IdentityHKTMarker to IdentityKind

    /// The `ReaderT` monad transformer for Kind-encoded types.
    ///
    /// `ReaderT<R, MKind, A>` represents a computation that:
    /// 1. Takes an environment of type `R`.
    /// 2. Produces a value of type `A` wrapped in an inner monad `MKind`.
    ///
    /// The actual computation is stored in `run_reader_t`, which is a function
    /// `R -> MKind::Of<A>`. `MKind::Of<A>` is the concrete type
    /// of the inner monad (e.g., `Option<A>`, `Result<A, E>`).
    ///
    /// # Type Parameters
    /// - `R`: The type of the read-only environment.
    /// - `MKind`: The Kind marker for the inner monad (e.g., [`crate::kind_based::kind::OptionKind`]).
    ///   It must implement [`Kind1`].
    /// - `A`: The type of the value produced by the computation within the inner monad.
    #[derive(Clone)]
    pub struct ReaderT<R, MKind: Kind1, A> {
        // Changed MMarker to MKind, HKT1 to Kind1
        /// The core function that defines the `ReaderT` computation.
        /// It takes an environment `R` and returns the result wrapped in the inner monad `MKind::Of<A>`.
        pub run_reader_t: Rc<dyn Fn(R) -> MKind::Of<A> + 'static>, // Changed MMarker::Applied to MKind::Of
        _phantom: PhantomData<(R, MKind, A)>,
    }

    impl<R, MKind: Kind1, A> ReaderT<R, MKind, A> {
        // Changed MMarker to MKind, HKT1 to Kind1
        /// Creates a new `ReaderT` from a function `R -> MKind::Of<A>`.
        #[must_use]
        pub fn new<F>(f: F) -> Self
        where
            F: Fn(R) -> MKind::Of<A> + 'static, // Changed MMarker::Applied to MKind::Of
        {
            ReaderT {
                run_reader_t: Rc::new(f),
                _phantom: PhantomData,
            }
        }

        /// Runs `self` in a modified environment — the chainable method form of
        /// [`MonadReader::local`].
        ///
        /// Applies `f` to the environment before the computation sees it, without
        /// changing the environment visible to the surrounding context.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::reader::kind::{ReaderT, ReaderTKind};
        /// use monadify::identity::kind::{Identity, IdentityKind};
        ///
        /// let comp: ReaderT<i32, IdentityKind, i32> = ReaderT::new(|env: i32| Identity(env * 2));
        /// let result = (comp.local(|env: i32| env + 10).run_reader_t)(5);
        /// assert_eq!(result, Identity(30)); // (5+10)*2 = 30
        /// ```
        #[must_use]
        pub fn local<F>(self, f: F) -> Self
        where
            R: 'static,
            A: 'static,
            MKind: 'static,
            MKind::Of<A>: 'static,
            F: Fn(R) -> R + 'static,
        {
            <ReaderTKind<R, MKind> as MonadReader<R, A, MKind>>::local(f, self)
        }
    }

    /// The Kind marker for `ReaderT<R, MKind, _>`.
    ///
    /// This struct is used to implement Kind traits like `Functor`, `Applicative`, `Monad`
    /// for the `ReaderT` type constructor.
    ///
    /// # Type Parameters
    /// - `R`: The environment type.
    /// - `MKind`: The Kind marker for the inner monad.
    #[derive(Default)]
    pub struct ReaderTKind<R, MKind: Kind1>(PhantomData<(R, MKind)>); // Renamed ReaderTHKTMarker, MMarker to MKind, HKT1 to Kind1

    impl<R, MKind: Kind1> Kind for ReaderTKind<R, MKind> {
        // Renamed ReaderTHKTMarker, MMarker to MKind, HKT to Kind, HKT1 to Kind1
        type Of<A> = ReaderT<R, MKind, A>; // Changed Applied to Of
    }
    // Kind1 is implemented by the blanket impl in kind_based/kind.rs for types that impl Kind.

    /// A type alias for `ReaderT` with [`IdentityKind`] as the inner monad.
    /// This represents a simple Reader monad (not a transformer).
    /// `Reader<R, A>` is a computation `R -> Identity<A>`.
    pub type Reader<R, A> = ReaderT<R, IdentityKind, A>; // Changed IdentityHKTMarker to IdentityKind

    // --- Kind Trait Implementations for ReaderTKind ---

    impl<R, MKind, A, B> functor_kind::Functor<A, B> for ReaderTKind<R, MKind>
    // Renamed ReaderTHKTMarker, MMarker to MKind
    where
        R: Clone + 'static,
        MKind: functor_kind::Functor<A, B> + Kind1 + 'static, // Inner MKind must be Functor. HKT1 to Kind1
        A: 'static,
        B: 'static,
        MKind::Of<A>: 'static, // M<A>. Applied to Of
        MKind::Of<B>: 'static, // M<B>. Applied to Of
    {
        /// Maps a function `A -> B` over the value within the `ReaderT` context.
        /// The environment `R` is passed through. The mapping happens within the inner monad `MKind`.
        fn map(
            input: ReaderT<R, MKind, A>,
            func: impl FnMut(A) -> B + Clone + 'static,
        ) -> ReaderT<R, MKind, B> {
            let run_reader_t_clone = input.run_reader_t.clone();
            ReaderT::new(move |env: R| {
                let m_val: MKind::Of<A> = run_reader_t_clone(env); // Applied to Of
                MKind::map(m_val, func.clone()) // Delegate to MKind's map
            })
        }
    }

    impl<R, MKind, A, B> apply_kind::Apply<A, B> for ReaderTKind<R, MKind>
    // Renamed ReaderTHKTMarker, MMarker to MKind
    where
        R: Clone + 'static,
        MKind: apply_kind::Apply<A, B> + Kind1 + 'static, // Inner MKind must be Apply. HKT1 to Kind1
        A: 'static,
        B: 'static,
        MKind::Of<A>: 'static,          // M<A>. Applied to Of
        MKind::Of<B>: 'static,          // M<B>. Applied to Of
        MKind::Of<RcFn<A, B>>: 'static, // M<RcFn<A,B>>.
    {
        /// Applies a wrapped function within `ReaderT` to a wrapped value within `ReaderT`.
        /// Both computations share the same environment `R`. The application happens within `MKind`.
        fn apply(
            value_container: ReaderT<R, MKind, A>,
            function_container: ReaderT<R, MKind, RcFn<A, B>>,
        ) -> ReaderT<R, MKind, B> {
            let value_run = value_container.run_reader_t.clone();
            let function_run = function_container.run_reader_t.clone();
            ReaderT::new(move |env: R| {
                let m_val: MKind::Of<A> = value_run(env.clone());
                let m_func: MKind::Of<RcFn<A, B>> = function_run(env);
                MKind::apply(m_val, m_func) // Delegate to MKind's apply
            })
        }
    }

    impl<R, MKind, T> applicative_kind::Applicative<T> for ReaderTKind<R, MKind>
    // Renamed ReaderTHKTMarker, MMarker to MKind
    where
        R: Clone + 'static, // Though _env is not used, new needs Fn(R)
        MKind: applicative_kind::Applicative<T> + Kind1 + 'static, // Inner MKind must be Applicative. HKT1 to Kind1
        T: Clone + 'static,                                        // For MKind::pure(value.clone())
        MKind::Of<T>: 'static,                                     // M<T>. Applied to Of
    {
        /// Lifts a value `T` into the `ReaderT` context.
        /// The resulting computation ignores the environment and returns `MKind::pure(value)`.
        fn pure(value: T) -> ReaderT<R, MKind, T> {
            ReaderT::new(move |_env: R| MKind::pure(value.clone()))
        }
    }

    impl<R, MKind, A, B> monad_kind::Bind<A, B> for ReaderTKind<R, MKind>
    // Renamed ReaderTHKTMarker, MMarker to MKind
    where
        R: Clone + 'static,
        MKind: monad_kind::Bind<A, B> + Kind1 + 'static, // Inner MKind must be Bind. HKT1 to Kind1
        A: 'static,
        B: 'static,
        MKind::Of<A>: 'static, // M<A>. Applied to Of
        MKind::Of<B>: 'static, // M<B>. Applied to Of
    {
        /// Sequentially composes a `ReaderT` computation with a function that returns a new `ReaderT`.
        /// The environment `R` is passed to both the initial computation and the one produced by `func`.
        /// The `bind` operation itself is delegated to the inner monad `MKind`.
        fn bind(
            input: ReaderT<R, MKind, A>,
            func: impl FnMut(A) -> ReaderT<R, MKind, B> + Clone + 'static,
        ) -> ReaderT<R, MKind, B> {
            let self_run = input.run_reader_t.clone();
            ReaderT::new(move |env: R| {
                let m_a_val: MKind::Of<A> = self_run(env.clone()); // Applied to Of
                let mut f_clone = func.clone();
                // Delegate to MKind's bind.
                // The function for MKind::bind takes `A` and must return `MKind::Of<B>`.
                // `f_clone(a_val)` returns `ReaderT<R, MKind, B>`.
                // We run this `ReaderT` with the current `env` to get `MKind::Of<B>`.
                MKind::bind(m_a_val, move |a_val: A| {
                    let next_reader_t: ReaderT<R, MKind, B> = f_clone(a_val);
                    (next_reader_t.run_reader_t)(env.clone())
                })
            })
        }
    }

    impl<R, MKind, A> monad_kind::Monad<A> for ReaderTKind<R, MKind>
    // Renamed ReaderTHKTMarker, MMarker to MKind
    where
        R: Clone + 'static,
        MKind: applicative_kind::Applicative<A> // For ReaderTKind's Monad<A> supertrait Applicative<A>
            + monad_kind::Bind<ReaderT<R, MKind, A>, A> // For the join implementation
            + Kind1 // HKT1 to Kind1
            + 'static,
        A: Clone + 'static,    // From Applicative<A> constraint on ReaderTKind
        MKind::Of<A>: 'static, // M<A>. Applied to Of
        MKind::Of<ReaderT<R, MKind, A>>: 'static, // M<ReaderT<R,M,A>>. Applied to Of
    {
        /// Flattens a nested `ReaderT<R, MKind, ReaderT<R, MKind, A>>` into
        /// `ReaderT<R, MKind, A>`.
        /// This is achieved by running the outer `ReaderT` to get `MKind::Of<ReaderT<R,MKind,A>>`,
        /// then using `MKind::bind` to run the inner `ReaderT` with the same environment.
        fn join(mma: ReaderT<R, MKind, ReaderT<R, MKind, A>>) -> ReaderT<R, MKind, A> {
            ReaderT::new(move |env: R| {
                let m_reader_t_a: MKind::Of<ReaderT<R, MKind, A>> = (mma.run_reader_t)(env.clone()); // Applied to Of

                let m_a: MKind::Of<A> = // Applied to Of
                    <MKind as monad_kind::Bind<ReaderT<R, MKind, A>, A>>::bind(
                        m_reader_t_a,
                        move |inner_reader_t: ReaderT<R, MKind, A>| {
                            (inner_reader_t.run_reader_t)(env.clone())
                        },
                    );
                m_a
            })
        }
    }

    /// Trait for monads that can access a read-only environment `REnv`.
    ///
    /// # Type Parameters
    /// - `REnv`: The type of the environment.
    /// - `AVal`: The type of the value produced by computations in this monad.
    /// - `MKind`: The Kind marker for the inner monad (if `Self` is a transformer like `ReaderT`).
    ///
    /// This Kind-based version is specific to `ReaderT`. A more general `MonadReader` might be
    /// generic over `Self` directly if `Self` is the Kind marker of the reader-like monad.
    pub trait MonadReader<REnv, AVal, MKind: Kind1>
    where
        Self: Sized,
    {
        /// Retrieves the environment `REnv` from the context.
        ///
        /// The result is wrapped in the `ReaderT` structure, e.g., `ReaderT<REnv, MKind, REnv>`.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::reader::kind::{ReaderT, ReaderTKind, MonadReader};
        /// use monadify::kind_based::kind::OptionKind;
        ///
        /// #[derive(Clone, PartialEq, Debug)]
        /// struct MyConfig { id: i32 }
        /// type ConfigReader<A> = ReaderT<MyConfig, OptionKind, A>;
        /// type ConfigReaderKind = ReaderTKind<MyConfig, OptionKind>;
        ///
        /// // Specify the type REnv and AVal for ask, which are both MyConfig here.
        /// let get_config: ConfigReader<MyConfig> = <ConfigReaderKind as MonadReader<MyConfig, MyConfig, OptionKind>>::ask();
        /// let env = MyConfig { id: 123 };
        /// assert_eq!((get_config.run_reader_t)(env.clone()), Some(env));
        /// ```
        #[must_use]
        fn ask() -> ReaderT<REnv, MKind, REnv>
        where
            REnv: Clone + 'static,
            MKind: applicative_kind::Applicative<REnv> + 'static,
            MKind::Of<REnv>: 'static; // Changed Applied to Of

        /// Executes a computation in a modified environment.
        ///
        /// # Parameters
        /// - `map_env_fn`: A function `REnv -> REnv` that transforms the current environment.
        /// - `computation`: The `ReaderT` computation to run with the modified environment.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::reader::kind::{ReaderT, ReaderTKind, MonadReader};
        /// use monadify::kind_based::kind::OptionKind;
        /// use monadify::applicative::kind::Applicative; // For pure
        /// use monadify::functor::kind::Functor; // For map
        ///
        /// #[derive(Clone, PartialEq, Debug)]
        /// struct MyConfig { prefix: String, value: i32 }
        /// type ConfigReader<A> = ReaderT<MyConfig, OptionKind, A>;
        /// type ConfigReaderKind = ReaderTKind<MyConfig, OptionKind>;
        ///
        /// let initial_config = MyConfig { prefix: "Value: ".to_string(), value: 10 };
        ///
        /// let get_value_str_op: ConfigReader<MyConfig> = <ConfigReaderKind as MonadReader<MyConfig, MyConfig, OptionKind>>::ask();
        /// let get_value_str: ConfigReader<String> = <ConfigReaderKind as Functor<MyConfig, String>>::map(
        ///     get_value_str_op,
        ///     |cfg: MyConfig| format!("{}{}", cfg.prefix, cfg.value)
        /// );
        ///
        /// let modified_computation = ConfigReaderKind::local(
        ///     |mut cfg: MyConfig| { cfg.prefix = "New Value: ".to_string(); cfg.value = 20; cfg },
        ///     get_value_str.clone()
        /// );
        ///
        /// assert_eq!((get_value_str.run_reader_t)(initial_config.clone()), Some("Value: 10".to_string()));
        /// assert_eq!((modified_computation.run_reader_t)(initial_config.clone()), Some("New Value: 20".to_string()));
        /// ```
        #[must_use]
        fn local<FMapEnv>(
            map_env_fn: FMapEnv,
            computation: ReaderT<REnv, MKind, AVal>,
        ) -> ReaderT<REnv, MKind, AVal>
        where
            REnv: 'static,
            AVal: 'static,
            MKind: 'static,
            MKind::Of<AVal>: 'static, // Changed Applied to Of
            FMapEnv: Fn(REnv) -> REnv + 'static;
    }

    impl<R, MKindImpl, A> MonadReader<R, A, MKindImpl> for ReaderTKind<R, MKindImpl>
    // Renamed ReaderTHKTMarker, MMarkerImpl to MKindImpl
    where
        R: 'static,
        A: 'static,
        MKindImpl: Kind1 + 'static, // MKindImpl is the Kind marker for the inner monad. HKT1 to Kind1
        MKindImpl::Of<A>: 'static,  // M<A>. Applied to Of
    {
        fn ask() -> ReaderT<R, MKindImpl, R>
        where
            R: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<R> + 'static,
            MKindImpl::Of<R>: 'static, // Changed Applied to Of
        {
            ReaderT::new(move |env: R| MKindImpl::pure(env.clone()))
        }

        fn local<FMapEnv>(
            map_env_fn: FMapEnv,
            computation: ReaderT<R, MKindImpl, A>,
        ) -> ReaderT<R, MKindImpl, A>
        where
            FMapEnv: Fn(R) -> R + 'static,
        {
            let computation_run = computation.run_reader_t.clone();
            ReaderT::new(move |current_env: R| {
                let modified_env = map_env_fn(current_env);
                computation_run(modified_env)
            })
        }
    }

    impl<R, MKindImpl: Kind1> ReaderTKind<R, MKindImpl> {
        /// Retrieves the environment as the value — the ergonomic concrete form of
        /// [`MonadReader::ask`]. The generic `MonadReader` trait remains for code
        /// generic over the inner monad.
        ///
        /// # Example
        /// ```
        /// use monadify::transformers::reader::kind::{ReaderT, ReaderTKind};
        /// use monadify::kind_based::kind::OptionKind;
        ///
        /// #[derive(Clone, PartialEq, Debug)]
        /// struct Cfg { id: i32 }
        /// type CfgReaderKind = ReaderTKind<Cfg, OptionKind>;
        ///
        /// let get_env: ReaderT<Cfg, OptionKind, Cfg> = CfgReaderKind::ask();
        /// assert_eq!((get_env.run_reader_t)(Cfg { id: 7 }), Some(Cfg { id: 7 }));
        /// ```
        #[must_use]
        pub fn ask() -> ReaderT<R, MKindImpl, R>
        where
            R: Clone + 'static,
            MKindImpl: applicative_kind::Applicative<R> + 'static,
            MKindImpl::Of<R>: 'static,
        {
            <Self as MonadReader<R, R, MKindImpl>>::ask()
        }
    }
}

// Directly export Kind-based versions
pub use kind::{MonadReader, Reader, ReaderT, ReaderTKind}; // Renamed ReaderTHKTMarker
