use monadify::applicative::kind::Applicative; // Changed hkt to kind
use monadify::function::{CFnOnce, RcFn};
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::kind_based::kind::{CFnOnceKind, OptionKind, RcFnKind, ResultKind, VecKind};
use monadify::monad::kind::{Bind, Monad}; // Changed hkt to kind

// Common error type for Result tests
type TestError = String;
type TestResult<T> = Result<T, TestError>;

// Helper for creating a cloneable FnMut for bind
fn clone_fn<A, B, F>(f: F) -> impl FnMut(A) -> B + Clone + 'static
where
    F: Fn(A) -> B + Clone + 'static,
    A: 'static,
    B: 'static,
{
    f
}

// Unused helper `once_fn` removed.

mod result_kind_monad_laws {
    // Renamed module
    use super::*;

    // 1. Left Identity: ResultKind::pure(a).bind(f) == f(a)
    #[test]
    fn result_kind_monad_left_identity_ok() {
        // Renamed test
        let a: i32 = 10;
        let f = clone_fn(|x: i32| -> TestResult<String> { Ok((x * 2).to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::pure(a), // Renamed Marker
            f.clone(),
        );
        let rhs = f.clone()(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok("20".to_string()));
    }

    #[test]
    fn result_kind_monad_left_identity_f_returns_err() {
        // Renamed test
        let a: i32 = 10;
        let f = clone_fn(|_x: i32| -> TestResult<String> { Err("f_error".to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::pure(a), // Renamed Marker
            f.clone(),
        );
        let rhs = f.clone()(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("f_error".to_string()));
    }

    // 2. Right Identity: m.bind(ResultKind::pure) == m
    #[test]
    fn result_kind_monad_right_identity_ok() {
        // Renamed test
        let m: TestResult<i32> = Ok(10);
        let pure_fn = clone_fn(|val: i32| ResultKind::<TestError>::pure(val)); // Renamed Marker

        let lhs = ResultKind::<TestError>::bind(m.clone(), pure_fn); // Renamed Marker
        let rhs = m;

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn result_kind_monad_right_identity_err() {
        // Renamed test
        let m: TestResult<i32> = Err("m_error".to_string());
        let pure_fn = clone_fn(|val: i32| ResultKind::<TestError>::pure(val)); // Renamed Marker

        let lhs = ResultKind::<TestError>::bind(m.clone(), pure_fn); // Renamed Marker
        let rhs = m;

        assert_eq!(lhs, rhs);
    }

    // 3. Associativity: m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    #[test]
    fn result_kind_monad_associativity_all_ok() {
        // Renamed test
        let m: TestResult<i32> = Ok(10);
        let f = clone_fn(|x: i32| -> TestResult<f64> { Ok((x * 2) as f64) });
        let g = clone_fn(|y: f64| -> TestResult<String> { Ok(y.to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::bind(m.clone(), f.clone()), // Renamed Marker
            g.clone(),
        );

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x: i32| {
            ResultKind::<TestError>::bind(f_inner.clone()(x), g_inner.clone()) // Renamed Marker
        });
        let rhs = ResultKind::<TestError>::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok("20".to_string()));
    }

    #[test]
    fn result_kind_monad_associativity_m_is_err() {
        // Renamed test
        let m: TestResult<i32> = Err("m_error".to_string());
        let f = clone_fn(|x: i32| -> TestResult<f64> { Ok((x * 2) as f64) });
        let g = clone_fn(|y: f64| -> TestResult<String> { Ok(y.to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::bind(m.clone(), f.clone()), // Renamed Marker
            g.clone(),
        );

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x: i32| {
            ResultKind::<TestError>::bind(f_inner.clone()(x), g_inner.clone()) // Renamed Marker
        });
        let rhs = ResultKind::<TestError>::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("m_error".to_string()));
    }

    #[test]
    fn result_kind_monad_associativity_f_returns_err() {
        // Renamed test
        let m: TestResult<i32> = Ok(10);
        let f = clone_fn(|_x: i32| -> TestResult<f64> { Err("f_error".to_string()) });
        let g = clone_fn(|y: f64| -> TestResult<String> { Ok(y.to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::bind(m.clone(), f.clone()), // Renamed Marker
            g.clone(),
        );

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x: i32| {
            ResultKind::<TestError>::bind(f_inner.clone()(x), g_inner.clone()) // Renamed Marker
        });
        let rhs = ResultKind::<TestError>::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("f_error".to_string()));
    }

    #[test]
    fn result_kind_monad_associativity_g_returns_err() {
        // Renamed test
        let m: TestResult<i32> = Ok(10);
        let f = clone_fn(|x: i32| -> TestResult<f64> { Ok((x * 2) as f64) });
        let g = clone_fn(|_y: f64| -> TestResult<String> { Err("g_error".to_string()) });

        let lhs = ResultKind::<TestError>::bind(
            // Renamed Marker
            ResultKind::<TestError>::bind(m.clone(), f.clone()), // Renamed Marker
            g.clone(),
        );

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x: i32| {
            ResultKind::<TestError>::bind(f_inner.clone()(x), g_inner.clone()) // Renamed Marker
        });
        let rhs = ResultKind::<TestError>::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("g_error".to_string()));
    }

    // Monad::join laws for ResultKind
    #[test]
    fn result_kind_monad_join_law1() {
        // Renamed test
        let x = 10;
        let mma: TestResult<TestResult<i32>> =
            ResultKind::<TestError>::pure(ResultKind::<TestError>::pure(x)); // Renamed Marker

        let lhs = ResultKind::<TestError>::join(mma); // Renamed Marker
        let rhs = ResultKind::<TestError>::pure(x); // Renamed Marker
        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok(10));
    }

    #[test]
    fn result_kind_monad_join_law1_outer_err() {
        // Renamed test
        let mma: TestResult<TestResult<i32>> = Err("outer_error".to_string());
        let lhs = ResultKind::<TestError>::join(mma); // Renamed Marker
        assert_eq!(lhs, Err("outer_error".to_string()));
    }

    #[test]
    fn result_kind_monad_join_law1_inner_err() {
        // Renamed test
        let mma: TestResult<TestResult<i32>> =
            ResultKind::<TestError>::pure(Err("inner_error".to_string())); // Renamed Marker
        let lhs = ResultKind::<TestError>::join(mma); // Renamed Marker
        assert_eq!(lhs, Err("inner_error".to_string()));
    }

    #[test]
    fn result_kind_monad_join_law2() {
        // Renamed test
        let m_ok: TestResult<i32> = Ok(10);
        let m_err: TestResult<i32> = Err("m_error".to_string());

        let pure_fn = clone_fn(|val: i32| ResultKind::<TestError>::pure(val)); // Renamed Marker

        let mapped_m_ok = ResultKind::<TestError>::map(m_ok.clone(), pure_fn.clone()); // Renamed Marker
        assert_eq!(ResultKind::<TestError>::join(mapped_m_ok), m_ok); // Renamed Marker

        let mapped_m_err = ResultKind::<TestError>::map(m_err.clone(), pure_fn.clone()); // Renamed Marker
        assert_eq!(ResultKind::<TestError>::join(mapped_m_err), m_err); // Renamed Marker
    }

    #[test]
    fn result_kind_monad_join_law3() {
        // Renamed test
        let m_ok: TestResult<i32> = Ok(10);
        let m_err: TestResult<i32> = Err("m_error".to_string());

        let pure_m_ok = ResultKind::<TestError>::pure(m_ok.clone()); // Renamed Marker
        assert_eq!(ResultKind::<TestError>::join(pure_m_ok), m_ok); // Renamed Marker

        let pure_m_err = ResultKind::<TestError>::pure(m_err.clone()); // Renamed Marker
        assert_eq!(ResultKind::<TestError>::join(pure_m_err), m_err); // Renamed Marker
    }
}

mod option_kind_join_tests {
    // Renamed module
    use super::*;

    #[test]
    fn option_kind_join_some_some() {
        // Renamed test
        let mma: Option<Option<i32>> = Some(Some(10));
        assert_eq!(OptionKind::join(mma), Some(10)); // Renamed Marker
    }

    #[test]
    fn option_kind_join_some_none() {
        // Renamed test
        let mma: Option<Option<i32>> = Some(None);
        assert_eq!(OptionKind::join(mma), None); // Renamed Marker
    }

    #[test]
    fn option_kind_join_none() {
        // Renamed test
        let mma: Option<Option<i32>> = None;
        assert_eq!(OptionKind::join(mma), None); // Renamed Marker
    }
}

mod vec_kind_monad_laws {
    // Renamed module
    use super::*;

    // 1. Left Identity: VecKind::pure(a).bind(f) == f(a)
    #[test]
    fn vec_kind_monad_left_identity() {
        // Renamed test
        let a: i32 = 10;
        let f = clone_fn(|x: i32| -> Vec<String> { vec![x.to_string(), (x + 1).to_string()] });

        let lhs = VecKind::bind(VecKind::pure(a), f.clone()); // Renamed Marker
        let rhs = f.clone()(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, vec!["10".to_string(), "11".to_string()]);
    }

    #[test]
    fn vec_kind_monad_left_identity_f_returns_empty() {
        // Renamed test
        let a: i32 = 10;
        let f = clone_fn(|_x: i32| -> Vec<String> { vec![] });

        let lhs = VecKind::bind(VecKind::pure(a), f.clone()); // Renamed Marker
        let rhs = f.clone()(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    // 2. Right Identity: m.bind(VecKind::pure) == m
    #[test]
    fn vec_kind_monad_right_identity_non_empty() {
        // Renamed test
        let m: Vec<i32> = vec![10, 20];
        let pure_fn = clone_fn(|val: i32| VecKind::pure(val)); // Renamed Marker

        let lhs = VecKind::bind(m.clone(), pure_fn); // Renamed Marker
        let rhs = m;

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn vec_kind_monad_right_identity_empty() {
        // Renamed test
        let m: Vec<i32> = vec![];
        let pure_fn = clone_fn(|val: i32| VecKind::pure(val)); // Renamed Marker

        let lhs = VecKind::bind(m.clone(), pure_fn); // Renamed Marker
        let rhs = m;

        assert_eq!(lhs, rhs);
    }

    // 3. Associativity: m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    #[test]
    fn vec_kind_monad_associativity() {
        // Renamed test
        let m: Vec<i32> = vec![1, 2];
        let f = clone_fn(|x: i32| -> Vec<i32> { vec![x, x * 10] });
        let g = clone_fn(|y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] });

        let lhs = VecKind::bind(
            // Renamed Marker
            VecKind::bind(m.clone(), f.clone()), // Renamed Marker
            g.clone(),
        );

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x: i32| {
            VecKind::bind(f_inner.clone()(x), g_inner.clone()) // Renamed Marker
        });
        let rhs = VecKind::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(
            lhs,
            vec![
                "1".to_string(),
                "2".to_string(),
                "10".to_string(),
                "11".to_string(),
                "2".to_string(),
                "3".to_string(),
                "20".to_string(),
                "21".to_string()
            ]
        );
    }

    #[test]
    fn vec_kind_monad_associativity_empty_start() {
        // Renamed test
        let m: Vec<i32> = vec![];
        let f = clone_fn(|x: i32| -> Vec<i32> { vec![x, x * 10] });
        let g = clone_fn(|y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] });

        let lhs = VecKind::bind(VecKind::bind(m.clone(), f.clone()), g.clone()); // Renamed Marker
        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func =
            clone_fn(move |x: i32| VecKind::bind(f_inner.clone()(x), g_inner.clone())); // Renamed Marker
        let rhs = VecKind::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    #[test]
    fn vec_kind_monad_associativity_f_returns_empty() {
        // Renamed test
        let m: Vec<i32> = vec![1, 2];
        let f = clone_fn(|_x: i32| -> Vec<i32> { vec![] });
        let g = clone_fn(|y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] });

        let lhs = VecKind::bind(VecKind::bind(m.clone(), f.clone()), g.clone()); // Renamed Marker
        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func =
            clone_fn(move |x: i32| VecKind::bind(f_inner.clone()(x), g_inner.clone())); // Renamed Marker
        let rhs = VecKind::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    #[test]
    fn vec_kind_monad_associativity_g_returns_empty() {
        // Renamed test
        let m: Vec<i32> = vec![1, 2];
        let f = clone_fn(|x: i32| -> Vec<i32> { vec![x, x * 10] });
        let g = clone_fn(|_y: i32| -> Vec<String> { vec![] });

        let lhs = VecKind::bind(VecKind::bind(m.clone(), f.clone()), g.clone()); // Renamed Marker
        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func =
            clone_fn(move |x: i32| VecKind::bind(f_inner.clone()(x), g_inner.clone())); // Renamed Marker
        let rhs = VecKind::bind(m.clone(), composed_func); // Renamed Marker

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    // Monad::join laws for VecKind
    #[test]
    fn vec_kind_monad_join_law1() {
        // Renamed test
        let x = 10;
        let mma: Vec<Vec<i32>> = VecKind::pure(VecKind::pure(x)); // Renamed Marker

        let lhs = VecKind::join(mma); // Renamed Marker
        let rhs = VecKind::pure(x); // Renamed Marker
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn vec_kind_monad_join_law2() {
        // Renamed test
        let m_non_empty: Vec<i32> = vec![10, 20];
        let m_empty: Vec<i32> = vec![];

        let pure_fn = clone_fn(|val: i32| VecKind::pure(val)); // Renamed Marker

        let mapped_m_non_empty = VecKind::map(m_non_empty.clone(), pure_fn.clone()); // Renamed Marker
        assert_eq!(VecKind::join(mapped_m_non_empty), m_non_empty); // Renamed Marker

        let mapped_m_empty = VecKind::map(m_empty.clone(), pure_fn.clone()); // Renamed Marker
        assert_eq!(VecKind::join(mapped_m_empty), m_empty); // Renamed Marker
    }

    #[test]
    fn vec_kind_monad_join_law3() {
        // Renamed test
        let m_non_empty: Vec<i32> = vec![10, 20];
        let m_empty: Vec<i32> = vec![];

        let pure_m_non_empty = VecKind::pure(m_non_empty.clone()); // Renamed Marker
        assert_eq!(VecKind::join(pure_m_non_empty), m_non_empty); // Renamed Marker

        let pure_m_empty = VecKind::pure(m_empty.clone()); // Renamed Marker
        assert_eq!(VecKind::join(pure_m_empty), m_empty); // Renamed Marker
    }

    #[test]
    fn vec_kind_join_specific_examples() {
        // Renamed test
        assert_eq!(
            VecKind::join(vec![vec![1, 2], vec![3, 4]]),
            vec![1, 2, 3, 4]
        ); // Renamed Marker
        assert_eq!(VecKind::join(vec![vec![], vec![3, 4]]), vec![3, 4]); // Renamed Marker
        assert_eq!(VecKind::join(vec![vec![1, 2], vec![]]), vec![1, 2]); // Renamed Marker
        assert_eq!(VecKind::join(Vec::<Vec<i32>>::new()), Vec::<i32>::new()); // Renamed Marker
        assert_eq!(
            VecKind::join(vec![Vec::<i32>::new(), Vec::<i32>::new()]),
            Vec::<i32>::new()
        ); // Renamed Marker
    }
}

mod rcfn_kind_monad_laws {
    use super::*;
    type Env = i32;

    // 1. Left Identity: RcFnKind::pure(a).bind(f) == f(a)
    #[test]
    fn rcfn_kind_monad_left_identity() {
        let env_val: Env = 5;
        let a: i32 = 10;

        let f = clone_fn(move |x: i32| -> RcFn<Env, String> {
            RcFn::new(move |env: Env| (x + env).to_string())
        });

        let pure_a: RcFn<Env, i32> = RcFnKind::<Env>::pure(a);
        let lhs: RcFn<Env, String> = RcFnKind::<Env>::bind(pure_a, f.clone());

        let rhs: RcFn<Env, String> = f.clone()(a);

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "15".to_string());
    }

    // 2. Right Identity: m.bind(RcFnKind::pure) == m
    #[test]
    fn rcfn_kind_monad_right_identity() {
        let env_val: Env = 7;
        let m_creator = || RcFn::new(move |env: Env| env * 2);

        let pure_fn = clone_fn(|val: i32| RcFnKind::<Env>::pure(val));

        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::bind(m_creator(), pure_fn);
        let rhs: RcFn<Env, i32> = m_creator();

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), 14);
    }

    // 3. Associativity
    #[test]
    fn rcfn_kind_monad_associativity() {
        let env_val: Env = 3;
        let m_creator = || RcFn::new(move |env: Env| env + 1);

        let f = clone_fn(move |x: i32| -> RcFn<Env, f64> {
            RcFn::new(move |env: Env| (x * env) as f64)
        });

        let g = clone_fn(move |y: f64| -> RcFn<Env, String> {
            RcFn::new(move |env: Env| (y + (env as f64)).to_string())
        });

        let bound_f: RcFn<Env, f64> = RcFnKind::<Env>::bind(m_creator(), f.clone());
        let lhs: RcFn<Env, String> = RcFnKind::<Env>::bind(bound_f, g.clone());

        let f_inner = f.clone();
        let g_inner = g.clone();
        let composed_func = clone_fn(move |x_val: i32| -> RcFn<Env, String> {
            let fx: RcFn<Env, f64> = f_inner.clone()(x_val);
            RcFnKind::<Env>::bind(fx, g_inner.clone())
        });
        let rhs: RcFn<Env, String> = RcFnKind::<Env>::bind(m_creator(), composed_func);

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "15".to_string());
    }

    // Monad::join laws for RcFnKind
    #[test]
    fn rcfn_kind_monad_join_law1() {
        let env_val: Env = 5;
        let x: i32 = 10;

        let mma: RcFn<Env, RcFn<Env, i32>> =
            RcFn::new(move |_env_outer: Env| RcFnKind::<Env>::pure(x));

        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::join(mma);
        let rhs: RcFn<Env, i32> = RcFnKind::<Env>::pure(x);

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), 10);
    }

    #[test]
    fn rcfn_kind_monad_join_law2() {
        let env_val: Env = 7;
        let m_creator = || RcFn::new(move |env: Env| env * 3);

        let pure_fn = clone_fn(|val: i32| RcFnKind::<Env>::pure(val));

        let mapped_m: RcFn<Env, RcFn<Env, i32>> = RcFnKind::<Env>::map(m_creator(), pure_fn);

        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::join(mapped_m);
        let rhs: RcFn<Env, i32> = m_creator();

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), 21);
    }
}

mod cfn_once_kind_monad_laws {
    // Renamed module
    use super::*;
    type Env = i32;

    // 1. Left Identity: CFnOnceKind::pure(a).bind(f) == f(a)
    #[test]
    fn cfn_once_kind_monad_left_identity() {
        // Renamed test
        let env_val: Env = 5;
        let a: i32 = 10;

        let f = |x: i32| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (x + env).to_string())
        };

        let pure_a_cfn_once: CFnOnce<Env, i32> = CFnOnceKind::<Env>::pure(a); // Renamed Marker
        let lhs_cfn_once: CFnOnce<Env, String> = CFnOnceKind::<Env>::bind(pure_a_cfn_once, f); // Renamed Marker

        let f_for_rhs = |x: i32| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (x + env).to_string())
        };
        let rhs_cfn_once: CFnOnce<Env, String> = f_for_rhs(a);

        assert_eq!(
            lhs_cfn_once.call_once(env_val),
            rhs_cfn_once.call_once(env_val)
        );

        let f_for_assert = |x: i32| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (x + env).to_string())
        };
        let pure_a_for_assert: CFnOnce<Env, i32> = CFnOnceKind::<Env>::pure(a); // Renamed Marker
        assert_eq!(
            CFnOnceKind::<Env>::bind(pure_a_for_assert, f_for_assert).call_once(env_val),
            "15".to_string()
        ); // Renamed Marker
    }

    // 2. Right Identity: m.bind(CFnOnceKind::pure) == m
    #[test]
    fn cfn_once_kind_monad_right_identity() {
        // Renamed test
        let env_val: Env = 7;
        let m_creator = || CFnOnce::new(move |env: Env| env * 2);

        let pure_fn = |val: i32| CFnOnceKind::<Env>::pure(val); // Renamed Marker

        let lhs_cfn_once: CFnOnce<Env, i32> = CFnOnceKind::<Env>::bind(m_creator(), pure_fn); // Renamed Marker
        let rhs_cfn_once: CFnOnce<Env, i32> = m_creator();

        assert_eq!(
            lhs_cfn_once.call_once(env_val),
            rhs_cfn_once.call_once(env_val)
        );
        assert_eq!(m_creator().call_once(env_val), 14);
    }

    // 3. Associativity: m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    #[test]
    fn cfn_once_kind_monad_associativity() {
        // Renamed test
        let env_val: Env = 3;
        let m_creator = || CFnOnce::new(move |env: Env| env + 1);

        let f = |x: i32| -> CFnOnce<Env, f64> { CFnOnce::new(move |env: Env| (x * env) as f64) };
        let g = |y: f64| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (y + (env as f64)).to_string())
        };

        let bound_f: CFnOnce<Env, f64> = CFnOnceKind::<Env>::bind(m_creator(), f); // Renamed Marker
        let lhs_cfn_once: CFnOnce<Env, String> = CFnOnceKind::<Env>::bind(bound_f, g); // Renamed Marker

        let f_for_rhs =
            |x: i32| -> CFnOnce<Env, f64> { CFnOnce::new(move |env: Env| (x * env) as f64) };
        let g_for_rhs = |y: f64| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (y + (env as f64)).to_string())
        };
        let composed_func = move |x_val: i32| -> CFnOnce<Env, String> {
            let fx: CFnOnce<Env, f64> = f_for_rhs(x_val);
            CFnOnceKind::<Env>::bind(fx, g_for_rhs) // Renamed Marker
        };
        let rhs_cfn_once: CFnOnce<Env, String> =
            CFnOnceKind::<Env>::bind(m_creator(), composed_func); // Renamed Marker

        assert_eq!(
            lhs_cfn_once.call_once(env_val),
            rhs_cfn_once.call_once(env_val)
        );

        let m_check = || CFnOnce::new(move |env: Env| env + 1);
        let f_check =
            |x: i32| -> CFnOnce<Env, f64> { CFnOnce::new(move |env: Env| (x * env) as f64) };
        let g_check = |y: f64| -> CFnOnce<Env, String> {
            CFnOnce::new(move |env: Env| (y + (env as f64)).to_string())
        };
        let bound_f_check = CFnOnceKind::<Env>::bind(m_check(), f_check); // Renamed Marker
        assert_eq!(
            CFnOnceKind::<Env>::bind(bound_f_check, g_check).call_once(env_val),
            "15".to_string()
        ); // Renamed Marker
    }

    // Monad::join laws for CFnOnceKind
    #[test]
    fn cfn_once_kind_monad_join_law1() {
        // Renamed test
        let env_val: Env = 5;
        let x: i32 = 10;

        let mma: CFnOnce<Env, CFnOnce<Env, i32>> =
            CFnOnce::new(move |_env_outer: Env| CFnOnceKind::<Env>::pure(x)); // Renamed Marker

        let lhs_cfn_once: CFnOnce<Env, i32> = CFnOnceKind::<Env>::join(mma); // Renamed Marker
        let rhs_cfn_once: CFnOnce<Env, i32> = CFnOnceKind::<Env>::pure(x); // Renamed Marker

        assert_eq!(
            lhs_cfn_once.call_once(env_val),
            rhs_cfn_once.call_once(env_val)
        );
        assert_eq!(CFnOnceKind::<Env>::pure(x).call_once(env_val), 10); // Renamed Marker
    }

    #[test]
    fn cfn_once_kind_monad_join_law2() {
        // Renamed test
        let env_val: Env = 7;
        let m_creator = || CFnOnce::new(move |env: Env| env * 3);

        let pure_fn = |val: i32| CFnOnceKind::<Env>::pure(val); // Renamed Marker

        let mapped_m_cfn_once: CFnOnce<Env, CFnOnce<Env, i32>> =
            CFnOnceKind::<Env>::map(m_creator(), pure_fn); // Renamed Marker

        let lhs_cfn_once: CFnOnce<Env, i32> = CFnOnceKind::<Env>::join(mapped_m_cfn_once); // Renamed Marker
        let rhs_cfn_once: CFnOnce<Env, i32> = m_creator();

        assert_eq!(
            lhs_cfn_once.call_once(env_val),
            rhs_cfn_once.call_once(env_val)
        );
        assert_eq!(m_creator().call_once(env_val), 21);
    }
}
