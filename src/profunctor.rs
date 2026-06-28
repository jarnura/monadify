use std::{marker::PhantomData, ops::Deref};

use crate::function::{CFnOnce, RcFn};

/// A `Profunctor` is a bifunctor that is contravariant in its first type parameter
/// and covariant in its second type parameter.
///
/// If `P<B, C>` is a profunctor, `dimap` allows you to change its input from `A` to `B`
/// using a function `A -> B`, and its output from `C` to `D` using a function `C -> D`.
/// So, `P<B, C>` can be transformed into `P<A, D>`.
///
/// This is often useful for types that represent transformations or functions,
/// like `fn(B) -> C`. `dimap(f, g)` on `h: B -> C` would produce `a -> g(h(f(a)))`.
///
/// Laws:
/// 1. `dimap(id, id) == id`
/// 2. `dimap(f . g, h . i) == dimap(g, h) . dimap(f, i)`
pub trait Profunctor<B, C> {
    /// The associated type representing the structure of the Profunctor.
    /// For example, if `Self` is `RcFn<B, C>`, then `Pro<T, U>` would be `RcFn<T, U>`.
    type Pro<T, U>;

    /// Maps the input and output of the profunctor.
    ///
    /// Given `self` of type `P<B, C>`, a function `a2b: A -> B`, and a function `c2d: C -> D`,
    /// `dimap` produces a new profunctor of type `Self::Pro<A, D>`.
    ///
    /// The function `a2b` pre-processes the input (contravariant mapping).
    /// The function `c2d` post-processes the output (covariant mapping).
    ///
    /// # Parameters
    /// - `self`: The profunctor instance `P<B, C>`.
    /// - `a2b`: A function `A -> B` to transform the input type from `A` to `B`.
    /// - `c2d`: A function `C -> D` to transform the output type from `C` to `D`.
    ///
    /// # Returns
    /// A new profunctor instance `Self::Pro<A, D>`.
    #[must_use]
    fn dimap<A, D, A2B, C2D>(self, a2b: A2B, c2d: C2D) -> Self::Pro<A, D>
    where
        A2B: Fn(A) -> B + 'static,
        C2D: Fn(C) -> D + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static;
}

/// `RcFn<B, C>` (a shared-ownership function `B -> C`) as a `Profunctor`.
///
/// `dimap` on `h: RcFn<B,C>` with `f: A->B` and `g: C->D` results in a new
/// `RcFn<A,D>` that computes `a -> g(h(f(a)))`.
impl<B, C> Profunctor<B, C> for RcFn<B, C> {
    type Pro<T, U> = RcFn<T, U>;
    fn dimap<A, D, A2B, C2D>(self, a2b: A2B, c2d: C2D) -> Self::Pro<A, D>
    where
        A2B: Fn(A) -> B + 'static,
        C2D: Fn(C) -> D + 'static,
        C: 'static,
        B: 'static,
        A: 'static,
        D: 'static,
    {
        // self is h: B -> C
        // a2b is f: A -> B
        // c2d is g: C -> D
        // Result is a function A -> D: a |-> g(h(f(a)))
        RcFn::new(move |a: A| c2d(self.call(a2b(a))))
    }
}

/// `CFnOnce<B, C>` (a boxed, once-callable function `B -> C`) as a `Profunctor`.
///
/// Similar to `RcFn`, `dimap` on `h: CFnOnce<B,C>` with `f: A->B` and `g: C->D`
/// results in a new `CFnOnce<A,D>` that computes `a -> g(h(f(a)))`.
impl<B, C> Profunctor<B, C> for CFnOnce<B, C> {
    type Pro<T, U> = CFnOnce<T, U>;
    fn dimap<A, D, A2B, C2D>(self, a2b: A2B, c2d: C2D) -> Self::Pro<A, D>
    where
        A2B: Fn(A) -> B + 'static,
        C2D: Fn(C) -> D + 'static,
        C: 'static,
        B: 'static,
        A: 'static,
        D: 'static,
    {
        CFnOnce::new(move |a: A| c2d(self.call_once(a2b(a))))
    }
}

/// `Strong` profunctors are profunctors that can operate on product types (tuples).
///
/// `Strong` extends `Profunctor` with `first` and `second` methods.
/// - `first`: Given `P<A, B>`, produces `P<(A, C), (B, C)>`. It processes the first
///   element of a pair, leaving the second untouched.
/// - `second`: Given `P<A, B>`, produces `P<(C, A), (C, B)>`. It processes the second
///   element of a pair, leaving the first untouched.
///
/// This is particularly useful for optics like Lenses.
pub trait Strong<A, B>: Profunctor<A, B> {
    /// Adapts the profunctor to operate on the first component of a pair.
    /// If `self` is `P<A,B>`, `first` returns `P<(A,C), (B,C)>`.
    #[must_use]
    fn first<C: 'static>(self) -> Self::Pro<(A, C), (B, C)>;

    /// Adapts the profunctor to operate on the second component of a pair.
    /// If `self` is `P<A,B>`, `second` returns `P<(C,A), (C,B)>`.
    #[must_use]
    fn second<C: 'static>(self) -> Self::Pro<(C, A), (C, B)>;
}

/// `RcFn<A, B>` as a `Strong` profunctor.
impl<A: 'static, B: 'static> Strong<A, B> for RcFn<A, B> {
    /// If `self` is `f: A -> B`, `first` returns `((A,C)) -> (B,C)`
    /// where the new function is `(a,c) -> (f(a), c)`.
    fn first<C: 'static>(self) -> Self::Pro<(A, C), (B, C)> {
        RcFn::new(move |(a, c)| (self.call(a), c))
    }

    /// If `self` is `f: A -> B`, `second` returns `((C,A)) -> (C,B)`
    /// where the new function is `(c,a) -> (c, f(a))`.
    fn second<C: 'static>(self) -> Self::Pro<(C, A), (C, B)> {
        RcFn::new(move |(c, a)| (c, self.call(a)))
    }
}

/// `Choice` profunctors are profunctors that can operate on sum types (`Result`).
///
/// `Choice` extends `Profunctor` with `left` and `right` methods.
/// - `left`: Given `P<A, B>`, produces `P<Result<C, A>, Result<C, B>>`. It processes
///   the `Err` part of a `Result`, leaving `Ok` untouched.
/// - `right`: Given `P<A, B>`, produces `P<Result<A, C>, Result<B, C>>`. It processes
///   the `Ok` part of a `Result`, leaving `Err` untouched.
///
/// This is useful for optics like Prisms.
pub trait Choice<A, B>: Profunctor<A, B> {
    /// Adapts the profunctor to operate on the `Err` variant of a `Result`.
    /// If `self` is `P<A,B>`, `left` returns `P<Result<C,A>, Result<C,B>>`.
    #[must_use]
    fn left<C>(self) -> Self::Pro<Result<C, A>, Result<C, B>>;

    /// Adapts the profunctor to operate on the `Ok` variant of a `Result`.
    /// If `self` is `P<A,B>`, `right` returns `P<Result<A,C>, Result<B,C>>`.
    #[must_use]
    fn right<C>(self) -> Self::Pro<Result<A, C>, Result<B, C>>;
}

/// `RcFn<A, B>` as a `Choice` profunctor.
impl<A: 'static, B: 'static> Choice<A, B> for RcFn<A, B> {
    /// If `self` is `f: A -> B`, `left` returns `Result<C,A> -> Result<C,B>`.
    /// If input is `Ok(c)`, it remains `Ok(c)`.
    /// If input is `Err(a)`, it becomes `Err(f(a))`.
    fn left<C>(self) -> Self::Pro<Result<C, A>, Result<C, B>> {
        RcFn::new(move |r: Result<C, A>| match r {
            Ok(c) => Ok(c),
            Err(a) => Err(self.call(a)),
        })
    }

    /// If `self` is `f: A -> B`, `right` returns `Result<A,C> -> Result<B,C>`.
    /// If input is `Ok(a)`, it becomes `Ok(f(a))`.
    /// If input is `Err(c)`, it remains `Err(c)`.
    fn right<C>(self) -> Self::Pro<Result<A, C>, Result<B, C>> {
        RcFn::new(move |r: Result<A, C>| match r {
            Ok(a) => Ok(self.call(a)),
            Err(c) => Err(c),
        })
    }
}

/// Represents a general Optic using Profunctor encoding.
///
/// An Optic `Optic<POuter, PInner, S, T, A, B>` transforms an "inner" profunctor `PInner<A, B>`
/// into an "outer" profunctor `POuter<S, T>`.
///
/// - `S`: Source type of the outer structure (e.g., the whole data structure).
/// - `T`: Target type of the outer structure (after modification).
/// - `A`: Source type of the inner part (e.g., a field).
/// - `B`: Target type of the inner part (after modification).
/// - `PInner`: The type of profunctor focused on the part `A -> B`.
/// - `POuter`: The type of profunctor focused on the whole `S -> T`.
///
/// The `optic` field holds a boxed closure that performs this transformation.
pub struct Optic<POuter: Profunctor<S, T>, PInner: Profunctor<A, B>, S, T, A, B> {
    /// The function that transforms an inner profunctor to an outer profunctor.
    /// `Box<dyn FnOnce(PInner) -> POuter>`
    pub optic: Box<dyn FnOnce(PInner) -> POuter>,
    _marker: PhantomData<(S, T, A, B)>,
}

/// A `Lens` is a type of Optic that focuses on a part `A` of a whole `S`,
/// allowing both getting the part and setting it (which might change `S` to `T`
/// and `A` to `B`).
///
/// Lenses are typically built from `Strong` profunctors.
/// This `Lens` struct wraps an `Optic`.
pub struct Lens<PO: Strong<S, T>, PI: Strong<A, B>, S, T, A, B>(
    /// The underlying `Optic` representation of the lens.
    pub Optic<PO, PI, S, T, A, B>,
);

impl<PO: Strong<S, T>, PI: Strong<A, B>, S, T, A, B> Deref for Lens<PO, PI, S, T, A, B> {
    type Target = Optic<PO, PI, S, T, A, B>;
    fn deref(&self) -> &Optic<PO, PI, S, T, A, B> {
        &self.0
    }
}

/// A `Fold` is an Optic that can extract multiple pieces of data `A` from a structure `S`,
/// and fold them using a monoid. It's a generalization of a Getter.
///
/// Here, it's defined using `Optic` with `Forget` profunctors.
/// `Forget<R, X, Y>` is a profunctor that "forgets" its `Y` type parameter and
/// maps its `X` input to a monoidal result `R`.
///
/// - `R`: The result type of the fold (often the type `A` itself, or a monoid built from `A`).
/// - `S`, `T`: Outer structure types (T is often phantom for folds).
/// - `A`, `B`: Inner part types (B is often phantom for folds).
pub type Fold<R, S, T, A, B> = Optic<Forget<R, S, T>, Forget<R, A, B>, S, T, A, B>;

/// An `AGetter` (or simply `Getter`) is a specialized `Fold` that extracts a single part `A`
/// from a structure `S`.
///
/// It's a `Fold<A, S, T, A, B>`, meaning the fold result `R` is the same as the part type `A`.
/// `T` and `B` are typically phantom types for getters.
pub type AGetter<S, T, A, B> = Fold<A, S, T, A, B>;

impl<PA: Strong<S, T>, PB: Strong<A, B>, S: 'static, T: 'static, A: 'static, B: 'static>
    From<Lens<PA, PB, S, T, A, B>> for Optic<PA, PB, S, T, A, B>
{
    fn from(value: Lens<PA, PB, S, T, A, B>) -> Self {
        value.0
    }
}

/// Extracts a value `A` from a structure `S` using a `AGetter`.
///
/// # Parameters
/// - `getter`: The `AGetter<S, T, A, B>` used to focus on the part `A`.
///   `T` and `B` are typically phantom.
/// - `s`: The structure `S` from which to extract the part.
///
/// # Returns
/// The extracted part `A`.
#[must_use]
pub fn view<S: 'static, T: 'static, A: 'static, B: 'static>(
    getter: AGetter<S, T, A, B>,
    s: S,
) -> A {
    // To view with a getter (which is Optic<Forget<A,S,T>, Forget<A,A,B>>),
    // we provide an "inner" profunctor Forget<A,A,B>.
    // Forget<A,A,B> needs a function A -> A (identity) for its `inner` field.
    let inner_profunctor = Forget {
        inner: RcFn::new(|x: A| x), // Identity function A -> A
        _forget: PhantomData,       // Phantom for B
    };
    // Applying the getter's optic transformation:
    // (getter.optic)(Forget<A,A,B>) results in Forget<A,S,T>.
    // This Forget<A,S,T> has an `inner` field which is RcFn<S,A>.
    // Calling this RcFn<S,A> with `s` gives the result `A`.
    (getter.optic)(inner_profunctor).inner.call(s)
}

/// A `Profunctor` that "forgets" its second type parameter (`BPhantom`) and maps its
/// first type parameter (`AInput`) to a fixed result type `R`.
///
/// `Forget<R, AInput, BPhantom>` takes `AInput -> R` as its core.
/// When used in `dimap(f: X -> AInput, g: BPhantom -> Y)`, it effectively becomes
/// `Forget<R, X, Y>` where the new core function is `x -> original_core(f(x))`.
/// The `g` function and `Y` type are ignored.
///
/// This is used in defining `Fold` and `Getter` optics.
pub struct Forget<R, AInput, BPhantom> {
    /// The function `AInput -> R` that Forget wraps.
    pub inner: RcFn<AInput, R>,
    _forget: PhantomData<BPhantom>,
}

// AInput here is ProfB from Profunctor<ProfB, ProfC>
// BPhantom here is ProfC from Profunctor<ProfB, ProfC>
impl<R: 'static, ProfB: 'static, ProfC> Profunctor<ProfB, ProfC> for Forget<R, ProfB, ProfC> {
    type Pro<NextA, NextC> = Forget<R, NextA, NextC>;

    // self: Forget<R, ProfB, ProfC>
    // a2b: ImplA -> ProfB
    // _c2d: ProfC -> ImplD (ignored)
    // Returns: Forget<R, ImplA, ImplD>
    fn dimap<ImplA: 'static, ImplD: 'static, A2B, C2D>(
        self,
        a2b: A2B,
        _c2d: C2D, // This function is ignored by Forget's dimap
    ) -> Self::Pro<ImplA, ImplD>
    where
        A2B: Fn(ImplA) -> ProfB + 'static,
        C2D: Fn(ProfC) -> ImplD + 'static,
    {
        // Original inner: ProfB -> R
        // New inner: ImplA -> R, via ImplA -> ProfB -> R
        Forget {
            inner: RcFn::new(a2b) >> self.inner, // Compose a2b with self.inner
            _forget: PhantomData,                // Phantom for ImplD
        }
    }
}

// AInput here is AStrong from Strong<AStrong, BStrong>
// BPhantom here is BStrong from Strong<AStrong, BStrong>
impl<R: 'static, AStrong: 'static, BStrong: 'static> Strong<AStrong, BStrong>
    for Forget<R, AStrong, BStrong>
{
    // self: Forget<R, AStrong, BStrong>
    // first<C: 'static> should return Pro<(AStrong, C), (BStrong, C)>
    // which is Forget<R, (AStrong, C), (BStrong, C)>
    fn first<C: 'static>(self) -> Self::Pro<(AStrong, C), (BStrong, C)> {
        // Original inner: AStrong -> R
        // New inner: (AStrong, C) -> R. We only care about AStrong.
        Forget {
            inner: RcFn::new(|(a, _c): (AStrong, C)| a) >> self.inner,
            _forget: PhantomData, // Phantom for (BStrong, C)
        }
    }

    // second<C: 'static> should return Pro<(C, AStrong), (C, BStrong)>
    // which is Forget<R, (C, AStrong), (C, BStrong)>
    fn second<C: 'static>(self) -> Self::Pro<(C, AStrong), (C, BStrong)> {
        // Original inner: AStrong -> R
        // New inner: (C, AStrong) -> R. We only care about AStrong.
        Forget {
            inner: RcFn::new(|(_c, a): (C, AStrong)| a) >> self.inner,
            _forget: PhantomData, // Phantom for (C, BStrong)
        }
    }
}

/// Internal helper to construct a Lens using the "van Laarhoven" representation.
/// Not typically called directly by users. `lens` function is preferred.
///
/// `to`: A function `S -> (A, RcFn<B, T>)`.
///   - Takes the whole structure `S`.
///   - Returns a pair:
///     - The focused part `A`.
///     - A function `RcFn<B, T>` that takes a new part `B` and reconstructs the whole `T`.
#[must_use]
pub fn lens_<PO, PI, PBC, S: 'static, T: 'static, A: 'static, B: 'static>(
    to: RcFn<S, (A, RcFn<B, T>)>,
) -> Lens<PO, PBC, S, T, A, B>
where
    PO: Strong<S, T>,
    PI: Profunctor<(A, RcFn<B, T>), (B, RcFn<B, T>), Pro<S, T> = PO>,
    PBC: Strong<A, B, Pro<(A, RcFn<B, T>), (B, RcFn<B, T>)> = PI>,
{
    let optics_fn = |pbc: PBC| {
        let pi_instance = PBC::first(pbc);
        PI::dimap(
            pi_instance,
            move |s_val| to.call(s_val), // S -> (A, RcFn<B,T>)
            |(b_val, f_b_to_t)| f_b_to_t.call(b_val), // (B, RcFn<B,T>) -> T
        )
    };
    Lens(Optic {
        optic: Box::new(optics_fn),
        _marker: PhantomData,
    })
}

/// Constructs a `Lens` from a getter function and a setter function.
///
/// # Parameters
/// - `s2a`: A "getter" function `RcFn<S, A>` that extracts the part `A` from the whole `S`.
/// - `s2b2t`: A "setter" function `RcFn<S, RcFn<B, T>>`. It takes the original whole `S`,
///   returns a new function that takes the new part `B` and produces the new whole `T`.
///   `S` must be `Copy` because it's used by both the getter and to capture in the setter closure.
///
/// # Returns
/// A `Lens<PO, PI, S, T, A, B>`. The profunctor types `PO` and `PI` are usually inferred.
#[must_use]
pub fn lens<PO, PI, S: Copy + 'static, T: 'static, A: 'static, B: 'static>(
    s2a: RcFn<S, A>,
    s2b2t: RcFn<S, RcFn<B, T>>,
) -> Lens<PO, PI, S, T, A, B>
where
    PO: Strong<S, T>,
    PI: Strong<A, B>,
    // This constraint links the result of PI.first.dimap to PO.
    <PI as Profunctor<A, B>>::Pro<(A, RcFn<B, T>), (B, RcFn<B, T>)>:
        Profunctor<(A, RcFn<B, T>), (B, RcFn<B, T>), Pro<S, T> = PO>,
{
    // Combine getter and setter into the S -> (A, RcFn<B, T>) form required by lens_
    let combined_fn = RcFn::new(move |s: S| {
        let part_a = s2a.call(s);
        let setter_fn_for_s = s2b2t.call(s); // This is RcFn<B,T>
        (part_a, setter_fn_for_s)
    });
    lens_(combined_fn)
}

/// A `Lens` that focuses on the first element of a pair `(A, C)`.
/// Allows getting `A` and setting `A` to `B`, resulting in `(B, C)`.
///
/// Type parameters `PA` and `PS` are the profunctor types involved, usually inferred.
#[must_use]
pub fn _1<
    A: 'static,
    B: 'static,
    C: 'static,
    PA: Strong<A, B, Pro<(A, C), (B, C)> = PS> + 'static,
    PS: Strong<(A, C), (B, C)>,
>() -> Lens<PS, PA, (A, C), (B, C), A, B> {
    Lens(Optic {
        optic: Box::new(Strong::first),
        _marker: PhantomData,
    })
}

/// A `Lens` that focuses on the second element of a pair `(C, A)`.
/// Allows getting `A` and setting `A` to `B`, resulting in `(C, B)`.
///
/// Type parameters `PA` and `PS` are the profunctor types involved, usually inferred.
#[must_use]
pub fn _2<
    A: 'static,
    B: 'static,
    C: 'static,
    PA: Strong<A, B, Pro<(C, A), (C, B)> = PS> + 'static,
    PS: Strong<(C, A), (C, B)>,
>() -> Lens<PS, PA, (C, A), (C, B), A, B> {
    Lens(Optic {
        optic: Box::new(Strong::second),
        _marker: PhantomData,
    })
}

/// Example helper function to create a lens for the first element of a pair.
#[allow(dead_code)]
fn _1_new<A: Copy + 'static, BTuple: Copy + 'static, PO, PI>(
) -> Lens<PO, PI, (A, BTuple), (A, BTuple), A, A>
where
    PI: Strong<A, A>,
    PO: Strong<(A, BTuple), (A, BTuple)>,
    <PI as Profunctor<A, A>>::Pro<(A, RcFn<A, (A, BTuple)>), (A, RcFn<A, (A, BTuple)>)>: Profunctor<
        (A, RcFn<A, (A, BTuple)>),
        (A, RcFn<A, (A, BTuple)>),
        Pro<(A, BTuple), (A, BTuple)> = PO,
    >,
{
    lens(
        RcFn::new(|(first_val, _second_val): (A, BTuple)| first_val),
        RcFn::new(|(_original_a, original_b_tuple): (A, BTuple)| {
            RcFn::new(move |new_a: A| (new_a, original_b_tuple))
        }),
    )
}

/// A simple struct for demonstrating lenses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Check {
    /// A field in the Check struct.
    pub key: i8,
    /// Another field in the Check struct.
    pub other: i8,
}

/// A `Lens` that focuses on the `key` field of a `Check` struct.
/// Allows getting `check.key` and setting it.
#[must_use]
pub fn _key<PO, PI>() -> Lens<PO, PI, Check, Check, i8, i8>
where
    PO: Strong<Check, Check>,
    PI: Strong<i8, i8>,
    <PI as Profunctor<i8, i8>>::Pro<(i8, RcFn<i8, Check>), (i8, RcFn<i8, Check>)>:
        Profunctor<(i8, RcFn<i8, Check>), (i8, RcFn<i8, Check>), Pro<Check, Check> = PO>,
{
    lens(
        RcFn::new(|c: Check| c.key),
        RcFn::new(|c: Check| {
            RcFn::new(move |new_key: i8| Check {
                key: new_key,
                other: c.other,
            })
        }),
    )
}

/// Maps the input of a `Profunctor` (contravariant mapping).
/// `lcmap(f, p)` is equivalent to `p.dimap(f, id)`.
///
/// If `p: P<B,C>` and `f: A->B`, then `lcmap(f, p)` results in `P<A,C>`.
///
/// # Parameters
/// - `a2b`: The function `A -> B` to pre-compose with the profunctor's input.
/// - `profunctor`: The `Profunctor<B, C>` instance.
///
/// # Returns
/// A new profunctor `Profunctor::Pro<A, C>`.
#[must_use]
pub fn lcmap<A, B, C, F, Pbc, Pac>(a2b: F, profunctor: Pbc) -> Pac
where
    A: 'static,
    B: 'static,
    C: 'static,
    F: Fn(A) -> B + 'static,
    Pbc: Profunctor<B, C, Pro<A, C> = Pac>,
{
    profunctor.dimap(a2b, |c_val: C| c_val)
}

/// Maps the output of a `Profunctor` (covariant mapping).
/// `rmap(f, p)` is equivalent to `p.dimap(id, f)`.
///
/// If `p: P<A,B>` and `f: B->C`, then `rmap(f, p)` results in `P<A,C>`.
///
/// # Parameters
/// - `b2c`: The function `B -> C` to post-compose with the profunctor's output.
/// - `profunctor`: The `Profunctor<A, B>` instance.
///
/// # Returns
/// A new profunctor `Profunctor::Pro<A, C>`.
#[must_use]
pub fn rmap<A, B, C, F, Pab, Pac>(b2c: F, profunctor: Pab) -> Pac
where
    A: 'static,
    B: 'static,
    C: 'static,
    F: Fn(B) -> C + 'static,
    Pab: Profunctor<A, B, Pro<A, C> = Pac>,
{
    profunctor.dimap(|a_val: A| a_val, b2c)
}
