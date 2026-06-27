// Original imports from src/identity.rs kind_tests module, adjusted
use monadify::applicative::kind::Applicative; // Changed hkt to kind
use monadify::apply::kind::Apply; // Changed hkt to kind
use monadify::function::RcFn;
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::identity::{Identity, IdentityKind}; // Changed HKTMarker to Kind
use monadify::monad::kind::{Bind, Monad}; // Changed hkt to kind

// The kind_tests module content
#[test]
fn test_identity_kind_functor_map() {
    // Renamed test
    let id_val: Identity<String> = Identity(String::from("hello"));
    let mapped_id: Identity<usize> = IdentityKind::map(id_val, |s: String| s.len()); // Renamed Marker
    assert_eq!(mapped_id, Identity(5));

    let id_num: Identity<i32> = Identity(10);
    let mapped_id_num: Identity<i32> = IdentityKind::map(id_num, |x| x * x); // Renamed Marker
    assert_eq!(mapped_id_num, Identity(100));
}

#[test]
fn test_identity_kind_apply() {
    // Renamed test
    let id_val: Identity<i32> = Identity(5);
    let id_fn: Identity<RcFn<i32, i32>> = Identity(RcFn::new(|x| x * 2));
    let result: Identity<i32> = IdentityKind::apply(id_val, id_fn); // Renamed Marker
    assert_eq!(result, Identity(10));
}

#[test]
fn test_identity_kind_applicative_pure() {
    // Renamed test
    let pure_val: Identity<i32> = IdentityKind::pure(42); // Renamed Marker
    assert_eq!(pure_val, Identity(42));
}

#[test]
fn test_identity_kind_monad_bind() {
    // Renamed test
    let id_val: Identity<i32> = Identity(3);
    let f = |x: i32| -> Identity<i32> { Identity(x + 7) };
    let result: Identity<i32> = IdentityKind::bind(id_val, f); // Renamed Marker
    assert_eq!(result, Identity(10));
}

#[test]
fn test_identity_kind_monad_join() {
    // Renamed test
    let nested_id: Identity<Identity<i32>> = Identity(Identity(42));
    let joined_id: Identity<i32> = IdentityKind::join(nested_id); // Renamed Marker
    assert_eq!(joined_id, Identity(42));

    let nested_str_id: Identity<Identity<String>> = Identity(Identity(String::from("test")));
    let joined_str_id: Identity<String> = IdentityKind::join(nested_str_id); // Renamed Marker
    assert_eq!(joined_str_id, Identity(String::from("test")));
}

// Law tests for Kind version
#[test]
fn test_identity_kind_left_identity() {
    // Renamed test
    let a = 10;
    let f = |x: i32| -> Identity<i32> { Identity(x * x) };
    let lhs: Identity<i32> = IdentityKind::bind(
        // Renamed Marker
        IdentityKind::pure(a), // Renamed Marker
        f,
    );
    let rhs: Identity<i32> = f(a);
    assert_eq!(lhs, rhs);
}

#[test]
fn test_identity_kind_right_identity() {
    // Renamed test
    let m: Identity<i32> = Identity(20);
    let pure_fn = |x: i32| -> Identity<i32> { IdentityKind::pure(x) }; // Renamed Marker
    let lhs = IdentityKind::bind(m.clone(), pure_fn); // Renamed Marker
    let rhs = m;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_identity_kind_associativity() {
    // Renamed test
    let m: Identity<i32> = Identity(5);
    let f = |x: i32| -> Identity<i32> { Identity(x + 1) };
    let g = |y: i32| -> Identity<i32> { Identity(y * 2) };

    let lhs = IdentityKind::bind(IdentityKind::bind(m.clone(), f), g); // Renamed Marker
    let rhs_fn = move |x: i32| -> Identity<i32> { IdentityKind::bind(f(x), g) }; // Renamed Marker
    let rhs = IdentityKind::bind(m, rhs_fn); // Renamed Marker
    assert_eq!(lhs, rhs);
}
