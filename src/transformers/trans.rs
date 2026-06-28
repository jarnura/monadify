//! # `MonadTrans`: lifting inner computations into a transformer
//!
//! A monad transformer `T m` adds an effect on top of an inner monad `m`. The
//! [`MonadTrans`] trait captures the single operation every transformer shares:
//! [`lift`](MonadTrans::lift), which embeds an inner computation `m a` into the
//! transformed monad `T m a` while adding *no* effect of its own.
//!
//! This crate implements `MonadTrans` for all four transformers:
//! [`ReaderTKind`],
//! [`StateTKind`],
//! [`WriterTKind`], and
//! [`ExceptTKind`].
//!
//! ## Law
//!
//! `lift` is a monad morphism — it must commute with `pure` and `bind`:
//! ```text
//! lift(m.pure(a))            == T::pure(a)
//! lift(m.bind(x, k))         == T::bind(lift(x), |a| lift(k(a)))
//! ```
//! Intuitively: lifting a pure inner value is the transformer's own `pure`, and
//! lifting a sequence of inner steps is the same as lifting each and sequencing
//! in the transformer.
//!
//! ## Example
//! ```
//! use monadify::transformers::writer::{Writer, WriterTKind};
//! use monadify::{Applicative, Identity, IdentityKind};
//!
//! // Lift a pure inner `Identity(7)` into a `Writer<String, _>` — empty log.
//! type W<A> = Writer<String, A>;
//! let lifted: W<i32> = WriterTKind::<String, IdentityKind>::lift(
//!     IdentityKind::pure(7),
//! );
//! let Identity((v, log)) = lifted.run_writer_t;
//! assert_eq!(v, 7);
//! assert_eq!(log, ""); // lifting adds no log
//! ```

use crate::functor::kind as functor_kind;
use crate::kind_based::kind::{Kind, Kind1};
use crate::monoid::Monoid;
use crate::transformers::except::kind::{ExceptT, ExceptTKind};
use crate::transformers::reader::kind::{ReaderT, ReaderTKind};
use crate::transformers::state::kind::{StateT, StateTKind};
use crate::transformers::writer::kind::{WriterT, WriterTKind};

/// Lifts an inner monadic computation into a monad transformer.
///
/// `Self` is the transformer's Kind marker (e.g.
/// [`WriterTKind`]); `MKind` is the
/// inner monad's marker. [`lift`](Self::lift) maps `MKind::Of<A>` to
/// `Self::Of<A>`, adding none of the transformer's own effect (an empty log,
/// an unchanged state, an ignored environment).
///
/// # Type Parameters
/// - `A`: the value type of the lifted computation.
/// - `MKind`: the Kind marker of the inner monad.
pub trait MonadTrans<A, MKind: Kind1>: Kind {
    /// Embeds `inner: MKind::Of<A>` into the transformer `Self::Of<A>`.
    #[must_use]
    fn lift(inner: MKind::Of<A>) -> Self::Of<A>;
}

// `ReaderT`: ignore the environment, yielding the inner computation as-is.
// The carrier is a `Fn(R) -> M::Of<A>` that may run repeatedly, so the inner
// value must be `Clone`.
impl<R, MKind, A> MonadTrans<A, MKind> for ReaderTKind<R, MKind>
where
    R: 'static,
    A: 'static,
    MKind: Kind1 + 'static,
    MKind::Of<A>: Clone + 'static,
{
    fn lift(inner: MKind::Of<A>) -> ReaderT<R, MKind, A> {
        ReaderT::new(move |_r: R| inner.clone())
    }
}

// `StateT`: thread the state through unchanged, pairing the inner value with it.
// The carrier runs per starting state, so the inner value must be `Clone`.
impl<S, MKind, A> MonadTrans<A, MKind> for StateTKind<S, MKind>
where
    S: Clone + 'static,
    A: 'static,
    MKind: functor_kind::Functor<A, (A, S)> + Kind1 + 'static,
    MKind::Of<A>: Clone + 'static,
    MKind::Of<(A, S)>: 'static,
{
    fn lift(inner: MKind::Of<A>) -> StateT<S, MKind, A> {
        StateT::new(move |s: S| {
            <MKind as functor_kind::Functor<A, (A, S)>>::map(inner.clone(), move |a| (a, s.clone()))
        })
    }
}

// `WriterT`: pair the inner value with an empty log. No `Clone` needed — the
// inner value is consumed exactly once.
impl<W, MKind, A> MonadTrans<A, MKind> for WriterTKind<W, MKind>
where
    W: Monoid + 'static,
    A: 'static,
    MKind: functor_kind::Functor<A, (A, W)> + Kind1 + 'static,
    MKind::Of<A>: 'static,
    MKind::Of<(A, W)>: 'static,
{
    fn lift(inner: MKind::Of<A>) -> WriterT<W, MKind, A> {
        let paired: MKind::Of<(A, W)> =
            <MKind as functor_kind::Functor<A, (A, W)>>::map(inner, move |a| (a, W::empty()));
        WriterT::new(paired)
    }
}

// `ExceptT`: wrap the inner value on the success branch with `Ok`. No `Clone`
// needed — the inner value is consumed exactly once (like `WriterT`).
impl<E, MKind, A> MonadTrans<A, MKind> for ExceptTKind<E, MKind>
where
    E: 'static,
    A: 'static,
    MKind: functor_kind::Functor<A, Result<A, E>> + Kind1 + 'static,
    MKind::Of<A>: 'static,
    MKind::Of<Result<A, E>>: 'static,
{
    fn lift(inner: MKind::Of<A>) -> ExceptT<E, MKind, A> {
        let wrapped: MKind::Of<Result<A, E>> =
            <MKind as functor_kind::Functor<A, Result<A, E>>>::map(inner, Ok);
        ExceptT::new(wrapped)
    }
}

// ── Inherent ergonomic `lift` helpers ──────────────────────────────────────
//
// These delegate to the `MonadTrans` trait and expose the concrete shorthand
// `RKind::lift(inner)` instead of the verbose UFCS form.  The trait impls
// remain the canonical implementation and are still the right choice for
// code that is generic over the transformer marker.

impl<R, MKind: Kind1> ReaderTKind<R, MKind> {
    /// Ergonomic inherent form of [`MonadTrans::lift`] for [`ReaderT`].
    ///
    /// Embeds an inner computation `MKind::Of<A>` into a `ReaderT<R, MKind, A>`,
    /// ignoring the environment and adding no effect of its own. This is the
    /// concrete shorthand for
    /// `<ReaderTKind<R, MKind> as MonadTrans<A, MKind>>::lift(inner)`;
    /// the trait form remains available for generic code.
    #[must_use]
    pub fn lift<A>(inner: MKind::Of<A>) -> ReaderT<R, MKind, A>
    where
        R: 'static,
        A: 'static,
        MKind: 'static,
        MKind::Of<A>: Clone + 'static,
    {
        <Self as MonadTrans<A, MKind>>::lift(inner)
    }
}

impl<S, MKind: Kind1> StateTKind<S, MKind> {
    /// Ergonomic inherent form of [`MonadTrans::lift`] for [`StateT`].
    ///
    /// Embeds an inner computation `MKind::Of<A>` into a `StateT<S, MKind, A>`,
    /// threading the state through unchanged and adding no effect of its own.
    /// This is the concrete shorthand for
    /// `<StateTKind<S, MKind> as MonadTrans<A, MKind>>::lift(inner)`;
    /// the trait form remains available for generic code.
    #[must_use]
    pub fn lift<A>(inner: MKind::Of<A>) -> StateT<S, MKind, A>
    where
        S: Clone + 'static,
        A: 'static,
        MKind: functor_kind::Functor<A, (A, S)> + 'static,
        MKind::Of<A>: Clone + 'static,
        MKind::Of<(A, S)>: 'static,
    {
        <Self as MonadTrans<A, MKind>>::lift(inner)
    }
}

impl<W, MKind: Kind1> WriterTKind<W, MKind> {
    /// Ergonomic inherent form of [`MonadTrans::lift`] for [`WriterT`].
    ///
    /// Embeds an inner computation `MKind::Of<A>` into a `WriterT<W, MKind, A>`,
    /// pairing the value with an empty log and adding no effect of its own. This
    /// is the concrete shorthand for
    /// `<WriterTKind<W, MKind> as MonadTrans<A, MKind>>::lift(inner)`;
    /// the trait form remains available for generic code.
    #[must_use]
    pub fn lift<A>(inner: MKind::Of<A>) -> WriterT<W, MKind, A>
    where
        W: Monoid + 'static,
        A: 'static,
        MKind: functor_kind::Functor<A, (A, W)> + 'static,
        MKind::Of<A>: 'static,
        MKind::Of<(A, W)>: 'static,
    {
        <Self as MonadTrans<A, MKind>>::lift(inner)
    }
}

impl<E, MKind: Kind1> ExceptTKind<E, MKind> {
    /// Ergonomic inherent form of [`MonadTrans::lift`] for [`ExceptT`].
    ///
    /// Embeds an inner computation `MKind::Of<A>` into an `ExceptT<E, MKind, A>`,
    /// wrapping the value on the success (`Ok`) branch and adding no effect of
    /// its own. This is the concrete shorthand for
    /// `<ExceptTKind<E, MKind> as MonadTrans<A, MKind>>::lift(inner)`;
    /// the trait form remains available for generic code.
    #[must_use]
    pub fn lift<A>(inner: MKind::Of<A>) -> ExceptT<E, MKind, A>
    where
        E: 'static,
        A: 'static,
        MKind: functor_kind::Functor<A, Result<A, E>> + 'static,
        MKind::Of<A>: 'static,
        MKind::Of<Result<A, E>>: 'static,
    {
        <Self as MonadTrans<A, MKind>>::lift(inner)
    }
}
