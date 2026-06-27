#![cfg(all(test, feature = "legacy"))] // Ensure these run only when 'legacy' is active

// These imports will need to point to legacy versions.
// use monadify::legacy_monad::{Bind, Monad, join, bind as legacy_bind_fn}; // Assuming join and bind (fn) are part of legacy::monad
// use monadify::legacy_applicative::Applicative;
// use monadify::legacy_function::CFn; // CFn is not part of legacy/hkt split

// For now, using fully qualified paths or direct use from crate::legacy modules.

#[cfg(test)]
mod classic_monad_tests {
    use monadify::legacy::monad::Bind; // Import Bind trait for method calls
                                       // Assuming join and bind (fn) are available via monadify::legacy::monad
                                       // And Applicative via monadify::legacy::applicative::Applicative

    #[test]
    fn bind_option() {
        let add_one = |x: i32| Some(x + 1);
        let add_two = |x: i32| Some(x + 2);
        let add_three = |x: i32| Some(x + 3);
        let result = <Option<i32> as monadify::legacy::monad::Bind<i32>>::bind(Some(1), add_one)
            .bind(add_two)
            .bind(add_three);
        assert_eq!(result, Some(7))
    }

    #[test]
    fn bind_option_with_composing() {
        let add_one = |x: i32| Some(x + 1);
        let add_two = |x: i32| Some(x + 2);
        let add_three = |x: i32| Some(x + 3);

        let composed_closure = move |x| add_one(x).and_then(add_two).and_then(add_three);
        let result =
            <Option<i32> as monadify::legacy::monad::Bind<i32>>::bind(Some(1), composed_closure);
        assert_eq!(result, Some(7));

        let result_join = monadify::legacy::monad::join(Some(Some(1)));
        assert_eq!(result_join, Some(1));

        let result_join_none = monadify::legacy::monad::join(Some(None::<i32>));
        assert_eq!(result_join_none, None);
    }

    #[test]
    fn bind_option_with_bind_composing() {
        let add_one = |x: i32| Some(x + 1);
        let add_two = |x: i32| Some(x + 2);
        let add_three = |x: i32| Some(x + 3);
        let result = <Option<i32> as monadify::legacy::monad::Bind<i32>>::bind(Some(1), add_one)
            .bind(add_two)
            .bind(add_three);
        assert_eq!(result, Some(7))
    }
}

#[cfg(test)]
mod monad_laws {
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::monad::Bind;

    #[test]
    fn option_monad_left_identity() {
        let a = 10;
        let f = |x: i32| -> Option<String> { Some((x * 2).to_string()) };

        let lhs = <Option<i32> as Bind<i32>>::bind(<Option<i32> as Applicative<i32>>::pure(a), f);
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Some("20".to_string()));
    }

    #[test]
    fn option_monad_left_identity_f_returns_none() {
        let a = 10;
        let f = |_x: i32| -> Option<String> { None };

        let lhs = <Option<i32> as Bind<i32>>::bind(<Option<i32> as Applicative<i32>>::pure(a), f);
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }

    #[test]
    fn option_monad_right_identity_some() {
        let m = Some(10);
        let pure_fn = |x: i32| <Option<i32> as Applicative<i32>>::pure(x);

        let lhs = <Option<i32> as Bind<i32>>::bind(m, pure_fn);
        let rhs = Some(10);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Some(10));
    }

    #[test]
    fn option_monad_right_identity_none() {
        let m: Option<i32> = None;
        let pure_fn = |x: i32| <Option<i32> as Applicative<i32>>::pure(x);

        let lhs = <Option<i32> as Bind<i32>>::bind(m, pure_fn);
        let rhs = None::<i32>;

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }

    #[test]
    fn option_monad_associativity_some() {
        let m = Some(10);
        let f = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g = |y: f64| -> Option<String> { Some(y.to_string()) };

        let lhs = <Option<f64> as Bind<f64>>::bind(<Option<i32> as Bind<i32>>::bind(m, f), g);

        let f_inner = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g_inner = |y: f64| -> Option<String> { Some(y.to_string()) };
        let inner_closure = move |x: i32| <Option<f64> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Option<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Some("20".to_string()));
    }

    #[test]
    fn option_monad_associativity_none_start() {
        let m: Option<i32> = None;
        let f = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g = |y: f64| -> Option<String> { Some(y.to_string()) };

        let lhs = <Option<f64> as Bind<f64>>::bind(<Option<i32> as Bind<i32>>::bind(m, f), g);

        let f_inner = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g_inner = |y: f64| -> Option<String> { Some(y.to_string()) };
        let inner_closure = move |x: i32| <Option<f64> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Option<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }

    #[test]
    fn option_monad_associativity_f_returns_none() {
        let m = Some(10);
        let f = |_x: i32| -> Option<f64> { None };
        let g = |y: f64| -> Option<String> { Some(y.to_string()) };

        let lhs = <Option<f64> as Bind<f64>>::bind(<Option<i32> as Bind<i32>>::bind(m, f), g);

        let f_inner = |_x: i32| -> Option<f64> { None };
        let g_inner = |y: f64| -> Option<String> { Some(y.to_string()) };
        let inner_closure = move |x: i32| <Option<f64> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Option<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }

    #[test]
    fn option_monad_associativity_g_returns_none() {
        let m = Some(10);
        let f = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g = |_y: f64| -> Option<String> { None };

        let lhs = <Option<f64> as Bind<f64>>::bind(<Option<i32> as Bind<i32>>::bind(m, f), g);

        let f_inner = |x: i32| -> Option<f64> { Some((x * 2) as f64) };
        let g_inner = |_y: f64| -> Option<String> { None };
        let inner_closure = move |x: i32| <Option<f64> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Option<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, None);
    }
}

#[cfg(test)]
mod result_monad_laws {
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::monad::Bind;

    #[test]
    fn result_monad_left_identity_ok() {
        let a = 10;
        let f = |x: i32| -> Result<String, String> { Ok((x * 2).to_string()) };

        let lhs = <Result<i32, String> as Bind<i32>>::bind(
            <Result<i32, String> as Applicative<i32>>::pure(a),
            f,
        );
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok("20".to_string()));
    }

    #[test]
    fn result_monad_left_identity_f_returns_err() {
        let a = 10;
        let f = |_x: i32| -> Result<String, String> { Err("f_error".to_string()) };

        let lhs = <Result<i32, String> as Bind<i32>>::bind(
            <Result<i32, String> as Applicative<i32>>::pure(a),
            f,
        );
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("f_error".to_string()));
    }

    #[test]
    fn result_monad_right_identity_ok() {
        let m: Result<i32, String> = Ok(10);
        let pure_fn = |x: i32| <Result<i32, String> as Applicative<i32>>::pure(x);

        let lhs = <Result<i32, String> as Bind<i32>>::bind(m.clone(), pure_fn);
        let rhs = m;

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok(10));
    }

    #[test]
    fn result_monad_right_identity_err() {
        let m: Result<i32, String> = Err("m_error".to_string());
        let pure_fn = |x: i32| <Result<i32, String> as Applicative<i32>>::pure(x);

        let lhs = <Result<i32, String> as Bind<i32>>::bind(m.clone(), pure_fn);
        let rhs = m;

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("m_error".to_string()));
    }

    #[test]
    fn result_monad_associativity_all_ok() {
        let m: Result<i32, String> = Ok(10);
        let f = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g = |y: f64| -> Result<String, String> { Ok(y.to_string()) };

        let lhs = <Result<f64, String> as Bind<f64>>::bind(
            <Result<i32, String> as Bind<i32>>::bind(m.clone(), f),
            g,
        );

        let f_inner = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g_inner = |y: f64| -> Result<String, String> { Ok(y.to_string()) };
        let inner_closure =
            move |x: i32| <Result<f64, String> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Result<i32, String> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Ok("20".to_string()));
    }

    #[test]
    fn result_monad_associativity_m_is_err() {
        let m: Result<i32, String> = Err("m_error".to_string());
        let f = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g = |y: f64| -> Result<String, String> { Ok(y.to_string()) };

        let lhs = <Result<f64, String> as Bind<f64>>::bind(
            <Result<i32, String> as Bind<i32>>::bind(m.clone(), f),
            g,
        );

        let f_inner = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g_inner = |y: f64| -> Result<String, String> { Ok(y.to_string()) };
        let inner_closure =
            move |x: i32| <Result<f64, String> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Result<i32, String> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("m_error".to_string()));
    }

    #[test]
    fn result_monad_associativity_f_returns_err() {
        let m: Result<i32, String> = Ok(10);
        let f = |_x: i32| -> Result<f64, String> { Err("f_error".to_string()) };
        let g = |y: f64| -> Result<String, String> { Ok(y.to_string()) };

        let lhs = <Result<f64, String> as Bind<f64>>::bind(
            <Result<i32, String> as Bind<i32>>::bind(m.clone(), f),
            g,
        );

        let f_inner = |_x: i32| -> Result<f64, String> { Err("f_error".to_string()) };
        let g_inner = |y: f64| -> Result<String, String> { Ok(y.to_string()) };
        let inner_closure =
            move |x: i32| <Result<f64, String> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Result<i32, String> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("f_error".to_string()));
    }

    #[test]
    fn result_monad_associativity_g_returns_err() {
        let m: Result<i32, String> = Ok(10);
        let f = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g = |_y: f64| -> Result<String, String> { Err("g_error".to_string()) };

        let lhs = <Result<f64, String> as Bind<f64>>::bind(
            <Result<i32, String> as Bind<i32>>::bind(m.clone(), f),
            g,
        );

        let f_inner = |x: i32| -> Result<f64, String> { Ok((x * 2) as f64) };
        let g_inner = |_y: f64| -> Result<String, String> { Err("g_error".to_string()) };
        let inner_closure =
            move |x: i32| <Result<f64, String> as Bind<f64>>::bind(f_inner(x), g_inner);
        let rhs = <Result<i32, String> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Err("g_error".to_string()));
    }
}

#[cfg(test)]
mod vec_monad_laws {
    use monadify::legacy::applicative::Applicative;
    use monadify::legacy::monad::Bind;

    #[test]
    fn vec_monad_left_identity() {
        let a = 10;
        let f = |x: i32| -> Vec<String> { vec![x.to_string(), (x + 1).to_string()] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Applicative<i32>>::pure(a), f);
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, vec!["10".to_string(), "11".to_string()]);
    }

    #[test]
    fn vec_monad_left_identity_f_returns_empty() {
        let a = 10;
        let f = |_x: i32| -> Vec<String> { vec![] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Applicative<i32>>::pure(a), f);
        let rhs = f(a);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    #[test]
    fn vec_monad_right_identity_non_empty() {
        let m = vec![10, 20];
        let pure_fn = |x: i32| <Vec<i32> as Applicative<i32>>::pure(x);

        let lhs = <Vec<i32> as Bind<i32>>::bind(m.clone(), pure_fn);
        let rhs = m;

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn vec_monad_right_identity_empty() {
        let m: Vec<i32> = vec![];
        let pure_fn = |x: i32| <Vec<i32> as Applicative<i32>>::pure(x);

        let lhs = <Vec<i32> as Bind<i32>>::bind(m.clone(), pure_fn);
        let rhs = m;

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<i32>::new());
    }

    #[test]
    fn vec_monad_associativity() {
        let m = vec![1, 2];
        let f = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Bind<i32>>::bind(m.clone(), f), g);

        let f_inner = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g_inner = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };
        let inner_closure = move |x: i32| <Vec<i32> as Bind<i32>>::bind(f_inner(x), g_inner);
        let rhs = <Vec<i32> as Bind<i32>>::bind(m, inner_closure);

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
    fn vec_monad_associativity_empty_start() {
        let m: Vec<i32> = vec![];
        let f = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Bind<i32>>::bind(m.clone(), f), g);

        let f_inner = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g_inner = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };
        let inner_closure = move |x: i32| <Vec<i32> as Bind<i32>>::bind(f_inner(x), g_inner);
        let rhs = <Vec<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    #[test]
    fn vec_monad_associativity_f_returns_empty() {
        let m = vec![1, 2];
        let f = |_x: i32| -> Vec<i32> { vec![] };
        let g = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Bind<i32>>::bind(m.clone(), f), g);

        let f_inner = |_x: i32| -> Vec<i32> { vec![] };
        let g_inner = |y: i32| -> Vec<String> { vec![y.to_string(), (y + 1).to_string()] };
        let inner_closure = move |x: i32| <Vec<i32> as Bind<i32>>::bind(f_inner(x), g_inner);
        let rhs = <Vec<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }

    #[test]
    fn vec_monad_associativity_g_returns_empty() {
        let m = vec![1, 2];
        let f = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g = |_y: i32| -> Vec<String> { vec![] };

        let lhs = <Vec<i32> as Bind<i32>>::bind(<Vec<i32> as Bind<i32>>::bind(m.clone(), f), g);

        let f_inner = |x: i32| -> Vec<i32> { vec![x, x * 10] };
        let g_inner = |_y: i32| -> Vec<String> { vec![] };
        let inner_closure = move |x: i32| <Vec<i32> as Bind<i32>>::bind(f_inner(x), g_inner);
        let rhs = <Vec<i32> as Bind<i32>>::bind(m, inner_closure);

        assert_eq!(lhs, rhs);
        assert_eq!(lhs, Vec::<String>::new());
    }
}
