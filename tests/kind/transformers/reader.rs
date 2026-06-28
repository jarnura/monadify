use monadify::applicative::kind::Applicative; // Changed hkt to kind
use monadify::functor::kind::Functor; // Changed hkt to kind
use monadify::identity::kind::{Identity, IdentityKind}; // Changed IdentityHKTMarker to IdentityKind
use monadify::monad::kind::{Bind, Monad}; // Changed hkt to kind
use monadify::OptionKind;
#[cfg(not(feature = "legacy"))]
// Imports adjusted for the new location
use monadify::{MonadReader, ReaderT, ReaderTKind}; // Changed ReaderTHKTMarker to ReaderTKind // Changed OptionHKTMarker to OptionKind

#[derive(Clone, Debug, PartialEq)]
struct EnvConfig {
    pub val: i32,
}

type TestReader<A> = ReaderT<EnvConfig, IdentityKind, A>; // Changed IdentityHKTMarker to IdentityKind
type TestReaderKind = ReaderTKind<EnvConfig, IdentityKind>; // Renamed TestReaderHKT, changed IdentityHKTMarker

#[test]
fn test_reader_t_kind_functor_map() {
    // Renamed test
    let reader: TestReader<i32> = ReaderT::new(|cfg: EnvConfig| Identity(cfg.val + 1));
    let mapped_reader: TestReader<i32> = TestReaderKind::map(reader, |x| x * 2); // Renamed Marker
    let result = (mapped_reader.run_reader_t)(EnvConfig { val: 10 });
    assert_eq!(result, Identity(22));
}

#[test]
fn test_reader_t_kind_applicative_pure() {
    // Renamed test
    let pure_reader: TestReader<i32> = TestReaderKind::pure(100); // Renamed Marker
    let result = (pure_reader.run_reader_t)(EnvConfig { val: 0 });
    assert_eq!(result, Identity(100));
}

#[test]
fn test_reader_t_kind_monad_bind() {
    // Renamed test
    let reader1: TestReader<i32> = ReaderT::new(|cfg: EnvConfig| Identity(cfg.val));
    let f = |x: i32| -> TestReader<i32> {
        ReaderT::new(move |cfg: EnvConfig| Identity(x + cfg.val + 5))
    };
    let bound_reader: TestReader<i32> = TestReaderKind::bind(reader1, f); // Renamed Marker
    let result = (bound_reader.run_reader_t)(EnvConfig { val: 3 });
    assert_eq!(result, Identity(11));
}

#[test]
fn test_monad_reader_kind_ask() {
    // Renamed test
    let ask_reader: ReaderT<EnvConfig, IdentityKind, EnvConfig> = // Changed IdentityHKTMarker
        TestReaderKind::ask(); // Renamed Marker
    let result = (ask_reader.run_reader_t)(EnvConfig { val: 7 });
    assert_eq!(result, Identity(EnvConfig { val: 7 }));
}

#[test]
fn test_monad_reader_kind_local() {
    // Renamed test
    let reader: TestReader<i32> = ReaderT::new(|cfg: EnvConfig| Identity(cfg.val * 10));
    let modified_reader: TestReader<i32> = TestReaderKind::local(
        |mut cfg: EnvConfig| {
            cfg.val += 1;
            cfg
        },
        reader,
    ); // Renamed Marker
    let result = (modified_reader.run_reader_t)(EnvConfig { val: 2 });
    assert_eq!(result, Identity(30));
}

// Helper to run reader and extract value for simple Identity case
fn run_test_reader<A: PartialEq + std::fmt::Debug>(reader: TestReader<A>, env: EnvConfig) -> A {
    (reader.run_reader_t)(env).0
}

// Helper to compare two TestReaders by running them with the same environment
fn assert_readers_eq<A: PartialEq + std::fmt::Debug + Clone>(
    reader1: TestReader<A>,
    reader2: TestReader<A>,
    env: EnvConfig,
) {
    let val1 = (reader1.run_reader_t)(env.clone()).0;
    let val2 = (reader2.run_reader_t)(env).0;
    assert_eq!(val1, val2);
}

#[test]
fn test_reader_t_kind_monad_join() {
    // Renamed test
    let env = EnvConfig { val: 10 };
    let mma_reader: TestReader<TestReader<i32>> = TestReaderKind::pure(TestReaderKind::pure(100)); // Renamed Marker

    let joined_reader: TestReader<i32> = TestReaderKind::join(mma_reader); // Renamed Marker
    assert_eq!(run_test_reader(joined_reader, env.clone()), 100);

    let mma_reader_env_dependent: TestReader<TestReader<i32>> =
        ReaderT::new(move |cfg1: EnvConfig| {
            Identity(ReaderT::new(move |cfg2: EnvConfig| {
                Identity(cfg1.val + cfg2.val)
            }))
        });

    let joined_reader_env: TestReader<i32> = TestReaderKind::join(mma_reader_env_dependent); // Renamed Marker
    assert_eq!(
        run_test_reader(joined_reader_env, env.clone()),
        env.val + env.val
    );
}

// Monad Laws for ReaderTKind<EnvConfig, IdentityKind>
#[test]
fn test_reader_t_kind_monad_law_left_identity() {
    // Renamed test
    let env = EnvConfig { val: 5 };
    let a = 10;
    let f = |x: i32| -> TestReader<String> {
        ReaderT::new(move |cfg: EnvConfig| Identity((x + cfg.val).to_string()))
    };

    let lhs: TestReader<String> = TestReaderKind::bind(TestReaderKind::pure(a), f.clone()); // Renamed Marker
    let rhs: TestReader<String> = f(a);

    assert_readers_eq(lhs.clone(), rhs.clone(), env.clone());
    let expected_val_rhs = ((f(a)).run_reader_t)(env).0;
    assert_eq!(expected_val_rhs, "15");
}

#[test]
fn test_reader_t_kind_monad_law_right_identity() {
    // Renamed test
    let env = EnvConfig { val: 7 };
    let m_orig: TestReader<i32> = ReaderT::new(move |cfg: EnvConfig| Identity(cfg.val * 2));

    let pure_fn = |x: i32| TestReaderKind::pure(x); // Renamed Marker

    let lhs: TestReader<i32> = TestReaderKind::bind(m_orig.clone(), pure_fn); // Renamed Marker
    let rhs: TestReader<i32> = m_orig.clone();

    assert_readers_eq(lhs.clone(), rhs.clone(), env.clone());
    let val_m_orig = (m_orig.run_reader_t)(env).0;
    assert_eq!(val_m_orig, 14);
}

#[test]
fn test_reader_t_kind_monad_law_associativity() {
    // Renamed test
    let env = EnvConfig { val: 3 };
    let m_orig: TestReader<i32> = ReaderT::new(move |cfg: EnvConfig| Identity(cfg.val + 1));

    let f =
        |x: i32| -> TestReader<i32> { ReaderT::new(move |cfg: EnvConfig| Identity(x * cfg.val)) };
    let g = |y: i32| -> TestReader<String> {
        ReaderT::new(move |cfg: EnvConfig| Identity((y + cfg.val).to_string()))
    };

    let lhs: TestReader<String> =
        TestReaderKind::bind(TestReaderKind::bind(m_orig.clone(), f.clone()), g.clone()); // Renamed Marker

    let f_clone_for_rhs = f.clone();
    let g_clone_for_rhs = g.clone();
    let composed_func = move |x_val: i32| -> TestReader<String> {
        TestReaderKind::bind(f_clone_for_rhs(x_val), g_clone_for_rhs.clone()) // Renamed Marker
    };
    let rhs: TestReader<String> = TestReaderKind::bind(m_orig.clone(), composed_func); // Renamed Marker

    assert_readers_eq(lhs.clone(), rhs.clone(), env.clone());
    let val_lhs = (lhs.run_reader_t)(env).0;
    assert_eq!(val_lhs, "15");
}

// --- Parity tests: inherent ergonomic forms vs trait UFCS forms ---

#[test]
fn test_reader_t_inherent_ask_parity_identity() {
    let env = EnvConfig { val: 42 };

    // Inherent ergonomic form
    let ask_inherent: TestReader<EnvConfig> = TestReaderKind::ask();
    // Trait UFCS form
    let ask_trait: TestReader<EnvConfig> =
        <TestReaderKind as MonadReader<EnvConfig, EnvConfig, IdentityKind>>::ask();

    let result_inherent = (ask_inherent.run_reader_t)(env.clone());
    let result_trait = (ask_trait.run_reader_t)(env.clone());
    assert_eq!(result_inherent, result_trait);
    assert_eq!(result_inherent, Identity(EnvConfig { val: 42 }));
}

#[test]
fn test_reader_t_inherent_ask_parity_option() {
    use monadify::ReaderT as RT;
    type OptReaderKind = monadify::ReaderTKind<i32, monadify::OptionKind>;

    let env: i32 = 99;

    // Inherent ergonomic form
    let ask_inherent: RT<i32, monadify::OptionKind, i32> = OptReaderKind::ask();
    // Trait UFCS form
    let ask_trait: RT<i32, monadify::OptionKind, i32> =
        <OptReaderKind as MonadReader<i32, i32, monadify::OptionKind>>::ask();

    assert_eq!(
        (ask_inherent.run_reader_t)(env),
        (ask_trait.run_reader_t)(env),
    );
    assert_eq!((ask_inherent.run_reader_t)(env), Some(99));
}

#[test]
fn test_reader_t_inherent_local_parity_identity() {
    let env = EnvConfig { val: 5 };
    let base: TestReader<i32> = ReaderT::new(|cfg: EnvConfig| Identity(cfg.val * 2));
    // A non-capturing closure is Copy so we can pass it to both forms.
    let modifier = |mut cfg: EnvConfig| {
        cfg.val += 10;
        cfg
    };

    // Method (inherent) form: comp.local(f)
    let local_method = base.clone().local(modifier);
    // Trait UFCS form: MonadReader::local(f, comp)
    let local_trait = TestReaderKind::local(modifier, base);

    let result_method = (local_method.run_reader_t)(env.clone());
    let result_trait = (local_trait.run_reader_t)(env.clone());
    assert_eq!(result_method, result_trait);
    // env.val=5, modifier adds 10 -> 15, *2 -> 30
    assert_eq!(result_method, Identity(30));
}

// Helper to run ReaderT with Option inner monad and compare
fn run_kind_reader_t_option<
    R: Clone + 'static,
    A: PartialEq + std::fmt::Debug + Clone + 'static,
>(
    // Renamed helper
    reader: ReaderT<R, OptionKind, A>, // Changed OptionHKTMarker to OptionKind
    env: R,
) -> Option<A> {
    (reader.run_reader_t)(env)
}

#[test]
fn test_kind_reader_t_monad_laws_option_inner() {
    // Renamed test
    type Env = i32;
    type A = i32;
    type B = String;
    type C = usize;

    let env_val: Env = 10;
    let a_val: A = 5;

    // f_opt: A -> ReaderT<Env, OptionKind, B>
    let f_opt = move |x: A| -> ReaderT<Env, OptionKind, B> {
        // Changed OptionHKTMarker
        ReaderT::new(move |env: Env| -> Option<B> {
            if x > 0 && env > 0 {
                Some(format!("f_opt: val={}, env={}", x, env))
            } else {
                None
            }
        })
    };

    // g_opt: B -> ReaderT<Env, OptionKind, C>
    let g_opt = move |s: B| -> ReaderT<Env, OptionKind, C> {
        // Changed OptionHKTMarker
        ReaderT::new(move |env: Env| -> Option<C> {
            if !s.is_empty() && env > 0 {
                Some(s.len() + env as usize)
            } else {
                None
            }
        })
    };

    // m_opt_some: ReaderT<Env, OptionKind, A> (yields Some)
    let m_opt_some: ReaderT<Env, OptionKind, A> = // Changed OptionHKTMarker
        ReaderT::new(move |env: Env| if env > 0 { Some(a_val + env) } else { None });

    // m_opt_none: ReaderT<Env, OptionKind, A> (yields None)
    let m_opt_none: ReaderT<Env, OptionKind, A> = // Changed OptionHKTMarker
        ReaderT::new(move |_env: Env| None::<A>);

    // --- Test with m_opt_some ---
    // Law 1: pure(a).bind(f) == f(a)
    let pure_a_opt: ReaderT<Env, OptionKind, A> = // Changed OptionHKTMarker
        ReaderTKind::<Env, OptionKind>::pure(a_val); // Renamed Marker
    let left_law1_opt = ReaderTKind::<Env, OptionKind>::bind(pure_a_opt.clone(), f_opt.clone()); // Renamed Marker
    let right_law1_opt = f_opt(a_val);
    assert_eq!(
        run_kind_reader_t_option(left_law1_opt.clone(), env_val), // Renamed helper
        run_kind_reader_t_option(right_law1_opt.clone(), env_val)  // Renamed helper
    );

    // Law 2: m.bind(pure) == m
    let left_law2_opt = ReaderTKind::<Env, OptionKind>::bind(m_opt_some.clone(), |x: A| {
        ReaderTKind::<Env, OptionKind>::pure(x)
    }); // Renamed Marker
    let right_law2_opt = m_opt_some.clone();
    assert_eq!(
        run_kind_reader_t_option(left_law2_opt, env_val), // Renamed helper
        run_kind_reader_t_option(right_law2_opt, env_val)  // Renamed helper
    );

    // Law 3: m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))
    let left_law3_opt = ReaderTKind::<Env, OptionKind>::bind(
        ReaderTKind::<Env, OptionKind>::bind(m_opt_some.clone(), f_opt.clone()),
        g_opt.clone(),
    ); // Renamed Marker
    let right_law3_opt = ReaderTKind::<Env, OptionKind>::bind(m_opt_some.clone(), move |x: A| {
        ReaderTKind::<Env, OptionKind>::bind(f_opt(x), g_opt.clone())
    }); // Renamed Marker
    assert_eq!(
        run_kind_reader_t_option(left_law3_opt.clone(), env_val), // Renamed helper
        run_kind_reader_t_option(right_law3_opt.clone(), env_val)  // Renamed helper
    );

    // --- Test with m_opt_none (should propagate None) ---
    let val_for_f_none: A = -1;
    let pure_val_for_f_none: ReaderT<Env, OptionKind, A> = // Changed OptionHKTMarker
        ReaderTKind::<Env, OptionKind>::pure(val_for_f_none); // Renamed Marker
    let left_law1_f_none =
        ReaderTKind::<Env, OptionKind>::bind(pure_val_for_f_none.clone(), f_opt.clone()); // Renamed Marker
    let right_law1_f_none = f_opt(val_for_f_none);
    assert_eq!(
        run_kind_reader_t_option(left_law1_f_none.clone(), env_val), // Renamed helper
        run_kind_reader_t_option(right_law1_f_none.clone(), env_val)  // Renamed helper
    );
    assert_eq!(
        run_kind_reader_t_option(right_law1_f_none.clone(), env_val),
        None
    ); // Renamed helper

    let left_law2_none = ReaderTKind::<Env, OptionKind>::bind(m_opt_none.clone(), |x: A| {
        ReaderTKind::<Env, OptionKind>::pure(x)
    }); // Renamed Marker
    let right_law2_none = m_opt_none.clone();
    assert_eq!(
        run_kind_reader_t_option(left_law2_none.clone(), env_val), // Renamed helper
        run_kind_reader_t_option(right_law2_none.clone(), env_val)  // Renamed helper
    );
    assert_eq!(
        run_kind_reader_t_option(right_law2_none.clone(), env_val),
        None
    ); // Renamed helper

    let left_law3_none = ReaderTKind::<Env, OptionKind>::bind(
        ReaderTKind::<Env, OptionKind>::bind(m_opt_none.clone(), f_opt.clone()),
        g_opt.clone(),
    ); // Renamed Marker
    let right_law3_none = ReaderTKind::<Env, OptionKind>::bind(m_opt_none.clone(), move |x: A| {
        ReaderTKind::<Env, OptionKind>::bind(f_opt(x), g_opt.clone())
    }); // Renamed Marker
    assert_eq!(
        run_kind_reader_t_option(left_law3_none.clone(), env_val), // Renamed helper
        run_kind_reader_t_option(right_law3_none.clone(), env_val)  // Renamed helper
    );
    assert_eq!(
        run_kind_reader_t_option(right_law3_none.clone(), env_val),
        None
    ); // Renamed helper
}
