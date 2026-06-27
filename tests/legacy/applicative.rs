// Original content from src/applicative.rs test modules
// with use statements adjusted for the new location.

// For map law test if Functor::map is used directly
// lift_a1 is defined in src/applicative.rs (and thus in the monadify::applicative module)
// `Applicative` and `lift_a1` are brought in via `super::*` in nested test modules.

// Re-checking lift_a1 location: it's defined in src/applicative.rs, so it will be monadify::applicative::lift_a1
// However, the test `test_lift_a1` uses `lift_a1` directly after `use super::*;`.
// So, in the new test file, it should be `use monadify::applicative::lift_a1;` at the top,
// or `use monadify::applicative::*;` if many items are needed.
// Let's be specific.

// Corrected imports for the test file structure:
// Items from monadify (re-exported from lib.rs)
// Items from monadify::module_name (specific modules)

// These imports will need to point to legacy versions.
// For now, using placeholder paths that will be fixed in Phase 5.
// use monadify::legacy_applicative::lift_a1;
// use monadify::legacy_applicative::Applicative;
// use monadify::legacy_apply::Apply;
// use monadify::legacy_functor::Functor;
// use monadify::legacy_function::CFn;

// The actual use statements will depend on how `src/legacy/mod.rs` re-exports items,
// or if we use fully qualified paths like `crate::legacy::applicative::Applicative`.
// For now, I will keep the structure and assume paths will be fixed later.

#[cfg(test)]
mod classic_applicative_tests {
    // Assuming lift_a1 will be available via crate::legacy::applicative::lift_a1
    // and Applicative via crate::legacy::applicative::Applicative
    // For now, let's use fully qualified paths for clarity during refactor,
    // or rely on a top-level `use crate::legacy::*;` in the integration file.

    // This module will test items from `crate::legacy::applicative`
    // The `lift_a1` function is part of the `classic` module in the original `src/applicative.rs`.
    // So it should be in `crate::legacy::applicative::lift_a1`.
    // `Option::pure` comes from `crate::legacy::applicative::Applicative` for `Option`.

    #[test]
    fn check_some() {
        // This will use the Applicative impl from monadify::legacy::applicative
        assert_eq!(
            <Option<i32> as monadify::legacy::applicative::Applicative<i32>>::pure(1),
            Some(1)
        );
    }
}

#[cfg(test)]
mod applicative_laws {
    use monadify::function::CFn; // Corrected path
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::apply::Apply;
    #[allow(unused_imports)]
    use monadify::legacy::functor::Functor;

    fn identity<T>(x: T) -> T {
        x
    }

    #[test]
    fn option_applicative_identity_some() {
        let v = Some(10);
        assert_eq!(
            Apply::apply(
                v,
                <Option<_> as Applicative<CFn<i32, i32>>>::pure(CFn::new(identity::<i32>))
            ),
            v
        );
    }

    #[test]
    fn option_applicative_identity_none() {
        let v: Option<i32> = None;
        assert_eq!(
            Apply::apply(
                v,
                <Option<_> as Applicative<CFn<i32, i32>>>::pure(CFn::new(identity::<i32>))
            ),
            v
        );
    }

    #[test]
    fn option_applicative_homomorphism() {
        let x = 10;
        let f = |y: i32| y * 2;
        let pure_x = <Option<_> as Applicative<i32>>::pure(x);

        assert_eq!(
            Apply::apply(
                pure_x,
                <Option<_> as Applicative<CFn<i32, i32>>>::pure(CFn::new(f))
            ),
            <Option<_> as Applicative<i32>>::pure(f(x))
        );
        assert_eq!(
            Apply::apply(
                pure_x,
                <Option<_> as Applicative<CFn<i32, i32>>>::pure(CFn::new(f))
            ),
            Some(20)
        );
    }

    #[test]
    fn option_applicative_interchange_some_fn() {
        let y = 10;
        let f = |x: i32| x + 5;

        let lhs = Apply::apply(<Option<_> as Applicative<i32>>::pure(y), Some(CFn::new(f)));

        let u_for_rhs = Some(CFn::new(f));
        let eval_at_y = move |f_func: CFn<i32, i32>| (f_func).call(y); // .call() not *
        let rhs_interchange = Functor::map(u_for_rhs, eval_at_y);

        assert_eq!(lhs, rhs_interchange);
        assert_eq!(lhs, Some(15));
    }

    #[test]
    fn option_applicative_interchange_none_fn() {
        let y = 10;
        let u: Option<CFn<i32, i32>> = None;

        let lhs = Apply::apply(
            <Option<_> as Applicative<i32>>::pure(y),
            None::<CFn<i32, i32>>,
        );

        let eval_at_y = move |f_func: CFn<i32, i32>| (f_func).call(y); // .call() not *
        let rhs_interchange = Functor::map(u, eval_at_y);

        assert_eq!(lhs, rhs_interchange);
        assert_eq!(lhs, None);
    }

    #[test]
    fn option_applicative_map_some() {
        let v = Some(10);
        let f = |x: i32| x.to_string();
        let pure_f = <Option<_> as Applicative<CFn<i32, String>>>::pure(CFn::new(f));

        let lhs = Functor::map(v, f); // clone v for map
        let rhs = Apply::apply(v, pure_f);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Some("10".to_string()));
    }

    #[test]
    fn option_applicative_map_none() {
        let v: Option<i32> = None;
        let f = |x: i32| x.to_string();
        let pure_f = <Option<_> as Applicative<CFn<i32, String>>>::pure(CFn::new(f));

        let lhs = Functor::map(v, f); // clone v for map
        let rhs = Apply::apply(v, pure_f);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }
}

#[cfg(test)]
mod result_applicative_laws {
    use monadify::function::CFn;
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::apply::Apply;
    use monadify::legacy::functor::Functor; // Corrected path

    fn identity<T>(x: T) -> T {
        x
    }

    #[test]
    fn result_applicative_identity_ok() {
        let v: Result<i32, String> = Ok(10);
        assert_eq!(
            Apply::apply(
                v.clone(),
                <Result<_, String> as Applicative<CFn<i32, i32>>>::pure(CFn::new(identity::<i32>))
            ),
            v
        );
    }

    #[test]
    fn result_applicative_identity_err() {
        let v: Result<i32, String> = Err("error".to_string());
        assert_eq!(
            Apply::apply(
                v.clone(),
                <Result<_, String> as Applicative<CFn<i32, i32>>>::pure(CFn::new(identity::<i32>))
            ),
            v
        );
    }

    #[test]
    fn result_applicative_identity_ok_apply_err() {
        let v: Result<i32, String> = Ok(10);
        let f_err: Result<CFn<i32, i32>, String> = Err("function error".to_string());
        assert_eq!(Apply::apply(v, f_err), Err("function error".to_string()));
    }

    #[test]
    fn result_applicative_homomorphism_ok() {
        let x = 10;
        let f = |y: i32| y * 2;
        let pure_x: Result<i32, String> = <Result<_, String> as Applicative<i32>>::pure(x); // Corrected

        assert_eq!(
            Apply::apply(
                pure_x.clone(),
                <Result<_, String> as Applicative<CFn<i32, i32>>>::pure(CFn::new(f))
            ),
            <Result<_, String> as Applicative<i32>>::pure(f(x))
        );
        assert_eq!(
            Apply::apply(
                <Result<_, String> as Applicative<i32>>::pure(x),
                <Result<_, String> as Applicative<CFn<i32, i32>>>::pure(CFn::new(f))
            ), // Corrected
            Ok::<i32, String>(20)
        );
    }

    #[test]
    fn result_applicative_homomorphism_err_val() {
        let x = 10;
        let _f = |y: i32| y * 2;
        let pure_f_err: Result<CFn<i32, i32>, String> = Err("function error".to_string());
        let pure_x: Result<i32, String> = <Result<_, String> as Applicative<i32>>::pure(x); // Corrected
        assert_eq!(
            Apply::apply(pure_x, pure_f_err),
            Err("function error".to_string())
        );
    }

    #[test]
    fn result_applicative_interchange_ok_fn() {
        let y = 10;
        let f = move |x: i32| x + y; // y is captured

        let lhs = Apply::apply(
            <Result<_, String> as Applicative<i32>>::pure(y),
            Ok(CFn::new(f)),
        );

        let u_for_rhs: Result<CFn<i32, i32>, String> = Ok(CFn::new(f));
        // The closure for map needs to capture y.
        let y_clone_for_map = y;
        let rhs = Functor::map(u_for_rhs, move |f_func: CFn<i32, i32>| {
            (f_func).call(y_clone_for_map)
        });

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok(20));
    }

    #[test]
    fn result_applicative_interchange_err_fn() {
        let y = 10;
        let u: Result<CFn<i32, i32>, String> = Err("function error".to_string());

        let lhs = Apply::apply(
            <Result<_, String> as Applicative<i32>>::pure(y),
            Err("function error".to_string()),
        );
        let y_clone_for_map = y;
        let rhs = Functor::map(u, move |f_func: CFn<i32, i32>| {
            (f_func).call(y_clone_for_map)
        });

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("function error".to_string()));
    }

    #[test]
    fn result_applicative_map_ok() {
        let v: Result<i32, String> = Ok(10);
        let f = |x: i32| x.to_string();

        let lhs = Functor::map(v.clone(), f);
        let rhs = Apply::apply(
            v,
            <Result<_, String> as Applicative<CFn<i32, String>>>::pure(CFn::new(f)),
        );

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok("10".to_string()));
    }

    #[test]
    fn result_applicative_map_err() {
        let v: Result<i32, String> = Err("error".to_string());
        let f = |x: i32| x.to_string();

        let lhs = Functor::map(v.clone(), f);
        let rhs = Apply::apply(
            v,
            <Result<_, String> as Applicative<CFn<i32, String>>>::pure(CFn::new(f)),
        );

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("error".to_string()));
    }
}

#[cfg(test)]
mod vec_applicative_laws {
    use monadify::function::CFn; // Corrected path
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::apply::Apply;
    use monadify::legacy::functor::Functor;

    fn identity<T: Clone>(x: T) -> T {
        x.clone()
    }

    #[test]
    fn vec_applicative_identity_non_empty() {
        let v = vec![10, 20];
        let pure_identity_fn_vec = vec![CFn::new(identity::<i32>)];
        assert_eq!(Apply::apply(v.clone(), pure_identity_fn_vec), v);
    }

    #[test]
    fn vec_applicative_identity_empty() {
        let v: Vec<i32> = vec![];
        let pure_identity_fn_vec = vec![CFn::new(identity::<i32>)];
        assert_eq!(Apply::apply(v.clone(), pure_identity_fn_vec), v);
    }

    #[test]
    fn vec_applicative_homomorphism() {
        let x = 10;
        let f = |y: i32| y * 2;
        let pure_x_vec = <Vec<_> as Applicative<i32>>::pure(x);

        let pure_f_vec = vec![CFn::new(f)];
        let lhs = Apply::apply(pure_x_vec, pure_f_vec);
        let rhs = <Vec<_> as Applicative<i32>>::pure(f(x));
        assert_eq!(lhs, rhs);
        assert_eq!(lhs, vec![20]);
    }

    #[test]
    fn vec_applicative_interchange() {
        let y = 10;
        let add_5 = |x: i32| x + 5;
        let mul_2 = |x: i32| x * 2;

        let u_lhs: Vec<CFn<i32, i32>> = vec![CFn::new(add_5), CFn::new(mul_2)];
        let lhs = Apply::apply(<Vec<_> as Applicative<i32>>::pure(y), u_lhs);

        let y_cloned = y;
        let u_rhs: Vec<CFn<i32, i32>> = vec![CFn::new(add_5), CFn::new(mul_2)];
        let rhs = Functor::map(u_rhs, move |f_val: CFn<i32, i32>| (f_val).call(y_cloned));

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, vec![15, 20]);
    }

    #[test]
    fn vec_applicative_interchange_empty_u() {
        let y = 10;
        let u_lhs: Vec<CFn<i32, i32>> = vec![];
        let u_rhs: Vec<CFn<i32, i32>> = vec![];

        let lhs = Apply::apply(<Vec<_> as Applicative<i32>>::pure(y), u_lhs);

        let y_cloned = y;
        let rhs = Functor::map(u_rhs, move |f_val: CFn<i32, i32>| (f_val).call(y_cloned));

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<i32>::new());
    }

    #[test]
    fn vec_applicative_map_non_empty() {
        let v = vec![10, 20];
        let f = |x: i32| x.to_string();

        let lhs = Functor::map(v.clone(), f);

        let pure_f_vec = vec![CFn::new(f)];
        let rhs = Apply::apply(v, pure_f_vec);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, vec!["10".to_string(), "20".to_string()]);
    }

    #[test]
    fn vec_applicative_map_empty() {
        let v: Vec<i32> = vec![];
        let f = |x: i32| x.to_string();

        let lhs = Functor::map(v.clone(), f);
        let pure_f_vec = vec![CFn::new(f)];
        let rhs = Apply::apply(v, pure_f_vec);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }
}
