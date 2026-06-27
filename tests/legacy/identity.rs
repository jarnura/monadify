#![cfg(all(test, feature = "legacy"))] // Ensure these run only when 'legacy' is active

// These imports will need to point to legacy versions.
use monadify::function::RcFn;
use monadify::legacy::applicative::Applicative;
use monadify::legacy::apply::Apply;
use monadify::legacy::functor::Functor;
use monadify::legacy::identity::Identity;
use monadify::legacy::monad::Bind; // Assuming legacy Monad is also re-exported or directly used // CFn is not part of legacy/hkt split, path should be fine

#[test]
fn test_identity_functor_map() {
    let id_val = Identity(String::from("hello"));
    let mapped_id = <Identity<String> as Functor<String>>::map(id_val, |s: String| s.len());
    assert_eq!(mapped_id, Identity(5));

    let id_num = Identity(10);
    let mapped_id_num = <Identity<i32> as Functor<i32>>::map(id_num, |x| x * x);
    assert_eq!(mapped_id_num, Identity(100));
}

#[test]
fn test_identity_apply() {
    let id_val = Identity(5);
    let id_fn: Identity<RcFn<i32, i32>> = Identity(RcFn::new(|x| x * 2));
    let result = <Identity<i32> as Apply<i32>>::apply(id_val, id_fn);
    assert_eq!(result, Identity(10));

    let id_str_val = Identity(String::from("test"));
    let id_str_fn: Identity<RcFn<String, usize>> = Identity(RcFn::new(|s: String| s.len()));
    let result_str = <Identity<String> as Apply<String>>::apply(id_str_val, id_str_fn);
    assert_eq!(result_str, Identity(4));
}

#[test]
fn test_identity_applicative_pure() {
    let pure_val: Identity<i32> = <Identity<i32> as Applicative<i32>>::pure(42);
    assert_eq!(pure_val, Identity(42));

    let pure_str: Identity<&str> = <Identity<&str> as Applicative<&str>>::pure("pure");
    assert_eq!(pure_str, Identity("pure"));
}

#[test]
fn test_identity_monad_bind() {
    let id_val = Identity(3);
    let f = |x: i32| Identity(x + 7);
    let result = <Identity<i32> as Bind<i32>>::bind(id_val, f);
    assert_eq!(result, Identity(10));

    let id_str = Identity(String::from("world"));
    let f_str = |s: String| Identity(format!("hello {}", s));
    let result_str = <Identity<String> as Bind<String>>::bind(id_str, f_str);
    assert_eq!(result_str, Identity(String::from("hello world")));
}

#[test]
fn test_identity_monad_left_identity() {
    let a = 10;
    let f = |x: i32| Identity(x * x);
    let lhs: Identity<i32> =
        <Identity<i32> as Bind<i32>>::bind(<Identity<i32> as Applicative<i32>>::pure(a), f);
    let rhs: Identity<i32> = f(a);
    assert_eq!(lhs, rhs);
}

#[test]
fn test_identity_monad_right_identity() {
    let m = Identity(20);
    let pure_fn = |x: i32| <Identity<i32> as Applicative<i32>>::pure(x);
    let lhs = <Identity<i32> as Bind<i32>>::bind(m.clone(), pure_fn);
    let rhs = m;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_identity_monad_associativity() {
    let m = Identity(5);
    let f = |x: i32| Identity(x + 1);
    let g = |y: i32| Identity(y * 2);
    let lhs =
        <Identity<i32> as Bind<i32>>::bind(<Identity<i32> as Bind<i32>>::bind(m.clone(), f), g);
    let rhs_fn = move |x: i32| <Identity<i32> as Bind<i32>>::bind(f(x), g);
    let rhs = <Identity<i32> as Bind<i32>>::bind(m, rhs_fn);
    assert_eq!(lhs, rhs);
}
