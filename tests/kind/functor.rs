use monadify::function::{CFnOnce, RcFn};
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::identity::{Identity, IdentityKind}; // Changed IdentityHKTMarker to IdentityKind
use monadify::kind_based::kind::{CFnOnceKind, OptionKind, RcFnKind, ResultKind, VecKind};
use monadify::transformers::reader::{ReaderT, ReaderTKind}; // Changed ReaderTHKTMarker to ReaderTKind
                                                            // Kind1 import might be needed if supertraits are checked explicitly, but Functor itself implies Kind1.

// Common error type for Result tests
type TestError = String;
type TestResult<T> = Result<T, TestError>;

// Helper for creating a cloneable FnMut for map
fn clone_fn_map<A, B, F>(f: F) -> impl FnMut(A) -> B + Clone + 'static
where
    F: Fn(A) -> B + Clone + 'static,
    A: 'static,
    B: 'static,
{
    f
}

// Unused helper `once_fn_map` removed.

pub mod option_kind_functor_laws {
    // Renamed module
    use super::*;

    // Identity law: OptionKind::map(opt, |x| x) == opt
    #[test]
    fn option_kind_functor_identity_some() {
        // Renamed test
        let opt = Some(10);
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(OptionKind::map(opt, identity_fn), opt); // Renamed Marker
    }

    #[test]
    fn option_kind_functor_identity_none() {
        // Renamed test
        let opt: Option<i32> = None;
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(OptionKind::map(opt, identity_fn), opt); // Renamed Marker
    }

    // Composition law: OptionKind::map(opt, |x| g(f(x))) == OptionKind::map(OptionKind::map(opt, f), g)
    #[test]
    fn option_kind_functor_composition_some() {
        // Renamed test
        let opt = Some(10);
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = OptionKind::map(opt, move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker
        let sequential_map = OptionKind::map(OptionKind::map(opt, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Some(25));
    }

    #[test]
    fn option_kind_functor_composition_none() {
        // Renamed test
        let opt: Option<i32> = None;
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = OptionKind::map(opt, move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker
        let sequential_map = OptionKind::map(OptionKind::map(opt, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, None);
    }

    #[test]
    fn option_kind_functor_composition_some_str() {
        // Renamed test
        let opt = Some("hello");
        let f = clone_fn_map(|x: &str| x.to_uppercase());
        let g = clone_fn_map(|y: String| y.len());

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = OptionKind::map(opt, move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker

        let opt_for_seq = Some("hello");
        let sequential_map = OptionKind::map(OptionKind::map(opt_for_seq, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Some(5));
    }
}

pub mod result_kind_functor_laws {
    // Renamed module
    use super::*;

    // Identity law: ResultKind::map(res, |x| x) == res
    #[test]
    fn result_kind_functor_identity_ok() {
        // Renamed test
        let res: TestResult<i32> = Ok(10);
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(ResultKind::<TestError>::map(res.clone(), identity_fn), res);
        // Renamed Marker
    }

    #[test]
    fn result_kind_functor_identity_err() {
        // Renamed test
        let res: TestResult<i32> = Err("error".to_string());
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(ResultKind::<TestError>::map(res.clone(), identity_fn), res);
        // Renamed Marker
    }

    // Composition law: ResultKind::map(res, |x| g(f(x))) == ResultKind::map(ResultKind::map(res, f), g)
    #[test]
    fn result_kind_functor_composition_ok() {
        // Renamed test
        let res: TestResult<i32> = Ok(10);
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map =
            ResultKind::<TestError>::map(res.clone(), move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker
        let sequential_map = ResultKind::<TestError>::map(ResultKind::<TestError>::map(res, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Ok(25));
    }

    #[test]
    fn result_kind_functor_composition_err() {
        // Renamed test
        let res: TestResult<i32> = Err("error".to_string());
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map =
            ResultKind::<TestError>::map(res.clone(), move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker
        let sequential_map = ResultKind::<TestError>::map(ResultKind::<TestError>::map(res, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Err("error".to_string()));
    }

    #[test]
    fn result_kind_functor_composition_ok_str_err_u32() {
        // Renamed test
        let res: Result<&str, u32> = Ok("hello");
        let f = clone_fn_map(|x: &str| x.to_uppercase());
        let g = clone_fn_map(|y: String| y.len());

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map =
            ResultKind::<u32>::map(res, move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker

        let res_for_seq: Result<&str, u32> = Ok("hello");
        let sequential_map = ResultKind::<u32>::map(ResultKind::<u32>::map(res_for_seq, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Ok(5));
    }

    #[test]
    fn result_kind_functor_composition_err_str_err_u32() {
        // Renamed test
        let res: Result<&str, u32> = Err(404);
        let f = clone_fn_map(|x: &str| x.to_uppercase());
        let g = clone_fn_map(|y: String| y.len());

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map =
            ResultKind::<u32>::map(res, move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker

        let res_for_seq: Result<&str, u32> = Err(404);
        let sequential_map = ResultKind::<u32>::map(ResultKind::<u32>::map(res_for_seq, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Err(404));
    }
}

pub mod vec_kind_functor_laws {
    // Renamed module
    use super::*;

    // Identity law: VecKind::map(vec, |x| x) == vec
    #[test]
    fn vec_kind_functor_identity_non_empty() {
        // Renamed test
        let vec_val = vec![10, 20, 30];
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(VecKind::map(vec_val.clone(), identity_fn), vec_val); // Renamed Marker
    }

    #[test]
    fn vec_kind_functor_identity_empty() {
        // Renamed test
        let vec_val: Vec<i32> = vec![];
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(VecKind::map(vec_val.clone(), identity_fn), vec_val); // Renamed Marker
    }

    // Composition law: VecKind::map(vec, |x| g(f(x))) == VecKind::map(VecKind::map(vec, f), g)
    #[test]
    fn vec_kind_functor_composition_non_empty() {
        // Renamed test
        let vec_val = vec![10, 20, 30];
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = VecKind::map(vec_val.clone(), move |x| {
            g_clone.clone()(f_clone.clone()(x))
        }); // Renamed Marker
        let sequential_map = VecKind::map(VecKind::map(vec_val, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, vec![25, 45, 65]);
    }

    #[test]
    fn vec_kind_functor_composition_empty() {
        // Renamed test
        let vec_val: Vec<i32> = vec![];
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = VecKind::map(vec_val.clone(), move |x| {
            g_clone.clone()(f_clone.clone()(x))
        }); // Renamed Marker
        let sequential_map = VecKind::map(VecKind::map(vec_val, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Vec::<i32>::new());
    }

    #[test]
    fn vec_kind_functor_composition_str() {
        // Renamed test
        let vec_val = vec!["hello", "world"];
        let f = clone_fn_map(|x: &str| x.to_uppercase());
        let g = clone_fn_map(|y: String| y.len());

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = VecKind::map(vec_val.clone(), move |x| {
            g_clone.clone()(f_clone.clone()(x))
        }); // Renamed Marker

        let sequential_map = VecKind::map(VecKind::map(vec_val, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, vec![5, 5]);
    }
}

pub mod rcfn_kind_functor_laws {
    use super::*;
    type Env = i32;

    // Identity law: RcFnKind::map(rcfn, |x| x) == rcfn
    #[test]
    fn rcfn_kind_functor_identity() {
        let env_val: Env = 5;
        let rcfn_creator = || RcFn::new(move |env: Env| env * 2);

        let identity_fn = clone_fn_map(|x: i32| x);
        let mapped_rcfn: RcFn<Env, i32> = RcFnKind::<Env>::map(rcfn_creator(), identity_fn);

        assert_eq!(mapped_rcfn.call(env_val), rcfn_creator().call(env_val));
        assert_eq!(mapped_rcfn.call(env_val), 10);
    }

    // Composition law: RcFnKind::map(rcfn, |x| g(f(x))) == RcFnKind::map(RcFnKind::map(rcfn, f), g)
    #[test]
    fn rcfn_kind_functor_composition() {
        let env_val: Env = 3;
        let rcfn_creator = || RcFn::new(move |env: Env| env + 1);

        let f = clone_fn_map(|x: i32| (x * x) as f64);
        let g = clone_fn_map(|y: f64| y.to_string());

        let f_clone_for_composed = f.clone();
        let g_clone_for_composed = g.clone();
        let composed_map_rcfn: RcFn<Env, String> = RcFnKind::<Env>::map(rcfn_creator(), move |x| {
            g_clone_for_composed.clone()(f_clone_for_composed.clone()(x))
        });

        let mapped_f_rcfn: RcFn<Env, f64> = RcFnKind::<Env>::map(rcfn_creator(), f);
        let sequential_map_rcfn: RcFn<Env, String> = RcFnKind::<Env>::map(mapped_f_rcfn, g);

        assert_eq!(
            composed_map_rcfn.call(env_val),
            sequential_map_rcfn.call(env_val)
        );
        assert_eq!(composed_map_rcfn.call(env_val), "16".to_string());
    }
}

pub mod cfn_once_kind_functor_laws {
    // Renamed module
    use super::*;
    type Env = i32; // Common environment type for these tests

    // Identity law: CFnOnceKind::map(cfn_once, |x| x) == cfn_once
    #[test]
    fn cfn_once_kind_functor_identity() {
        // Renamed test
        let env_val: Env = 5;
        let cfn_once_val_creator = || CFnOnce::new(move |env: Env| env * 2);

        let identity_fn = clone_fn_map(|x: i32| x);
        let mapped_cfn_once: CFnOnce<Env, i32> =
            CFnOnceKind::<Env>::map(cfn_once_val_creator(), identity_fn.clone()); // Renamed Marker

        assert_eq!(
            mapped_cfn_once.call_once(env_val),
            cfn_once_val_creator().call_once(env_val)
        );

        assert_eq!(
            CFnOnceKind::<Env>::map(cfn_once_val_creator(), identity_fn).call_once(env_val),
            10
        ); // Renamed Marker
    }

    // Composition law: CFnOnceKind::map(cfn_once, |x| g(f(x))) == CFnOnceKind::map(CFnOnceKind::map(cfn_once, f), g)
    #[test]
    fn cfn_once_kind_functor_composition() {
        // Renamed test
        let env_val: Env = 3;
        let cfn_once_val_creator = || CFnOnce::new(move |env: Env| env + 1);

        let f = clone_fn_map(|x: i32| (x * x) as f64);
        let g = clone_fn_map(|y: f64| y.to_string());

        let f_clone_for_composed = f.clone();
        let g_clone_for_composed = g.clone();
        let composed_closure = clone_fn_map(move |x_val: i32| {
            g_clone_for_composed.clone()(f_clone_for_composed.clone()(x_val))
        });
        let composed_map_cfn_once: CFnOnce<Env, String> =
            CFnOnceKind::<Env>::map(cfn_once_val_creator(), composed_closure); // Renamed Marker

        let mapped_f_cfn_once: CFnOnce<Env, f64> =
            CFnOnceKind::<Env>::map(cfn_once_val_creator(), f.clone()); // Renamed Marker
        let sequential_map_cfn_once: CFnOnce<Env, String> =
            CFnOnceKind::<Env>::map(mapped_f_cfn_once, g.clone()); // Renamed Marker

        assert_eq!(
            composed_map_cfn_once.call_once(env_val),
            sequential_map_cfn_once.call_once(env_val)
        );

        let f_check = clone_fn_map(|x: i32| (x * x) as f64);
        let g_check = clone_fn_map(|y: f64| y.to_string());
        let composed_closure_check =
            clone_fn_map(move |x_val: i32| g_check.clone()(f_check.clone()(x_val)));
        assert_eq!(
            CFnOnceKind::<Env>::map(cfn_once_val_creator(), composed_closure_check)
                .call_once(env_val),
            "16".to_string()
        ); // Renamed Marker
    }
}

pub mod identity_kind_functor_laws {
    // Renamed module
    use super::*;

    // Identity law: IdentityKind::map(id_val, |x| x) == id_val
    #[test]
    fn identity_kind_functor_identity() {
        // Renamed test
        let id_val = Identity(10);
        let identity_fn = clone_fn_map(|x: i32| x);
        assert_eq!(IdentityKind::map(id_val.clone(), identity_fn), id_val); // Renamed Marker
    }

    // Composition law: IdentityKind::map(id_val, |x| g(f(x))) == IdentityKind::map(IdentityKind::map(id_val, f), g)
    #[test]
    fn identity_kind_functor_composition() {
        // Renamed test
        let id_val = Identity(10);
        let f = clone_fn_map(|x: i32| x * 2);
        let g = clone_fn_map(|y: i32| y + 5);

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map =
            IdentityKind::map(id_val.clone(), move |x| g_clone.clone()(f_clone.clone()(x))); // Renamed Marker
        let sequential_map = IdentityKind::map(IdentityKind::map(id_val, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Identity(25));
    }

    #[test]
    fn identity_kind_functor_composition_str() {
        // Renamed test
        let id_val = Identity("hello");
        let f = clone_fn_map(|x: &str| x.to_uppercase());
        let g = clone_fn_map(|y: String| y.len());

        let f_clone = f.clone();
        let g_clone = g.clone();
        let composed_map = IdentityKind::map(id_val.clone(), move |x: &str| {
            g_clone.clone()(f_clone.clone()(x))
        }); // Renamed Marker

        let sequential_map = IdentityKind::map(IdentityKind::map(id_val, f), g); // Renamed Marker

        assert_eq!(composed_map, sequential_map);
        assert_eq!(composed_map, Identity(5));
    }
}

pub mod reader_t_kind_functor_laws {
    // Renamed module
    use super::*;
    type EnvReader = String;
    type InnerMonadKind = OptionKind; // ReaderT<R, OptionKind, A> // Renamed Marker

    // Identity law: ReaderTKind::map(reader_t, |x| x) == reader_t
    #[test]
    fn reader_t_kind_functor_identity() {
        // Renamed test
        let env_val = "test_env".to_string();
        let reader_t_creator = || ReaderT::new(move |_env: EnvReader| Some(10));

        let identity_fn = clone_fn_map(|x: i32| x);
        let mapped_reader_t: ReaderT<EnvReader, InnerMonadKind, i32> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), identity_fn); // Renamed Marker

        assert_eq!(
            (mapped_reader_t.run_reader_t)(env_val.clone()),
            (reader_t_creator().run_reader_t)(env_val.clone())
        );
        assert_eq!((mapped_reader_t.run_reader_t)(env_val), Some(10));
    }

    #[test]
    fn reader_t_kind_functor_identity_inner_none() {
        // Renamed test
        let env_val = "test_env".to_string();
        let reader_t_creator = || ReaderT::new(move |_env: EnvReader| None::<i32>);

        let identity_fn = clone_fn_map(|x: i32| x);
        let mapped_reader_t: ReaderT<EnvReader, InnerMonadKind, i32> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), identity_fn); // Renamed Marker

        assert_eq!(
            (mapped_reader_t.run_reader_t)(env_val.clone()),
            (reader_t_creator().run_reader_t)(env_val.clone())
        );
        assert_eq!((mapped_reader_t.run_reader_t)(env_val), None);
    }

    // Composition law
    #[test]
    fn reader_t_kind_functor_composition() {
        // Renamed test
        let env_val = "test_env".to_string();
        let reader_t_creator = || ReaderT::new(move |_env: EnvReader| Some(10));

        let f = clone_fn_map(|x: i32| x as f64 * 2.0);
        let g = clone_fn_map(|y: f64| format!("Value: {:.1}", y));

        let f_clone_composed = f.clone();
        let g_clone_composed = g.clone();
        let composed_map_reader_t: ReaderT<EnvReader, InnerMonadKind, String> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), move |x| {
                g_clone_composed.clone()(f_clone_composed.clone()(x))
            }); // Renamed Marker

        let mapped_f_reader_t: ReaderT<EnvReader, InnerMonadKind, f64> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), f); // Renamed Marker
        let sequential_map_reader_t: ReaderT<EnvReader, InnerMonadKind, String> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(mapped_f_reader_t, g); // Renamed Marker

        assert_eq!(
            (composed_map_reader_t.run_reader_t)(env_val.clone()),
            (sequential_map_reader_t.run_reader_t)(env_val.clone())
        );
        assert_eq!(
            (composed_map_reader_t.run_reader_t)(env_val),
            Some("Value: 20.0".to_string())
        );
    }

    #[test]
    fn reader_t_kind_functor_composition_inner_none() {
        // Renamed test
        let env_val = "test_env".to_string();
        let reader_t_creator = || ReaderT::new(move |_env: EnvReader| None::<i32>);

        let f = clone_fn_map(|x: i32| x as f64 * 2.0);
        let g = clone_fn_map(|y: f64| format!("Value: {:.1}", y));

        let f_clone_composed = f.clone();
        let g_clone_composed = g.clone();
        let composed_map_reader_t: ReaderT<EnvReader, InnerMonadKind, String> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), move |x| {
                g_clone_composed.clone()(f_clone_composed.clone()(x))
            }); // Renamed Marker

        let mapped_f_reader_t: ReaderT<EnvReader, InnerMonadKind, f64> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(reader_t_creator(), f); // Renamed Marker
        let sequential_map_reader_t: ReaderT<EnvReader, InnerMonadKind, String> =
            ReaderTKind::<EnvReader, InnerMonadKind>::map(mapped_f_reader_t, g); // Renamed Marker

        assert_eq!(
            (composed_map_reader_t.run_reader_t)(env_val.clone()),
            (sequential_map_reader_t.run_reader_t)(env_val.clone())
        );
        assert_eq!((composed_map_reader_t.run_reader_t)(env_val), None);
    }
}
