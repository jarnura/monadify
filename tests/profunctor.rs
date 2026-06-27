// Original content from src/profunctor.rs mod tests
// with use statements adjusted for the new location.

// Items re-exported from lib.rs
use monadify::Profunctor; // These are re-exported

// Items specific to the profunctor module
use monadify::profunctor::{_key, lcmap, rmap, view, Check, _1, _2};

// Items from other modules
use monadify::fn1; // Macro is at crate root

#[cfg(test)]
mod tests {
    // Bring all top-level imports from this file into the module's scope
    use super::*;

    #[test]
    fn test_fn_dimap() {
        let closure = fn1!(|x: i32| format!("{x}"));
        let proclosure = closure.dimap(|x: i8| (x + 1).into(), |s| vec![s]);
        let result = proclosure.call(1i8);
        assert_eq!(result, vec!["2"])
    }

    #[test]
    fn test_fn_lcmap() {
        let profunctor_val = fn1!(|x: i32| format!("{x}"));
        let proclosure = lcmap(|x: i8| x as i32 + 1, profunctor_val);
        let result = proclosure.call(1i8);
        assert_eq!(result, "2")
    }

    #[test]
    fn test_fn_rmap() {
        let profunctor_val = fn1!(|x: i32| format!("{x}"));
        let proclosure = rmap(|s| vec![s], profunctor_val);
        let result = proclosure.call(1i32);
        assert_eq!(result, vec!["1"])
    }

    #[test]
    fn test_fn_rmap_with_identity() {
        let profunctor_val = fn1!(|x: i32| x);
        let proclosure = rmap(|s| vec![s], profunctor_val);
        let result = proclosure.call(1i32);
        assert_eq!(result, vec![1])
    }

    #[test]
    fn test_1() {
        let tuple = (1, 3);
        let r = view::<_, _, _, ()>(_1().into(), tuple);
        assert_eq!(r, 1);
        let r = view::<_, _, _, ()>(_2().into(), tuple);
        assert_eq!(r, 3)
    }

    #[test]
    fn test_key() {
        let rec = Check { key: 1, other: 1 };
        let r = view(_key().0, rec);
        assert_eq!(r, 1);
    }
}

#[cfg(test)]
mod profunctor_laws {
    use monadify::function::RcFn;
    use monadify::Profunctor;

    // Helper identity function
    fn identity<T>(x: T) -> T {
        x
    }

    // Law 1: p.dimap(id, id) == p
    // We test by applying the same input and checking for equal output.
    #[test]
    fn profunctor_identity_law() {
        let p: RcFn<i32, String> = RcFn::new(|x: i32| x.to_string());
        let input = 123;

        // p.dimap(id, id)
        let p_lhs: RcFn<i32, String> = RcFn::new(|x: i32| x.to_string());
        let lhs_p = p_lhs.dimap(identity::<i32>, identity::<String>);

        let lhs_result = lhs_p.call(input);
        let rhs_result = p.call(input);

        assert_eq!(lhs_result, rhs_result);
        assert_eq!(lhs_result, "123".to_string());
    }

    // Law 2: p.dimap(h, i).dimap(f, g) == p.dimap(f . h, g . i)
    #[test]
    fn profunctor_composition_law() {
        // p: B -> C  (i32 -> String)
        let _p: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        // h: A -> B  (u16 -> i32)
        let h = |x: u16| x as i32 + 10;
        // i: C -> D  (String -> usize)
        let i = |s: String| s.len();
        // f: X -> A  (u8 -> u16)
        let f = |x: u8| x as u16 * 2;
        // g: D -> Y  (usize -> usize)
        let g = |x: usize| x + 1;

        let input: u8 = 5;

        // LHS: p.dimap(h, i).dimap(f, g)
        let p_lhs1: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_hi = p_lhs1.dimap(h, i); // RcFn<u16, usize>
        let lhs_p = p_hi.dimap(f, g); // RcFn<u8, usize>
        let lhs_result = lhs_p.call(input);

        // RHS: p.dimap(f . h, g . i)
        let f_dot_h = move |x: u8| h(f(x));
        let g_dot_i = move |s: String| g(i(s));
        let p_rhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let rhs_p = p_rhs.dimap(f_dot_h, g_dot_i);
        let rhs_result = rhs_p.call(input);

        assert_eq!(lhs_result, rhs_result);
        assert_eq!(lhs_result, 10);
    }
}

#[cfg(test)]
mod strong_laws {
    use monadify::function::RcFn;
    use monadify::{Profunctor, Strong};

    // Helper identity function
    fn identity<T>(x: T) -> T {
        x
    }

    // Helper split function (***)
    fn split<A, B, C, D>(
        f: impl Fn(A) -> C + 'static,
        g: impl Fn(B) -> D + 'static,
    ) -> impl Fn((A, B)) -> (C, D) + 'static {
        move |(a, b)| (f(a), g(b))
    }

    // Law: p.first().dimap(split(f, id), split(g, id)) == p.dimap(f, g).first()
    #[test]
    fn strong_first_dimap_law() {
        // p: i32 -> String
        let _p: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input: (u16, X) = (5, 99);

        // LHS: p.first().dimap(split(f, id), split(g, id))
        let p_lhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_first = p_lhs.first::<X>();
        let lhs_p = p_first.dimap(split(f, identity::<X>), split(g, identity::<X>));
        let lhs_result = lhs_p.call(input);

        // RHS: p.dimap(f, g).first()
        let p_rhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_fg = p_rhs.dimap(f, g);
        let rhs_p = p_fg.first::<X>();
        let rhs_result = rhs_p.call(input);

        assert_eq!(lhs_result, rhs_result);
        // input = (5, 99), f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result, (9, 99));
    }

    // Similar law for second
    #[test]
    fn strong_second_dimap_law() {
        // p: i32 -> String
        let _p: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input: (X, u16) = (99, 5);

        // LHS: p.second().dimap(split(id, f), split(id, g))
        let p_lhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_second = p_lhs.second::<X>();
        let lhs_p = p_second.dimap(split(identity::<X>, f), split(identity::<X>, g));
        let lhs_result = lhs_p.call(input);

        // RHS: p.dimap(f, g).second()
        let p_rhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_fg = p_rhs.dimap(f, g);
        let rhs_p = p_fg.second::<X>();
        let rhs_result = rhs_p.call(input);

        assert_eq!(lhs_result, rhs_result);
        // input = (99, 5), f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result, (99, 9));
    }

    // Law: p.first().first() == p.first().dimap(assoc, inv_assoc)
    #[test]
    fn strong_associativity_law() {
        // p: A -> B (i32 -> String)
        let _p_orig: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));

        type X = u8;
        type Y = bool;

        let input: ((i32, X), Y) = ((10, 20u8), true);

        // LHS: p.first().first()
        let p_lhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let lhs = p_lhs.first::<X>().first::<Y>();
        let lhs_result = lhs.call(input);

        // RHS: p.first().dimap(assoc, inv_assoc)
        let p_rhs: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let p_first_intermediate = p_rhs.first::<(X, Y)>();

        let assoc = |((a, x), y): ((i32, X), Y)| (a, (x, y));
        let inv_assoc = |(b, (x, y)): (String, (X, Y))| ((b, x), y);

        let rhs = p_first_intermediate.dimap(assoc, inv_assoc);
        let rhs_result = rhs.call(input);

        assert_eq!(lhs_result, rhs_result);
        // input = ((10, 20), true), p(10) = "Value: 10"
        assert_eq!(lhs_result, (("Value: 10".to_string(), 20u8), true));
    }
}

#[cfg(test)]
mod choice_laws {
    use monadify::function::RcFn;
    use monadify::{Choice, Profunctor};

    // Helper function for Choice laws
    fn map_result<A, B, C, F: Fn(A) -> B>(f: F, r: Result<C, A>) -> Result<C, B> {
        match r {
            Ok(c) => Ok(c),
            Err(a) => Err(f(a)),
        }
    }

    // Law: p.left().dimap(map_result(f, id), map_result(g, id)) == p.dimap(f, g).left()
    #[test]
    fn choice_left_dimap_law() {
        // p: i32 -> String
        let _p: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input_err: Result<X, u16> = Err(5);
        let input_ok: Result<X, u16> = Ok(99);

        // --- LHS (Err) ---
        let p_lhs_err: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let lhs_p_err = p_lhs_err.left::<X>().dimap(
            move |r: Result<X, u16>| map_result(f, r),
            move |r: Result<X, String>| map_result(g, r),
        );
        let lhs_result_err = lhs_p_err.call(input_err);

        // --- RHS (Err) ---
        let p_rhs_err: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let rhs_p_err = p_rhs_err.dimap(f, g).left::<X>();
        let rhs_result_err = rhs_p_err.call(input_err);

        assert_eq!(lhs_result_err, rhs_result_err);
        // f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result_err, Err(9));

        // --- LHS (Ok) ---
        let p_lhs_ok: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let lhs_p_ok = p_lhs_ok.left::<X>().dimap(
            move |r: Result<X, u16>| map_result(f, r),
            move |r: Result<X, String>| map_result(g, r),
        );
        let lhs_result_ok = lhs_p_ok.call(input_ok);

        // --- RHS (Ok) ---
        let p_rhs_ok: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let rhs_p_ok = p_rhs_ok.dimap(f, g).left::<X>();
        let rhs_result_ok = rhs_p_ok.call(input_ok);

        assert_eq!(lhs_result_ok, rhs_result_ok);
        assert_eq!(lhs_result_ok, Ok(99));
    }

    // Helper function for right
    fn map_result_right<A, B, C, F: Fn(A) -> B>(f: F, r: Result<A, C>) -> Result<B, C> {
        match r {
            Ok(a) => Ok(f(a)),
            Err(c) => Err(c),
        }
    }

    #[test]
    fn choice_right_dimap_law() {
        // p: i32 -> String
        let _p: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input_ok: Result<u16, X> = Ok(5);
        let input_err: Result<u16, X> = Err(99);

        // --- LHS (Ok) ---
        let p_lhs_ok: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let lhs_p_ok = p_lhs_ok.right::<X>().dimap(
            move |r: Result<u16, X>| map_result_right(f, r),
            move |r: Result<String, X>| map_result_right(g, r),
        );
        let lhs_result_ok = lhs_p_ok.call(input_ok);

        // --- RHS (Ok) ---
        let p_rhs_ok: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let rhs_p_ok = p_rhs_ok.dimap(f, g).right::<X>();
        let rhs_result_ok = rhs_p_ok.call(input_ok);

        assert_eq!(lhs_result_ok, rhs_result_ok);
        // f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result_ok, Ok(9));

        // --- LHS (Err) ---
        let p_lhs_err: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let lhs_p_err = p_lhs_err.right::<X>().dimap(
            move |r: Result<u16, X>| map_result_right(f, r),
            move |r: Result<String, X>| map_result_right(g, r),
        );
        let lhs_result_err = lhs_p_err.call(input_err);

        // --- RHS (Err) ---
        let p_rhs_err: RcFn<i32, String> = RcFn::new(|x| format!("Value: {x}"));
        let rhs_p_err = p_rhs_err.dimap(f, g).right::<X>();
        let rhs_result_err = rhs_p_err.call(input_err);

        assert_eq!(lhs_result_err, rhs_result_err);
        assert_eq!(lhs_result_err, Err(99));
    }
}
