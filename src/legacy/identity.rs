// Content from the original classic module in src/identity.rs
use crate::function::RcFn;
use crate::legacy::applicative::Applicative;
use crate::legacy::apply::Apply;
use crate::legacy::functor::Functor;
use crate::legacy::monad::{Bind, Monad};

/// Legacy version of the `Identity` monad.
///
/// `Identity<A>` simply wraps a value of type `A`. It is the simplest monad,
/// demonstrating the monadic structure without adding any computational context
/// other than the wrapping itself.
///
/// Its `map`, `apply`, and `bind` operations essentially unwrap the value,
/// apply the function, and rewrap the result in `Identity`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity<A>(pub A);

impl<A> Functor<A> for Identity<A> {
    type Functor<BVal> = Identity<BVal>;
    fn map<B, F>(self, mut f: F) -> Self::Functor<B>
    where
        F: FnMut(A) -> B + 'static,
    {
        Identity(f(self.0))
    }
}

impl<A: 'static> Apply<A> for Identity<A> {
    type Apply<T> = Identity<T>;
    type Fnn<T, U> = RcFn<T, U>;
    fn apply<B>(self, i: Identity<RcFn<A, B>>) -> Identity<B>
    where
        Self: Sized,
    {
        Identity(i.0.call(self.0))
    }
}

impl<A: 'static> Applicative<A> for Identity<A> {
    type Applicative<T> = Identity<T>;
    fn pure(v: A) -> Self::Applicative<A> {
        Identity(v)
    }
}

impl<A: 'static> Bind<A> for Identity<A> {
    type Bind<T> = Identity<T>;
    fn bind<B, F>(self, f: F) -> Self::Bind<B>
    where
        F: Fn(A) -> Self::Bind<B> + 'static,
    {
        f(self.0)
    }
}

impl<A: 'static> Monad<A> for Identity<A> {}
