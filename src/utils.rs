/// Creates an `RcFn` (shared-ownership `Rc<dyn Fn>`) from a nullary (0-argument) closure.
///
/// The resulting `RcFn` will take a dummy argument (e.g., `()`) which it ignores,
/// then calls the original nullary closure.
///
/// # Examples
/// ```
/// use monadify::fn0;
/// use monadify::function::RcFn;
///
/// let get_number = fn0!(|| 42);
/// let result: i32 = get_number.call(()); // Call with a dummy unit argument
/// assert_eq!(result, 42);
///
/// let greet = fn0!(|| "Hello".to_string());
/// assert_eq!(greet.call(()), "Hello");
/// ```
#[macro_export]
macro_rules! fn0 {
    ($closure:expr) => {
        $crate::function::RcFn::new(|_: ()| $closure())
    };
}

/// Creates an `RcFn` (shared-ownership `Rc<dyn Fn>`) from a unary (1-argument) closure.
///
/// This is a convenience macro for `RcFn::new(closure)`.
///
/// # Examples
/// ```
/// use monadify::fn1;
/// use monadify::function::RcFn;
///
/// let add_one = fn1!(|x: i32| x + 1);
/// assert_eq!(add_one.call(5), 6);
///
/// let to_string_fn = fn1!(|x: i32| x.to_string());
/// assert_eq!(to_string_fn.call(10), "10");
/// ```
#[macro_export]
macro_rules! fn1 {
    ($closure:expr) => {
        $crate::function::RcFn::new(move |x| $closure(x))
    };
}

/// Creates a curried function of two arguments, with the inner result wrapped in `RcFn`.
///
/// Given a closure `|x| |y| expr`, `fn2!` transforms it into
/// `move |x| RcFn::new(move |y| closure(x)(y))`.
/// The outer function takes `x` and returns an `RcFn` that takes `y`.
///
/// # Examples
/// ```
/// use monadify::fn2;
/// use monadify::function::RcFn;
///
/// let curried_add = fn2!(|x: i32| move |y: i32| x + y);
///
/// let add_5_fn = curried_add(5); // add_5_fn is RcFn<i32, i32>
/// assert_eq!(add_5_fn.call(10), 15); // Calls the inner closure with y = 10
///
/// assert_eq!(curried_add(3).call(7), 10);
/// ```
#[macro_export]
macro_rules! fn2 {
    ($closure:expr) => {
        move |x| $crate::function::RcFn::new(move |y| $closure(x)(y))
    };
}

/// Creates a curried function of three arguments, wrapped in nested `RcFn`s.
///
/// Given a closure `|x| |y| |z| expr`, `fn3!` transforms it into
/// `move |x| RcFn::new(move |y| RcFn::new(move |z| closure(x)(y)(z)))`.
///
/// # Examples
/// ```
/// use monadify::fn3;
/// use monadify::function::RcFn;
///
/// let curried_add3 = fn3!(|x: i32| move |y: i32| move |z: i32| x + y + z);
///
/// let add_5_and_10_fn = curried_add3(5).call(10); // add_5_and_10_fn is RcFn<i32, i32>
/// assert_eq!(add_5_and_10_fn.call(20), 35);
///
/// assert_eq!(curried_add3(1)(2).call(3), 6);
/// ```
#[macro_export]
macro_rules! fn3 {
    ($closure:expr) => {
        move |x| {
            $crate::function::RcFn::new(move |y| {
                $crate::function::RcFn::new(move |z| $closure(x)(y)(z))
            })
        }
    };
}
