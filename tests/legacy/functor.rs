// Original content from src/functor.rs mod tests, functor_laws, and result_functor_laws
// with `use super::Functor` changed to `use monadify::Functor`

// Note: The `crate::function::CFn` import was commented out in the original functor_laws,
// so it's not included here unless it was actually used.
// If other `crate::` or `super::` imports were present and used, they'd be adjusted similarly.
// For example, `use crate::function::CFn;` would become `use monadify::function::CFn;`

// These imports will need to point to legacy versions.
// For now, using placeholder paths that will be fixed in Phase 5.
// use monadify::legacy_functor::Functor;

#[cfg(test)]
mod classic_functor_tests {

    #[cfg(test)]
    mod tests {
        // This will use the Functor impl from monadify::legacy::functor
        #[allow(unused_imports)]
        use monadify::legacy::functor::Functor;

        #[test]
        fn add_one() {
            let closure = |x| x + 1;
            assert_eq!(
                <Option<i32> as Functor<i32>>::map(Some(1), closure),
                Some(2)
            )
        }
    }

    #[cfg(test)]
    mod functor_laws {
        #[allow(unused_imports)]
        use monadify::legacy::functor::Functor;
        // use monadify::legacy::function::CFn; // Example if CFn was from monadify::function and used

        #[test]
        fn option_functor_identity_some() {
            let opt = Some(10);
            let identity_fn = |x: i32| x;
            assert_eq!(
                <Option<i32> as Functor<i32>>::map(opt, identity_fn),
                Some(10)
            );
        }

        #[test]
        fn option_functor_identity_none() {
            let opt: Option<i32> = None;
            let identity_fn = |x: i32| x;
            assert_eq!(<Option<i32> as Functor<i32>>::map(opt, identity_fn), None);
        }

        #[test]
        fn option_functor_composition_some() {
            let opt = Some(10);
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map = <Option<i32> as Functor<i32>>::map(opt, move |x| g(f(x)));
            let sequential_map = <Option<i32> as Functor<i32>>::map(opt.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Some(25));
        }

        #[test]
        fn option_functor_composition_none() {
            let opt: Option<i32> = None;
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map = <Option<i32> as Functor<i32>>::map(opt, move |x| g(f(x)));
            let sequential_map = <Option<i32> as Functor<i32>>::map(opt.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, None);
        }

        #[test]
        fn option_functor_composition_some_str() {
            let opt = Some("hello");
            let f = |x: &str| x.to_uppercase();
            let g = |y: String| y.len();

            let composed_map = <Option<&str> as Functor<&str>>::map(opt, move |x| g(f(x)));
            let sequential_map = <Option<String> as Functor<String>>::map(opt.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Some(5));
        }
    }

    #[cfg(test)]
    mod result_functor_laws {
        #[allow(unused_imports)]
        use monadify::legacy::functor::Functor;

        #[test]
        fn result_functor_identity_ok() {
            let res: Result<i32, String> = Ok(10);
            let identity_fn = |x: i32| x;
            assert_eq!(
                <Result<i32, String> as Functor<i32>>::map(res, identity_fn),
                Ok(10)
            );
        }

        #[test]
        fn result_functor_identity_err() {
            let res: Result<i32, String> = Err("error".to_string());
            let identity_fn = |x: i32| x;
            assert_eq!(
                <Result<i32, String> as Functor<i32>>::map(res, identity_fn),
                Err("error".to_string())
            );
        }

        #[test]
        fn result_functor_composition_ok() {
            let res: Result<i32, String> = Ok(10);
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map =
                <Result<i32, String> as Functor<i32>>::map(res.clone(), move |x| g(f(x)));
            let sequential_map = <Result<i32, String> as Functor<i32>>::map(res.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Ok(25));
        }

        #[test]
        fn result_functor_composition_err() {
            let res: Result<i32, String> = Err("error".to_string());
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map =
                <Result<i32, String> as Functor<i32>>::map(res.clone(), move |x| g(f(x)));
            let sequential_map = <Result<i32, String> as Functor<i32>>::map(res.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Err("error".to_string()));
        }

        #[test]
        fn result_functor_composition_ok_str_err_u32() {
            let res: Result<&str, u32> = Ok("hello");
            let f = |x: &str| x.to_uppercase();
            let g = |y: String| y.len();

            let composed_map = <Result<&str, u32> as Functor<&str>>::map(res, move |x| g(f(x)));
            let sequential_map = <Result<String, u32> as Functor<String>>::map(res.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Ok(5));
        }

        #[test]
        fn result_functor_composition_err_str_err_u32() {
            let res: Result<&str, u32> = Err(404);
            let f = |x: &str| x.to_uppercase();
            let g = |y: String| y.len();

            let composed_map = <Result<&str, u32> as Functor<&str>>::map(res, move |x| g(f(x)));
            let sequential_map = <Result<String, u32> as Functor<String>>::map(res.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Err(404));
        }
    }

    #[cfg(test)]
    mod vec_functor_laws {
        use monadify::legacy::functor::Functor;

        #[test]
        fn vec_functor_identity_non_empty() {
            let vec_val = vec![10, 20, 30];
            let identity_fn = |x: i32| x;
            assert_eq!(
                <Vec<i32> as Functor<i32>>::map(vec_val.clone(), identity_fn),
                vec_val
            );
        }

        #[test]
        fn vec_functor_identity_empty() {
            let vec_val: Vec<i32> = vec![];
            let identity_fn = |x: i32| x;
            assert_eq!(
                <Vec<i32> as Functor<i32>>::map(vec_val.clone(), identity_fn),
                vec_val
            );
        }

        #[test]
        fn vec_functor_composition_non_empty() {
            let vec_val = vec![10, 20, 30];
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map = <Vec<i32> as Functor<i32>>::map(vec_val.clone(), move |x| g(f(x)));
            let sequential_map = <Vec<i32> as Functor<i32>>::map(vec_val.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, vec![25, 45, 65]);
        }

        #[test]
        fn vec_functor_composition_empty() {
            let vec_val: Vec<i32> = vec![];
            let f = |x: i32| x * 2;
            let g = |y: i32| y + 5;

            let composed_map = <Vec<i32> as Functor<i32>>::map(vec_val.clone(), move |x| g(f(x)));
            let sequential_map = <Vec<i32> as Functor<i32>>::map(vec_val.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, Vec::<i32>::new());
        }

        #[test]
        fn vec_functor_composition_str() {
            let vec_val = vec!["hello", "world"];
            let f = |x: &str| x.to_uppercase();
            let g = |y: String| y.len();

            let composed_map = <Vec<&str> as Functor<&str>>::map(vec_val.clone(), move |x| g(f(x)));
            let sequential_map = <Vec<String> as Functor<String>>::map(vec_val.map(f), g);

            assert_eq!(composed_map, sequential_map);
            assert_eq!(composed_map, vec![5, 5]);
        }
    }
}
