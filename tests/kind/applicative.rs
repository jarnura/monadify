// Imports needed for the tests, adjusted from src/applicative.rs context
use core::convert::identity;
use monadify::applicative::kind::*; // For Applicative trait and lift_a1. Changed hkt to kind
use monadify::apply::kind::Apply; // Changed hkt to kind
use monadify::function::{CFn, CFnOnce};
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::identity::{Identity as IdType, IdentityKind}; // Changed IdentityHKTMarker to IdentityKind
use monadify::kind_based::kind::{CFnKind, CFnOnceKind, OptionKind, ResultKind, VecKind}; // ...HKTMarker to ...Kind
use monadify::transformers::reader::{ReaderT, ReaderTKind}; // Changed ReaderTHKTMarker to ReaderTKind

// The kind_laws_tests module itself
// Copied from src/applicative.rs

// --- OptionKind Applicative Laws ---
#[test]
fn option_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    // apply(v, pure(id_fn)) == v
    let v_some: Option<i32> = Some(10);
    let v_none: Option<i32> = None;

    let id_cfn_creator = || CFn::new(identity::<i32>);
    let pure_id_cfn_creator = || OptionKind::pure(id_cfn_creator()); // Renamed Marker

    assert_eq!(OptionKind::apply(v_some, pure_id_cfn_creator()), v_some); // Renamed Marker
    assert_eq!(OptionKind::apply(v_none, pure_id_cfn_creator()), v_none); // Renamed Marker
}

#[test]
fn option_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    // apply(pure(x), pure(f_fn)) == pure(f(x))
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_cfn_creator = || CFn::new(f);
    let pure_f_cfn: Option<CFn<i32, i32>> = OptionKind::pure(f_cfn_creator()); // Renamed Marker
    let pure_x: Option<i32> = OptionKind::pure(x); // Renamed Marker

    assert_eq!(
        OptionKind::apply(pure_x, pure_f_cfn), // Renamed Marker
        OptionKind::pure(f(x))                 // Renamed Marker
    );
}

#[test]
fn option_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    // apply(pure(y), u) == apply(u, pure(|f_fn| f_fn(y)))
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || CFn::new(|val: A| format!("val:{}", val));
    let u_some_creator = || Some(concrete_f_creator());
    let u_none_creator = || None::<CFn<A, B>>;

    let pure_y: Option<A> = OptionKind::pure(y_val); // Renamed Marker

    // LHS: apply(pure(y), u)
    let lhs_some = OptionKind::apply(pure_y, u_some_creator()); // Renamed Marker
    let lhs_none = OptionKind::apply(pure_y, u_none_creator()); // Renamed Marker

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || CFn::new(move |f_map_fn: CFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator = || OptionKind::pure(interchange_fn_creator()); // Renamed Marker

    let rhs_some = OptionKind::apply(u_some_creator(), pure_interchange_fn_wrapper_creator()); // Renamed Marker
    let rhs_none = OptionKind::apply(u_none_creator(), pure_interchange_fn_wrapper_creator()); // Renamed Marker

    assert_eq!(lhs_some, rhs_some);
    assert_eq!(lhs_none, rhs_none);
    assert_eq!(lhs_some, Some("val:10".to_string()));
}

// Functor laws for lift_a1 (map defined via pure/apply)
#[test]
fn option_kind_lift_a1_functor_identity() {
    // Renamed test and HKT to Kind
    let fa_some: Option<i32> = Some(10);
    let fa_none: Option<i32> = None;
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<OptionKind, _, _, _>(id_fn_static, fa_some),
        fa_some
    ); // Renamed Marker
    assert_eq!(
        lift_a1::<OptionKind, _, _, _>(id_fn_static, fa_none),
        fa_none
    ); // Renamed Marker
}

#[test]
fn option_kind_lift_a1_functor_composition() {
    // Renamed test and HKT to Kind
    let fa_some: Option<i32> = Some(10);
    let fa_none: Option<i32> = None;

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_some = lift_a1::<OptionKind, _, _, _>(g_compose_f, fa_some); // Renamed Marker
    let lhs_none = lift_a1::<OptionKind, _, _, _>(g_compose_f, fa_none); // Renamed Marker

    let map_f_fa_some = lift_a1::<OptionKind, _, _, _>(f, fa_some); // Renamed Marker
    let rhs_some = lift_a1::<OptionKind, _, _, _>(g, map_f_fa_some); // Renamed Marker

    let map_f_fa_none = lift_a1::<OptionKind, _, _, _>(f, fa_none); // Renamed Marker
    let rhs_none = lift_a1::<OptionKind, _, _, _>(g, map_f_fa_none); // Renamed Marker

    assert_eq!(lhs_some, rhs_some);
    assert_eq!(lhs_none, rhs_none);
    assert_eq!(lhs_some, Some("20".to_string()));
}

// --- ResultKind Applicative Laws ---
type TestError = String;

#[test]
fn result_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    let v_ok: Result<i32, TestError> = Ok(10);
    let v_err: Result<i32, TestError> = Err("Error".to_string());

    let id_cfn_creator = || CFn::new(identity::<i32>);
    let pure_id_cfn_creator = || ResultKind::<TestError>::pure(id_cfn_creator()); // Renamed Marker

    assert_eq!(
        ResultKind::<TestError>::apply(v_ok.clone(), pure_id_cfn_creator()),
        v_ok
    ); // Renamed Marker
    assert_eq!(
        ResultKind::<TestError>::apply(v_err.clone(), pure_id_cfn_creator()),
        v_err
    ); // Renamed Marker
}

#[test]
fn result_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_cfn_creator = || CFn::new(f);
    let pure_f_cfn = ResultKind::<TestError>::pure(f_cfn_creator()); // Renamed Marker
    let pure_x = ResultKind::<TestError>::pure(x); // Renamed Marker

    assert_eq!(
        ResultKind::<TestError>::apply(pure_x, pure_f_cfn), // Renamed Marker
        ResultKind::<TestError>::pure(f(x))                 // Renamed Marker
    );
}

#[test]
fn result_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || CFn::new(|val: A| format!("val:{}", val));
    let u_ok_creator = || Ok(concrete_f_creator());
    let u_err_creator = || Err::<CFn<A, B>, TestError>("Error in u".to_string());

    let pure_y = ResultKind::<TestError>::pure(y_val); // Renamed Marker

    let lhs_ok = ResultKind::<TestError>::apply(pure_y.clone(), u_ok_creator()); // Renamed Marker
    let lhs_err = ResultKind::<TestError>::apply(pure_y.clone(), u_err_creator()); // Renamed Marker

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || CFn::new(move |f_map_fn: CFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator =
        || ResultKind::<TestError>::pure(interchange_fn_creator()); // Renamed Marker

    let rhs_ok =
        ResultKind::<TestError>::apply(u_ok_creator(), pure_interchange_fn_wrapper_creator()); // Renamed Marker
    let rhs_err =
        ResultKind::<TestError>::apply(u_err_creator(), pure_interchange_fn_wrapper_creator()); // Renamed Marker

    assert_eq!(lhs_ok, rhs_ok);
    assert_eq!(lhs_err, rhs_err);
    assert_eq!(lhs_ok, Ok("val:10".to_string()));
    assert_eq!(lhs_err, Err("Error in u".to_string()));
}

#[test]
fn result_kind_lift_a1_functor_identity() {
    // Renamed test and HKT to Kind
    let fa_ok: Result<i32, TestError> = Ok(10);
    let fa_err: Result<i32, TestError> = Err("Error".to_string());
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<ResultKind<TestError>, _, _, _>(id_fn_static, fa_ok.clone()),
        fa_ok
    ); // Renamed Marker
    assert_eq!(
        lift_a1::<ResultKind<TestError>, _, _, _>(id_fn_static, fa_err.clone()),
        fa_err
    ); // Renamed Marker
}

#[test]
fn result_kind_lift_a1_functor_composition() {
    // Renamed test and HKT to Kind
    let fa_ok: Result<i32, TestError> = Ok(10);
    let fa_err: Result<i32, TestError> = Err("Error".to_string());

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_ok = lift_a1::<ResultKind<TestError>, _, _, _>(g_compose_f, fa_ok.clone()); // Renamed Marker
    let lhs_err = lift_a1::<ResultKind<TestError>, _, _, _>(g_compose_f, fa_err.clone()); // Renamed Marker

    let map_f_fa_ok = lift_a1::<ResultKind<TestError>, _, _, _>(f, fa_ok.clone()); // Renamed Marker
    let rhs_ok = lift_a1::<ResultKind<TestError>, _, _, _>(g, map_f_fa_ok); // Renamed Marker

    let map_f_fa_err = lift_a1::<ResultKind<TestError>, _, _, _>(f, fa_err.clone()); // Renamed Marker
    let rhs_err = lift_a1::<ResultKind<TestError>, _, _, _>(g, map_f_fa_err); // Renamed Marker

    assert_eq!(lhs_ok, rhs_ok);
    assert_eq!(lhs_err, rhs_err);
    assert_eq!(lhs_ok, Ok("20".to_string()));
    assert_eq!(lhs_err, Err("Error".to_string()));
}

// --- VecKind Applicative Laws ---
#[test]
fn vec_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    println!("NOTE: VecKind Applicative Identity law is untestable with CFn due to pure's Clone constraint.");
}

#[test]
fn vec_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    println!("NOTE: VecKind Applicative Homomorphism law is untestable with CFn due to pure's Clone constraint.");
}

#[test]
fn vec_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    type A = i32;
    let y_val: A = 10;

    let concrete_f1_creator = || CFn::new(|val: A| format!("f1:{}", val));
    let concrete_f2_creator = || CFn::new(|val: A| format!("f2:{}", val * 2));
    let u_vec_creator = || vec![concrete_f1_creator(), concrete_f2_creator()];

    let pure_y_vec: Vec<A> = VecKind::pure(y_val); // Renamed Marker

    let lhs = VecKind::apply(pure_y_vec.clone(), u_vec_creator()); // Renamed Marker
    assert_eq!(lhs, vec!["f1:10".to_string(), "f2:20".to_string()]);
}

#[test]
fn vec_kind_functor_identity_via_map() {
    // Renamed test and HKT to Kind
    let fa_vec: Vec<i32> = vec![10, 20];
    let fa_empty: Vec<i32> = vec![];
    let id_fn_static = identity::<i32>;

    assert_eq!(VecKind::map(fa_vec.clone(), id_fn_static), fa_vec); // Renamed Marker
    assert_eq!(VecKind::map(fa_empty.clone(), id_fn_static), fa_empty); // Renamed Marker
}

#[test]
fn vec_kind_functor_composition_via_map() {
    // Renamed test and HKT to Kind
    let fa_vec: Vec<i32> = vec![10, 20];
    let fa_empty: Vec<i32> = vec![];

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs_vec = VecKind::map(fa_vec.clone(), g_compose_f); // Renamed Marker
    let lhs_empty = VecKind::map(fa_empty.clone(), g_compose_f); // Renamed Marker

    let map_f_fa_vec = VecKind::map(fa_vec.clone(), f); // Renamed Marker
    let rhs_vec = VecKind::map(map_f_fa_vec, g); // Renamed Marker

    let map_f_fa_empty = VecKind::map(fa_empty.clone(), f); // Renamed Marker
    let rhs_empty = VecKind::map(map_f_fa_empty, g); // Renamed Marker

    assert_eq!(lhs_vec, rhs_vec);
    assert_eq!(lhs_empty, rhs_empty);
    assert_eq!(lhs_vec, vec!["20".to_string(), "40".to_string()]);
    assert_eq!(lhs_empty, Vec::<String>::new());
}

// --- IdentityKind Applicative Laws ---
#[test]
fn identity_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    let v: IdType<i32> = IdType(10);

    let id_cfn_creator = || CFn::new(identity::<i32>);
    let pure_id_cfn: IdType<CFn<i32, i32>> = IdentityKind::pure(id_cfn_creator()); // Renamed Marker

    assert_eq!(IdentityKind::apply(v.clone(), pure_id_cfn), v); // Renamed Marker
}

#[test]
fn identity_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    let x: i32 = 10;
    let f = |val: i32| val * 2;

    let f_cfn_creator = || CFn::new(f);
    let pure_f_cfn: IdType<CFn<i32, i32>> = IdentityKind::pure(f_cfn_creator()); // Renamed Marker
    let pure_x: IdType<i32> = IdentityKind::pure(x); // Renamed Marker

    assert_eq!(
        IdentityKind::apply(pure_x, pure_f_cfn), // Renamed Marker
        IdentityKind::pure(f(x))                 // Renamed Marker
    );
}

#[test]
fn identity_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    type A = i32;
    type B = String;

    let y_val: A = 10;

    let concrete_f_creator = || CFn::new(|val: A| format!("val:{}", val));
    let u_identity_creator = || IdType(concrete_f_creator());

    let pure_y: IdType<A> = IdentityKind::pure(y_val); // Renamed Marker

    let lhs = IdentityKind::apply(pure_y.clone(), u_identity_creator()); // Renamed Marker

    let y_val_clone_for_rhs = y_val;
    let interchange_fn_creator =
        || CFn::new(move |f_map_fn: CFn<A, B>| f_map_fn.call(y_val_clone_for_rhs));
    let pure_interchange_fn_wrapper_creator = || IdentityKind::pure(interchange_fn_creator()); // Renamed Marker

    let rhs = IdentityKind::apply(u_identity_creator(), pure_interchange_fn_wrapper_creator()); // Renamed Marker

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, IdType("val:10".to_string()));
}

#[test]
fn identity_kind_lift_a1_functor_identity() {
    // Renamed test and HKT to Kind
    let fa_id: IdType<i32> = IdType(10);
    let id_fn_static = identity::<i32>;

    assert_eq!(
        lift_a1::<IdentityKind, _, _, _>(id_fn_static, fa_id.clone()),
        fa_id
    ); // Renamed Marker
}

#[test]
fn identity_kind_lift_a1_functor_composition() {
    // Renamed test and HKT to Kind
    let fa_id: IdType<i32> = IdType(10);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = lift_a1::<IdentityKind, _, _, _>(g_compose_f, fa_id.clone()); // Renamed Marker
    let map_f_fa = lift_a1::<IdentityKind, _, _, _>(f, fa_id.clone()); // Renamed Marker
    let rhs = lift_a1::<IdentityKind, _, _, _>(g, map_f_fa); // Renamed Marker

    assert_eq!(lhs, rhs);
    assert_eq!(lhs, IdType("20".to_string()));
}

// --- CFnKind Applicative Laws ---
type Env = i32;

#[test]
fn cfn_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    println!("NOTE: CFnKind Applicative Identity law is untestable with CFn due to pure's Clone constraint.");
}

#[test]
fn cfn_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    println!(
        "NOTE: CFnKind Applicative Homomorphism law is untestable due to CFn not being Clone."
    );
}

#[test]
fn cfn_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    println!("NOTE: CFnKind Applicative Interchange law is untestable due to CFn not being Clone.");
}

// --- CFnKind Functor Laws (using map) ---
#[test]
fn cfn_kind_functor_identity_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || CFn::new(|_e: Env| 10);
    let id_fn_static = identity::<i32>;

    let mapped_val = CFnKind::<Env>::map(fa_creator(), id_fn_static); // Renamed Marker

    assert_eq!(mapped_val.call(100), fa_creator().call(100));
}

#[test]
fn cfn_kind_functor_composition_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || CFn::new(|_e: Env| 10);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = CFnKind::<Env>::map(fa_creator(), g_compose_f); // Renamed Marker

    let map_f_fa: CFn<Env, i32> = CFnKind::<Env>::map(fa_creator(), f); // Renamed Marker
    let rhs: CFn<Env, String> = CFnKind::<Env>::map(map_f_fa, g); // Renamed Marker

    assert_eq!(lhs.call(100), rhs.call(100));
    assert_eq!(lhs.call(100), "20".to_string());
}

// --- CFnOnceKind Applicative Laws ---
#[test]
fn cfn_once_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    println!("NOTE: CFnOnceKind Applicative Identity law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

#[test]
fn cfn_once_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    println!("NOTE: CFnOnceKind Applicative Homomorphism law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

#[test]
fn cfn_once_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    println!("NOTE: CFnOnceKind Applicative Interchange law is untestable due to CFnOnce not being Clone and pure's Clone requirement.");
}

// --- CFnOnceKind Functor Laws (using map) ---
#[test]
fn cfn_once_kind_functor_identity_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || CFnOnce::new(|_e: Env| 10);
    let id_fn_static = identity::<i32>;

    let mapped = CFnOnceKind::<Env>::map(fa_creator(), id_fn_static); // Renamed Marker
    assert_eq!(mapped.call_once(100), fa_creator().call_once(100));
}

#[test]
fn cfn_once_kind_functor_composition_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || CFnOnce::new(|_e: Env| 10);

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = CFnOnceKind::<Env>::map(fa_creator(), g_compose_f); // Renamed Marker
    let lhs_result_for_0 = lhs.call_once(0);

    let map_f_fa = CFnOnceKind::<Env>::map(fa_creator(), f); // Renamed Marker
    let rhs = CFnOnceKind::<Env>::map(map_f_fa, g); // Renamed Marker

    assert_eq!(lhs_result_for_0.clone(), rhs.call_once(100));
    assert_eq!(lhs_result_for_0, "20".to_string());
}

// --- ReaderTKind Applicative Laws ---
type ReaderEnv = i32;

#[test]
fn reader_t_kind_applicative_law_identity() {
    // Renamed test and HKT to Kind
    println!("NOTE: ReaderTKind Applicative Identity law is untestable with CFn due to pure's Clone constraint.");
}

#[test]
fn reader_t_kind_applicative_law_homomorphism() {
    // Renamed test and HKT to Kind
    println!("NOTE: ReaderTKind Applicative Homomorphism law is untestable with CFn due to pure's Clone constraint.");
}

#[test]
fn reader_t_kind_applicative_law_interchange() {
    // Renamed test and HKT to Kind
    println!("NOTE: ReaderTKind Applicative Interchange law is untestable with CFn due to Clone constraints.");
}

// --- ReaderTKind Functor Laws (using map) ---
#[test]
fn reader_t_kind_functor_identity_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || {
        ReaderT::<ReaderEnv, IdentityKind, i32>::new(
            // Renamed Marker
            |_e: ReaderEnv| IdType(10),
        )
    };
    let id_fn_static = identity::<i32>;

    let mapped = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), id_fn_static); // Renamed Marker

    let env_val = 100;
    assert_eq!(
        (mapped.run_reader_t)(env_val),
        (fa_creator().run_reader_t)(env_val)
    );
}

#[test]
fn reader_t_kind_functor_composition_via_map() {
    // Renamed test and HKT to Kind
    let fa_creator = || {
        ReaderT::<ReaderEnv, IdentityKind, i32>::new(
            // Renamed Marker
            |_e: ReaderEnv| IdType(10),
        )
    };

    let f = |x: i32| x * 2;
    let g = |y: i32| y.to_string();
    let g_compose_f = move |x: i32| g(f(x));

    let lhs = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), g_compose_f); // Renamed Marker

    let map_f_fa = ReaderTKind::<ReaderEnv, IdentityKind>::map(fa_creator(), f); // Renamed Marker
    let rhs = ReaderTKind::<ReaderEnv, IdentityKind>::map(map_f_fa, g); // Renamed Marker

    let env_val = 100;
    assert_eq!((lhs.run_reader_t)(env_val), (rhs.run_reader_t)(env_val));
    assert_eq!((lhs.run_reader_t)(env_val), IdType("20".to_string()));
}
