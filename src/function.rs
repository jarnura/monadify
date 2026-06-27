use std::ops::Deref;
use std::rc::Rc;

/// Type alias for a boxed, dynamically dispatched, repeatable closure.
/// `BFn<A, B>` is equivalent to `Box<dyn Fn(A) -> B + 'static>`.
/// This represents a heap-allocated closure that can be called multiple times.
type BFn<A, B> = Box<dyn Fn(A) -> B + 'static>;

/// Type alias for an `Rc`-backed, dynamically dispatched, repeatable closure.
/// `RFn<A, B>` is equivalent to `Rc<dyn Fn(A) -> B + 'static>`.
type RFn<A, B> = Rc<dyn Fn(A) -> B + 'static>;

/// Type alias for a boxed, dynamically dispatched, once-callable closure.
/// `BFnOnce<A, B>` is equivalent to `Box<dyn FnOnce(A) -> B + 'static>`.
/// This represents a heap-allocated closure that can be called at most once.
type BFnOnce<A, B> = Box<dyn FnOnce(A) -> B + 'static>;

/// A wrapper around `BFn<A, B>` (a `Box<dyn Fn(A) -> B + 'static>`).
///
/// This struct provides a concrete type for heap-allocated, repeatable closures,
/// which is useful for storing them in structs or passing them as arguments
/// where a concrete type is needed (e.g., in trait implementations like `Functor` for functions).
///
/// `CFn` wraps `Box<dyn Fn(A) -> B + 'static>` and is therefore **not** `Clone`
/// (unique-ownership, box-backed). For a shared-ownership, cheaply-cloneable
/// sibling, see [`RcFn`], which wraps `Rc<dyn Fn(A) -> B + 'static>` and
/// implements `Clone` in O(1).
///
/// # Examples
/// ```
/// use monadify::function::CFn;
///
/// let add_one = CFn::new(|x: i32| x + 1);
/// assert_eq!(add_one.call(5), 6);
/// assert_eq!(add_one.call(10), 11); // Can be called multiple times
/// ```
pub struct CFn<A, B>(pub BFn<A, B>);

/// A wrapper around `BFnOnce<A, B>` (a `Box<dyn FnOnce(A) -> B + 'static>`).
///
/// This struct provides a concrete type for heap-allocated, once-callable closures.
/// Similar to `CFn`, it's useful for type concretization.
///
/// # Examples
/// ```
/// use monadify::function::CFnOnce;
///
/// let s = "hello".to_string();
/// // This closure captures `s` by move, so it's FnOnce.
/// let append_s_once = CFnOnce::new(move |x: i32| format!("{}-{}", s, x));
/// assert_eq!(append_s_once.call_once(5), "hello-5");
/// // append_s_once.call_once(10); // This would be a compile error (use of moved value) if not for Box
///                               // but logically it's consumed.
/// ```
pub struct CFnOnce<A, B>(pub BFnOnce<A, B>);

impl<A, B> CFn<A, B> {
    /// Creates a new `CFn` by boxing the given closure.
    ///
    /// # Parameters
    /// - `f`: A closure that implements `Fn(A) -> B` and is `'static`.
    ///
    /// # Returns
    /// A new `CFn<A, B>` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(A) -> B + 'static,
    {
        CFn(Box::new(f))
    }

    /// Calls the wrapped closure.
    ///
    /// This method takes `&self` and the argument `arg` by value,
    /// allowing the closure to be called multiple times.
    ///
    /// # Parameters
    /// - `arg`: The argument of type `A` to pass to the closure.
    ///
    /// # Returns
    /// The result of type `B` from calling the closure.
    pub fn call(&self, arg: A) -> B {
        (self.0)(arg)
    }
}

impl<A, B> CFnOnce<A, B> {
    /// Creates a new `CFnOnce` by boxing the given closure.
    ///
    /// # Parameters
    /// - `f`: A closure that implements `FnOnce(A) -> B` and is `'static`.
    ///
    /// # Returns
    /// A new `CFnOnce<A, B>` instance.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + 'static,
    {
        CFnOnce(Box::new(f))
    }

    /// Calls the wrapped closure once.
    ///
    /// This method takes `self` by value, consuming the `CFnOnce` instance,
    /// reflecting the `FnOnce` nature of the underlying closure.
    ///
    /// # Parameters
    /// - `arg`: The argument of type `A` to pass to the closure.
    ///
    /// # Returns
    /// The result of type `B` from calling the closure.
    pub fn call_once(self, arg: A) -> B {
        (self.0)(arg)
    }
}

/// Allows `CFn<A, B>` to be dereferenced to `&Box<dyn Fn(A) -> B + 'static>`.
/// This enables calling the boxed closure directly using `(*cfn_instance)(arg)` syntax
/// if desired, though `cfn_instance.call(arg)` is generally preferred for clarity.
impl<A, B> Deref for CFn<A, B> {
    type Target = BFn<A, B>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allows `CFnOnce<A, B>` to be dereferenced to `&Box<dyn FnOnce(A) -> B + 'static>`.
impl<A, B> Deref for CFnOnce<A, B> {
    type Target = BFnOnce<A, B>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Composes two boxed `Fn` closures.
/// Given `f: A -> B` and `g: B -> C`, returns a new boxed closure `h: A -> C`
/// such that `h(x) = g(f(x))`.
fn compose<A: 'static, B: 'static, C: 'static>(f: BFn<A, B>, g: BFn<B, C>) -> BFn<A, C> {
    Box::new(move |x| g(f(x)))
}

/// Composes two boxed `FnOnce` closures.
/// Given `f: A -> B` and `g: B -> C`, returns a new boxed closure `h: A -> C`
/// such that `h(x) = g(f(x))`.
/// The resulting closure is also `FnOnce`.
fn compose_fn_once<A: 'static, B: 'static, C: 'static>(
    f: BFnOnce<A, B>,
    g: BFnOnce<B, C>,
) -> BFnOnce<A, C> {
    Box::new(move |x| g(f(x))) // f and g are moved into the closure
}

/// Implements `f >> g` (forward composition) for `CFn`.
/// `(self >> rhs)(x)` is equivalent to `rhs(self(x))`.
/// `CFn<A,B> >> CFn<B,C>` results in `CFn<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shr<CFn<B, C>> for CFn<A, B> {
    type Output = CFn<A, C>;
    fn shr(self, rhs: CFn<B, C>) -> Self::Output {
        // self is f: A -> B, rhs is g: B -> C
        // Result is g(f(x))
        CFn(compose(self.0, rhs.0))
    }
}

/// Implements `g << f` (backward composition) for `CFn`.
/// `(self << rhs)(x)` is equivalent to `self(rhs(x))`.
/// `CFn<B,C> << CFn<A,B>` results in `CFn<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shl<CFn<A, B>> for CFn<B, C> {
    type Output = CFn<A, C>;
    fn shl(self, rhs: CFn<A, B>) -> Self::Output {
        // self is g: B -> C, rhs is f: A -> B
        // Result is g(f(x))
        CFn(compose(rhs.0, self.0))
    }
}

/// Implements `f >> g` (forward composition) for `CFnOnce`.
/// `(self >> rhs)(x)` is equivalent to `rhs(self(x))`.
/// `CFnOnce<A,B> >> CFnOnce<B,C>` results in `CFnOnce<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shr<CFnOnce<B, C>> for CFnOnce<A, B> {
    type Output = CFnOnce<A, C>;
    fn shr(self, rhs: CFnOnce<B, C>) -> Self::Output {
        CFnOnce(compose_fn_once(self.0, rhs.0))
    }
}

/// Implements `g << f` (backward composition) for `CFnOnce`.
/// `(self << rhs)(x)` is equivalent to `self(rhs(x))`.
/// `CFnOnce<B,C> << CFnOnce<A,B>` results in `CFnOnce<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shl<CFnOnce<A, B>> for CFnOnce<B, C> {
    type Output = CFnOnce<A, C>;
    fn shl(self, rhs: CFnOnce<A, B>) -> Self::Output {
        CFnOnce(compose_fn_once(rhs.0, self.0))
    }
}

// ── RcFn ──────────────────────────────────────────────────────────────────────

/// A Clone-able, shared-ownership function wrapper backed by
/// `Rc<dyn Fn(A) -> B + 'static>`.
///
/// `RcFn<A, B>` is the **Clone-able, shared-ownership** sibling of [`CFn`].
/// While [`CFn`] wraps `Box<dyn Fn(A) -> B + 'static>` and is therefore
/// **not** `Clone`, `RcFn` wraps `Rc<dyn Fn(A) -> B + 'static>` and
/// implements `Clone`. Cloning an `RcFn` bumps the `Rc` reference count
/// in O(1) — the closure body is **not** duplicated.
///
/// Unlike `#[derive(Clone)]` (which would add bounds `A: Clone, B: Clone`),
/// the `Clone` impl here only requires `Rc<dyn Fn(A)->B+'static>: Clone`,
/// which is always satisfied regardless of `A` and `B`. This allows
/// `RcFn<X, CFn<A,B>>` to be `Clone` even though `CFn<A,B>` is not.
///
/// This makes `RcFn` law-equivalent to a deep copy **only for
/// referentially-transparent `Fn`** closures (no observable interior
/// mutability such as `Cell`/`RefCell`). This is the same pattern
/// [`crate::transformers::reader::ReaderT`] already uses internally with
/// `Rc<dyn Fn>`.
///
/// # Examples
/// ```
/// use monadify::function::RcFn;
///
/// let f: RcFn<i32, i32> = RcFn::new(|x: i32| x + 1);
/// let g = f.clone(); // O(1) — shares the underlying closure
/// assert_eq!(f.call(3), 4);
/// assert_eq!(g.call(3), 4);
/// ```
pub struct RcFn<A, B>(pub RFn<A, B>);

/// Manual `Clone` impl that does NOT add `A: Clone` or `B: Clone` bounds.
///
/// `Rc<dyn Fn(A)->B+'static>` is always `Clone` (it just bumps the reference
/// count), so we can implement `Clone for RcFn<A,B>` unconditionally.
/// This means `RcFn<X, CFn<A,B>>` is `Clone` even though `CFn<A,B>` is not.
impl<A, B> Clone for RcFn<A, B> {
    fn clone(&self) -> Self {
        RcFn(Rc::clone(&self.0))
    }
}

impl<A, B> RcFn<A, B> {
    /// Creates a new `RcFn` by wrapping the given closure in an `Rc`.
    ///
    /// # Parameters
    /// - `f`: A closure that implements `Fn(A) -> B` and is `'static`.
    ///
    /// # Returns
    /// A new `RcFn<A, B>` instance whose clone shares the same closure allocation.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(A) -> B + 'static,
    {
        RcFn(Rc::new(f))
    }

    /// Calls the wrapped closure with `arg`.
    ///
    /// This method takes `&self`, so the same `RcFn` can be called multiple times.
    ///
    /// # Parameters
    /// - `arg`: The argument of type `A` to pass to the closure.
    ///
    /// # Returns
    /// The result of type `B` from calling the closure.
    pub fn call(&self, arg: A) -> B {
        // `self.0.as_ref()` coerces `&Rc<dyn Fn(A)->B>` → `&dyn Fn(A)->B`,
        // which implements `Fn(A)->B` via the blanket `impl<F: Fn+?Sized> Fn for &F`.
        (self.0.as_ref())(arg)
    }
}

/// Allows `RcFn<A, B>` to be dereferenced to `&Rc<dyn Fn(A) -> B + 'static>`.
impl<A, B> Deref for RcFn<A, B> {
    type Target = Rc<dyn Fn(A) -> B + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implements `f >> g` (forward composition) for `RcFn`.
/// `(self >> rhs)(x)` is equivalent to `rhs(self(x))`.
/// `RcFn<A,B> >> RcFn<B,C>` results in `RcFn<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shr<RcFn<B, C>> for RcFn<A, B> {
    type Output = RcFn<A, C>;
    fn shr(self, rhs: RcFn<B, C>) -> Self::Output {
        let f = self.0;
        let g = rhs.0;
        // Both `f` and `g` are owned `Rc<dyn Fn>` moved into the closure.
        // Calling `f(x)` auto-derefs `Rc<dyn Fn(A)->B>` → `dyn Fn(A)->B`.
        RcFn(Rc::new(move |x: A| g(f(x))))
    }
}

/// Implements `g << f` (backward composition) for `RcFn`.
/// `(self << rhs)(x)` is equivalent to `self(rhs(x))`.
/// `RcFn<B,C> << RcFn<A,B>` results in `RcFn<A,C>`.
impl<A: 'static, B: 'static, C: 'static> std::ops::Shl<RcFn<A, B>> for RcFn<B, C> {
    type Output = RcFn<A, C>;
    fn shl(self, rhs: RcFn<A, B>) -> Self::Output {
        let f = rhs.0;
        let g = self.0;
        RcFn(Rc::new(move |x: A| g(f(x))))
    }
}
