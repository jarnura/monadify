#[cfg(test)]
mod classic_apply_tests {
    // The lift2, lift3 functions are from crate::legacy::apply
    // The Apply and Functor traits are from crate::legacy::apply and crate::legacy::functor
    // Macros fn2, fn3 are assumed to be available via crate::fn2, crate::fn3

    #[test]
    fn apply_on_option() {
        let closure = monadify::fn2!(|x: i32| move |y: i8| format!("{x}{y}"));
        // Option::map uses Functor trait
        let some_closure =
            <Option<i32> as monadify::legacy::functor::Functor<i32>>::map(Some(1), closure);
        let none_closure =
            <Option<i32> as monadify::legacy::functor::Functor<i32>>::map(None, closure);

        // Option::apply uses Apply trait
        assert_eq!(
            <Option<i8> as monadify::legacy::apply::Apply<i8>>::apply(Some(2), some_closure),
            Some("12".to_string())
        );
        assert_eq!(
            <Option<i8> as monadify::legacy::apply::Apply<i8>>::apply(Some(2), none_closure),
            None
        );

        let closure_lift = monadify::fn2!(|x: i32| move |y: i8| format!("{x}{y}"));
        assert_eq!(
            monadify::legacy::apply::lift2(closure_lift, Some(1), Some(2)),
            Some("12".to_string())
        );
        assert_eq!(
            monadify::legacy::apply::lift2(closure_lift, None, Some(2)),
            None
        );

        let closure_lift3 = monadify::fn3!(|x: i32| move |y: i8| move |z: i32| x + y as i32 + z);
        assert_eq!(
            monadify::legacy::apply::lift3(closure_lift3, Some(1), Some(2), Some(3)),
            Some(6)
        );
    }
}
