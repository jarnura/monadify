//! # Identity Monad for the `monadify` library
// Kind-based version is now default.

pub mod kind {
    // Renamed from hkt to kind
    //! # Kind-based Identity Monad
    //!
    //! This module provides the Kind-based implementation of the `Identity` monad.
    //! The `Identity` monad is the simplest monad; it just wraps a value without
    //! adding any computational context beyond the wrapping itself.
    //!
    //! It's often used as a base for monad transformers or to make non-monadic
    //! code fit into a monadic interface within the `monadify` library.
    //!
    //! ## Key Components
    //! - [`Identity<A>`]: The wrapper struct holding a value of type `A`.
    //! - [`IdentityKind`]: The Kind marker for `Identity`.
    //!
    //! ## Example
    //! ```
    //! use monadify::identity::kind::{Identity, IdentityKind};
    //! use monadify::functor::kind::Functor;
    //! use monadify::applicative::kind::Applicative;
    //! use monadify::monad::kind::{Bind, Monad};
    //!
    //! // Pure (from Applicative)
    //! let id_val: Identity<i32> = IdentityKind::pure(10);
    //! assert_eq!(id_val, Identity(10));
    //!
    //! // Map (from Functor)
    //! let mapped_id: Identity<String> = IdentityKind::map(id_val, |x| (x * 2).to_string());
    //! assert_eq!(mapped_id, Identity("20".to_string()));
    //!
    //! // Bind
    //! let bound_id: Identity<f64> = IdentityKind::bind(mapped_id, |s| Identity(s.len() as f64));
    //! assert_eq!(bound_id, Identity(2.0));
    //!
    //! // Join (from Monad)
    //! let nested_id: Identity<Identity<i32>> = Identity(Identity(100));
    //! let joined_id: Identity<i32> = IdentityKind::join(nested_id);
    //! assert_eq!(joined_id, Identity(100));
    //! ```

    use crate::applicative::kind as applicative_kind; // Renamed hkt to kind
    use crate::apply::kind as apply_kind; // Renamed hkt to kind
    use crate::function::RcFn;
    use crate::functor::kind as functor_kind; // Renamed hkt to kind
    use crate::kind_based::kind::Kind; // Changed HKT to Kind
    use crate::monad::kind as monad_kind; // Renamed hkt to kind // For Apply's function container

    /// A simple wrapper struct that holds a value of type `A`.
    ///
    /// This is the core data type for the `Identity` monad. It doesn't add
    /// any special context other than simply containing the value.
    #[must_use]
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Identity<A>(pub A);

    /// The Kind marker for the `Identity` monad.
    ///
    /// This unit struct is used to implement the Kind traits (`Functor`, `Apply`,
    /// `Applicative`, `Monad`, `Bind`) for `Identity`.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct IdentityKind; // Renamed from IdentityHKTMarker

    impl Kind for IdentityKind {
        // Changed HKT to Kind, IdentityHKTMarker to IdentityKind
        type Of<T> = Identity<T>; // Changed Applied to Of
    }
    // Kind1 is implemented by the blanket impl in kind_based/kind.rs for types that impl Kind.

    // Kind-based Functor for IdentityKind
    impl<A, B> functor_kind::Functor<A, B> for IdentityKind
    // Changed IdentityHKTMarker to IdentityKind
    // No 'static bounds needed here for A, B for basic map
    {
        /// Applies a function to the value inside `Identity`.
        /// `Identity(a)` becomes `Identity(func(a))`.
        fn map(input: Identity<A>, mut func: impl FnMut(A) -> B) -> Identity<B> {
            Identity(func(input.0))
        }
    }

    // Kind-based Apply for IdentityKind
    impl<A, B> apply_kind::Apply<A, B> for IdentityKind
    where
        A: 'static, // Required by Apply trait definition
        B: 'static, // Required by Apply trait definition
    {
        /// Applies a wrapped function `Identity<RcFn<A, B>>` to a wrapped value `Identity<A>`.
        fn apply(
            value_container: Identity<A>,
            function_container: Identity<RcFn<A, B>>,
        ) -> Identity<B> {
            Identity(function_container.0.call(value_container.0))
        }
    }

    // Kind-based Applicative for IdentityKind
    impl<T: 'static> applicative_kind::Applicative<T> for IdentityKind
    // Changed IdentityHKTMarker to IdentityKind
    // T: 'static is already on the Applicative trait
    {
        /// Lifts a value `T` into `Identity<T>`.
        fn pure(value: T) -> Identity<T> {
            // Self::Of<T> is Identity<T>
            Identity(value)
        }
    }

    // Kind-based Bind for IdentityKind
    impl<A, B> monad_kind::Bind<A, B> for IdentityKind
    // Changed IdentityHKTMarker to IdentityKind
    where
        A: 'static, // Required by Bind trait definition
        B: 'static, // Required by Bind trait definition
    {
        /// Applies a function `A -> Identity<B>` to the value inside `Identity<A>`.
        /// Effectively, it unwraps the value, applies the function, and returns the new `Identity<B>`.
        fn bind(input: Identity<A>, mut func: impl FnMut(A) -> Identity<B>) -> Identity<B> {
            // Self::Of<B> is Identity<B>
            func(input.0)
        }
    }

    // Kind-based Monad for IdentityKind
    impl<A: 'static> monad_kind::Monad<A> for IdentityKind {
        // Changed IdentityHKTMarker to IdentityKind
        /// Flattens `Identity<Identity<A>>` to `Identity<A>`.
        /// This simply unwraps the outer `Identity`.
        fn join(mma: Identity<Identity<A>>) -> Identity<A> {
            // Self::Of<Self::Of<A>> is Identity<Identity<A>>
            // mma is Identity(Identity(value))
            // mma.0 is Identity(value)
            // So, we return mma.0 directly. This is Self::Of<A>.
            mma.0
        }
    }
}

// Directly export Kind-based Identity and its marker
pub use kind::{Identity, IdentityKind}; // Renamed from hkt and IdentityHKTMarker

// Note: Kind1 is imported within the kind module if needed, or used via crate::kind_based::kind::Kind1.
