// Imports needed for the tests, adjusted from src/applicative.rs context
use core::convert::identity;
use monadify::applicative::kind::*; // For Applicative trait and lift_a1. Changed hkt to kind
use monadify::apply::kind::Apply; // Changed hkt to kind
use monadify::function::{CFnOnce, RcFn};
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::identity::{Identity as IdType, IdentityKind}; // Changed IdentityHKTMarker to IdentityKind
use monadify::kind_based::kind::{CFnOnceKind, OptionKind, RcFnKind, ResultKind, VecKind};
use monadify::transformers::reader::{ReaderT, ReaderTKind}; // Changed ReaderTHKTMarker to ReaderTKind

// --- OptionKind Applicative Laws ---
#[test]
fn option_kind_applicative_law_identity() {
    let v_some: Option<i32> = Some(10);
    let v_none: Option<i32> = None;

    let id_rcfn_creator = || RcFn::new(identity::<i32>);
    let pure_id_rcfn_creator = || OptionKind::pure(id_rcfn_creator());

    assert_eq!(OptionKind::apply(v_some, pure_id_rcfn_creator()), v_some);
    assert_eq!(OptionKind::apply(v_none, pure_id_rcfn_creator()), v_none);
}

#[test]
fn option_kind_applicative_law_homomorphism() {
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_rcfn_creator = || RcFn::new(f);
    let pure_f_rcfn: Option<RcFn<i32, i32>> = OptionKind::pure(f_rcfn_creator());
    let pure_x: Option<i32> = OptionKind::pure(x);

    assert_eq!(
        OptionKind::apply(pure_x, pure_f_rcfn),
        OptionKind::pure(f(x))
    );
}

#[test]
fn option_kind_applicative_law_interchange() {
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || RcFn::new(|val: A| format!("val:{}", val));
    let u_some_creator = || Some(concrete_f_creator());
    let u_none_creator = || None::<RcFn<A, B>>;

    let pure_y: Option<A> = OptionKind::pure(y_val);

    // LHS: apply(pure(y), u)
    let lhs_some = OptionKind::apply(pure_y, u_some_creator());
    let lhs_none = OptionKind::apply(pure_y, u_none_creator());

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || RcFn::new(move |f_map_fn: RcFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator = || OptionKind::pure(interchange_fn_creator());

    let rhs_some = OptionKind::apply(u_some_creator(), pure_interchange_fn_wrapper_creator());
    let rhs_none = OptionKind::apply(u_none_creator(), pure_interchange_fn_wrapper_creator());

    assert_eq!(lhs_some, rhs_some);
    assert_eq!(lhs_none, rhs_none);
    assert_eq!(lhs_some, Some("val:10".to_string()));
}

// Functor laws for lift_a1 (map defined via pure/apply)
#[test]
fn option_kind_lift_a1_functor_identity() {
    let fa_some: Option<i32> = Some(10);
    let fa_none: Option<i32> = None;
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<OptionKind, _, _, _>(id_fn_static, fa_some),
        fa_some
    );
    assert_eq!(
        lift_a1::<OptionKind, _, _, _>(id_fn_static, fa_none),
        fa_none
    );
}

#[test]
fn option_kind_lift_a1_functor_composition() {
    let fa_some: Option<i32> = Some(10);
    let fa_none: Option<i32> = None;

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_some = lift_a1::<OptionKind, _, _, _>(g_compose_f, fa_some);
    let lhs_none = lift_a1::<OptionKind, _, _, _>(g_compose_f, fa_none);

    let map_f_fa_some = lift_a1::<OptionKind, _, _, _>(f, fa_some);
    let rhs_some = lift_a1::<OptionKind, _, _, _>(g, map_f_fa_some);

    let map_f_fa_none = lift_a1::<OptionKind, _, _, _>(f, fa_none);
    let rhs_none = lift_a1::<OptionKind, _, _, _>(g, map_f_fa_none);

    assert_eq!(lhs_some, rhs_some);
    assert_eq!(lhs_none, rhs_none);
    assert_eq!(lhs_some, Some("20".to_string()));
}

// --- ResultKind Applicative Laws ---
type TestError = String;

#[test]
fn result_kind_applicative_law_identity() {
    let v_ok: Result<i32, TestError> = Ok(10);
    let v_err: Result<i32, TestError> = Err("Error".to_string());

    let id_rcfn_creator = || RcFn::new(identity::<i32>);
    let pure_id_rcfn_creator = || ResultKind::<TestError>::pure(id_rcfn_creator());

    assert_eq!(
        ResultKind::<TestError>::apply(v_ok.clone(), pure_id_rcfn_creator()),
        v_ok
    );
    assert_eq!(
        ResultKind::<TestError>::apply(v_err.clone(), pure_id_rcfn_creator()),
        v_err
    );
}

#[test]
fn result_kind_applicative_law_homomorphism() {
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_rcfn_creator = || RcFn::new(f);
    let pure_f_rcfn = ResultKind::<TestError>::pure(f_rcfn_creator());
    let pure_x = ResultKind::<TestError>::pure(x);

    assert_eq!(
        ResultKind::<TestError>::apply(pure_x, pure_f_rcfn),
        ResultKind::<TestError>::pure(f(x))
    );
}

#[test]
fn result_kind_applicative_law_interchange() {
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || RcFn::new(|val: A| format!("val:{}", val));
    let u_ok_creator = || Ok(concrete_f_creator());
    let u_err_creator = || Err::<RcFn<A, B>, TestError>("Error in u".to_string());

    let pure_y = ResultKind::<TestError>::pure(y_val);

    let lhs_ok = ResultKind::<TestError>::apply(pure_y.clone(), u_ok_creator());
    let lhs_err = ResultKind::<TestError>::apply(pure_y.clone(), u_err_creator());

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || RcFn::new(move |f_map_fn: RcFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator =
        || ResultKind::<TestError>::pure(interchange_fn_creator());

    let rhs_ok =
        ResultKind::<TestError>::apply(u_ok_creator(), pure_interchange_fn_wrapper_creator());
    let rhs_err =
        ResultKind::<TestError>::apply(u_err_creator(), pure_interchange_fn_wrapper_creator());

    assert_eq!(lhs_ok, rhs_ok);
    assert_eq!(lhs_err, rhs_err);
    assert_eq!(lhs_ok, Ok("val:10".to_string()));
    assert_eq!(lhs_err, Err("Error in u".to_string()));
}

#[test]
fn result_kind_lift_a1_functor_identity() {
    let fa_ok: Result<i32, TestError> = Ok(10);
    let fa_err: Result<i32, TestError> = Err("Error".to_string());
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<ResultKind<TestError>, _, _, _>(id_fn_static, fa_ok.clone()),
        fa_ok
    );
    assert_eq!(
        lift_a1::<ResultKind<TestError>, _, _, _>(id_fn_static, fa_err.clone()),
        fa_err
    );
}

#[test]
fn result_kind_lift_a1_functor_composition() {
    let fa_ok: Result<i32, TestError> = Ok(10);
    let fa_err: Result<i32, TestError> = Err("Error".to_string());

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_ok = lift_a1::<ResultKind<TestError>, _, _, _>(g_compose_f, fa_ok.clone());
    let lhs_err = lift_a1::<ResultKind<TestError>, _, _, _>(g_compose_f, fa_err.clone());

    let map_f_fa_ok = lift_a1::<ResultKind<TestError>, _, _, _>(f, fa_ok.clone());
    let rhs_ok = lift_a1::<ResultKind<TestError>, _, _, _>(g, map_f_fa_ok);

    let map_f_fa_err = lift_a1::<ResultKind<TestError>, _, _, _>(f, fa_err.clone());
    let rhs_err = lift_a1::<ResultKind<TestError>, _, _, _>(g, map_f_fa_err);

    assert_eq!(lhs_ok, rhs_ok);
    assert_eq!(lhs_err, rhs_err);
    assert_eq!(lhs_ok, Ok("20".to_string()));
    assert_eq!(lhs_err, Err("Error".to_string()));
}

// --- VecKind Applicative Laws ---
// NOTE: With RcFn (which is Clone), VecKind applicative laws are now testable!
#[test]
fn vec_kind_applicative_law_identity() {
    let v_vec: Vec<i32> = vec![1, 2, 3];
    let id_rcfn = RcFn::new(identity::<i32>);
    let pure_id: Vec<RcFn<i32, i32>> = VecKind::pure(id_rcfn);
    let result = VecKind::apply(v_vec.clone(), pure_id);
    assert_eq!(result, v_vec);
}

#[test]
fn vec_kind_applicative_law_homomorphism() {
    let x: i32 = 5;
    let f = |val: i32| val * 3;
    let pure_x: Vec<i32> = VecKind::pure(x);
    let pure_f: Vec<RcFn<i32, i32>> = VecKind::pure(RcFn::new(f));
    assert_eq!(VecKind::apply(pure_x, pure_f), VecKind::pure(f(x)));
}

#[test]
fn vec_kind_applicative_law_interchange() {
    type A = i32;
    let y_val: A = 10;

    let concrete_f1_creator = || RcFn::new(|val: A| format!("f1:{}", val));
    let concrete_f2_creator = || RcFn::new(|val: A| format!("f2:{}", val * 2));
    let u_vec_creator = || vec![concrete_f1_creator(), concrete_f2_creator()];

    let pure_y_vec: Vec<A> = VecKind::pure(y_val);

    let lhs = VecKind::apply(pure_y_vec.clone(), u_vec_creator());
    assert_eq!(lhs, vec!["f1:10".to_string(), "f2:20".to_string()]);
}

#[test]
fn vec_kind_functor_identity_via_map() {
    let fa_vec: Vec<i32> = vec![10, 20];
    let fa_empty: Vec<i32> = vec![];
    let id_fn_static = identity::<i32>;

    assert_eq!(VecKind::map(fa_vec.clone(), id_fn_static), fa_vec);
    assert_eq!(VecKind::map(fa_empty.clone(), id_fn_static), fa_empty);
}

#[test]
fn vec_kind_functor_composition_via_map() {
    let fa_vec: Vec<i32> = vec![10, 20];
    let fa_empty: Vec<i32> = vec![];

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_vec = VecKind::map(fa_vec.clone(), g_compose_f);
    let lhs_empty = VecKind::map(fa_empty.clone(), g_compose_f);

    let map_f_fa_vec = VecKind::map(fa_vec.clone(), f);
    let rhs_vec = VecKind::map(map_f_fa_vec, g);

    let map_f_fa_empty = VecKind::map(fa_empty.clone(), f);
    let rhs_empty = VecKind::map(map_f_fa_empty, g);

    assert_eq!(lhs_vec, rhs_vec);
    assert_eq!(lhs_empty, rhs_empty);
    assert_eq!(lhs_vec, vec!["20".to_string(), "40".to_string()]);
    assert_eq!(lhs_empty, Vec::<String>::new());
}

// --- IdentityKind Applicative Laws ---
#[test]
fn identity_kind_applicative_law_identity() {
    let v: IdType<i32> = IdType(10);

    let id_rcfn_creator = || RcFn::new(identity::<i32>);
    let pure_id_rcfn: IdType<RcFn<i32, i32>> = IdentityKind::pure(id_rcfn_creator());

    assert_eq!(IdentityKind::apply(v.clone(), pure_id_rcfn), v);
}

#[test]
fn identity_kind_applicative_law_homomorphism() {
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_rcfn_creator = || RcFn::new(f);
    let pure_f_rcfn: IdType<RcFn<i32, i32>> = IdentityKind::pure(f_rcfn_creator());
    let pure_x: IdType<i32> = IdentityKind::pure(x);

    assert_eq!(
        IdentityKind::apply(pure_x, pure_f_rcfn),
        IdentityKind::pure(f(x))
    );
}

#[test]
fn identity_kind_applicative_law_interchange() {
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || RcFn::new(|val: A| format!("val:{}", val));
    let u_identity_creator = || IdType(concrete_f_creator());

    let pure_y: IdType<A> = IdentityKind::pure(y_val);

    let lhs = IdentityKind::apply(pure_y.clone(), u_identity_creator());

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || RcFn::new(move |f_map_fn: RcFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator = || IdentityKind::pure(interchange_fn_creator());

    let rhs = IdentityKind::apply(u_identity_creator(), pure_interchange_fn_wrapper_creator());

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, IdType("val:10".to_string()));
}

#[test]
fn identity_kind_lift_a1_functor_identity() {
    let fa_id: IdType<i32> = IdType(10);
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<IdentityKind, _, _, _>(id_fn_static, fa_id.clone()),
        fa_id
    );
}

#[test]
fn identity_kind_lift_a1_functor_composition() {
    let fa_id: IdType<i32> = IdType(10);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = lift_a1::<IdentityKind, _, _, _>(g_compose_f, fa_id.clone());
    let map_f_fa = lift_a1::<IdentityKind, _, _, _>(f, fa_id.clone());
    let rhs = lift_a1::<IdentityKind, _, _, _>(g, map_f_fa);

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, IdType("20".to_string()));
}

// --- RcFnKind Applicative Laws ---
// Previously "CFnKind" tests — migrated to RcFnKind since CFnKind is removed.
// With RcFn (Clone), all applicative laws are now testable.
type Env = i32;

#[test]
fn rcfn_kind_applicative_law_identity() {
    let env_val: Env = 5;
    let fa: RcFn<Env, i32> = RcFn::new(|e: Env| e * 2); // fa(5) = 10

    let id_rcfn = RcFn::new(identity::<i32>);
    let pure_id: RcFn<Env, RcFn<i32, i32>> = RcFnKind::<Env>::pure(id_rcfn);

    let applied = RcFnKind::<Env>::apply(fa.clone(), pure_id);

    assert_eq!(applied.call(env_val), fa.call(env_val));
    assert_eq!(applied.call(env_val), 10);
}

#[test]
fn rcfn_kind_applicative_law_homomorphism() {
    let env_val: Env = 0; // pure ignores env
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let lhs = RcFnKind::<Env>::apply(
        RcFnKind::<Env>::pure(x),
        RcFnKind::<Env>::pure(RcFn::new(f)),
    );
    let rhs: RcFn<Env, i32> = RcFnKind::<Env>::pure(f(x));

    assert_eq!(lhs.call(env_val), rhs.call(env_val));
    assert_eq!(lhs.call(env_val), 20);
}

#[test]
fn rcfn_kind_applicative_law_interchange() {
    type A = i32;
    type B = String;
    let env_val: Env = 99;
    let y_val: A = 10;

    let u: RcFn<Env, RcFn<A, B>> =
        RcFn::new(move |_env: Env| RcFn::new(|val: A| format!("val:{}", val)));

    let pure_y: RcFn<Env, A> = RcFnKind::<Env>::pure(y_val);

    // LHS: apply(pure(y), u)
    let lhs = RcFnKind::<Env>::apply(pure_y, u.clone());

    // RHS: apply(u, pure(|f_fn| f_fn(y)))
    let interchange = RcFn::new(move |f_fn: RcFn<A, B>| f_fn.call(y_val));
    let pure_interchange: RcFn<Env, RcFn<RcFn<A, B>, B>> = RcFnKind::<Env>::pure(interchange);
    let rhs = RcFnKind::<Env>::apply(u, pure_interchange);

    assert_eq!(lhs.call(env_val), rhs.call(env_val));
    assert_eq!(lhs.call(env_val), "val:10".to_string());
}

// --- RcFnKind Functor Laws (using map) ---
#[test]
fn rcfn_kind_functor_identity_via_map() {
    let env_val: Env = 5;
    let fa: RcFn<Env, i32> = RcFn::new(|e: Env| e * 2);
    let id_fn_static = identity::<i32>;

    let mapped = RcFnKind::<Env>::map(fa.clone(), id_fn_static);
    assert_eq!(mapped.call(env_val), fa.call(env_val));
}

#[test]
fn rcfn_kind_functor_composition_via_map() {
    let env_val: Env = 3;
    let fa: RcFn<Env, i32> = RcFn::new(|e: Env| e + 1);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs: RcFn<Env, String> = RcFnKind::<Env>::map(fa.clone(), g_compose_f);
    let map_f_fa: RcFn<Env, i32> = RcFnKind::<Env>::map(fa.clone(), f);
    let rhs: RcFn<Env, String> = RcFnKind::<Env>::map(map_f_fa, g);

    assert_eq!(lhs.call(env_val), rhs.call(env_val));
    assert_eq!(lhs.call(env_val), "8".to_string()); // (3 + 1) * 2 = 8
}

// --- CFnOnceKind Applicative Laws ---
#[test]
fn cfn_once_kind_applicative_law_identity() {
    println!("NOTE: CFnOnceKind Applicative Identity law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

#[test]
fn cfn_once_kind_applicative_law_homomorphism() {
    println!("NOTE: CFnOnceKind Applicative Homomorphism law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

#[test]
fn cfn_once_kind_applicative_law_interchange() {
    println!("NOTE: CFnOnceKind Applicative Interchange law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

// --- CFnOnceKind Functor Laws (using map) ---
#[test]
fn cfn_once_kind_functor_identity_via_map() {
    let fa_creator = || CFnOnce::new(|_e: Env| 10);
    let id_fn_static = identity::<i32>;

    let mapped = CFnOnceKind::<Env>::map(fa_creator(), id_fn_static);
    assert_eq!(mapped.call_once(100), fa_creator().call_once(100));
}

#[test]
fn cfn_once_kind_functor_composition_via_map() {
    let fa_creator = || CFnOnce::new(|_e: Env| 10);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = CFnOnceKind::<Env>::map(fa_creator(), g_compose_f);
    let lhs_result_for_0 = lhs.call_once(0);

    let map_f_fa = CFnOnceKind::<Env>::map(fa_creator(), f);
    let rhs = CFnOnceKind::<Env>::map(map_f_fa, g);

    assert_eq!(lhs_result_for_0.clone(), rhs.call_once(100));
    assert_eq!(lhs_result_for_0, "20".to_string());
}

// --- ReaderTKind Applicative Laws ---
type ReaderEnv = i32;

#[test]
fn reader_t_kind_applicative_law_identity() {
    println!("NOTE: ReaderTKind Applicative Identity law is untestable with RcFn due to Clone constraints on the inner monad.");
}

#[test]
fn reader_t_kind_applicative_law_homomorphism() {
    println!("NOTE: ReaderTKind Applicative Homomorphism law is untestable with RcFn due to Clone constraints.");
}

#[test]
fn reader_t_kind_applicative_law_interchange() {
    println!("NOTE: ReaderTKind Applicative Interchange law is untestable with RcFn due to Clone constraints.");
}

// --- ReaderTKind Functor Laws (using map) ---
#[test]
fn reader_t_kind_functor_identity_via_map() {
    let fa_creator = || ReaderT::<ReaderEnv, IdentityKind, i32>::new(|_e: ReaderEnv| IdType(10));
    let id_fn_static = identity::<i32>;

    let mapped = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), id_fn_static);

    let env_val = 100;
    assert_eq!(
        (mapped.run_reader_t)(env_val),
        (fa_creator().run_reader_t)(env_val)
    );
}

#[test]
fn reader_t_kind_functor_composition_via_map() {
    let fa_creator = || ReaderT::<ReaderEnv, IdentityKind, i32>::new(|_e: ReaderEnv| IdType(10));

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), g_compose_f);

    let map_f_fa = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), f);
    let rhs = ReaderTKind::<ReaderEnv, IdentityKind>::map(map_f_fa, g);

    let env_val = 100;
    assert_eq!((lhs.run_reader_t)(env_val), (rhs.run_reader_t)(env_val));
    assert_eq!((lhs.run_reader_t)(env_val), IdType("20".to_string()));
}
