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
        // Assuming Profunctor is in scope via `use monadify::Profunctor;` at the top of this test file
        // or `use super::Profunctor;` if it were defined in this file's parent.
        // Since Profunctor is a trait, `closure.dimap` should work if `closure` (a CFn) implements Profunctor.
        let proclosure = closure.dimap(|x: i8| (x + 1).into(), |s| vec![s]);
        let result = proclosure(1);
        assert_eq!(result, vec!["2"])
    }

    #[test]
    fn test_fn_lcmap() {
        let profunctor_val = fn1!(|x: i32| format!("{x}")); // Renamed from profunctor to avoid conflict
        let proclosure = lcmap(|x: i8| x as i32 + 1, profunctor_val);
        let result = proclosure(1);
        assert_eq!(result, "2")
    }

    #[test]
    fn test_fn_rmap() {
        let profunctor_val = fn1!(|x: i32| format!("{x}")); // Renamed
        let proclosure = rmap(|s| vec![s], profunctor_val);
        let result = proclosure(1);
        assert_eq!(result, vec!["1"])
    }

    #[test]
    fn test_fn_rmap_with_identity() {
        let profunctor_val = fn1!(|x: i32| x); // Renamed
        let proclosure = rmap(|s| vec![s], profunctor_val);
        let result = proclosure(1);
        assert_eq!(result, vec![1])
    }

    #[test]
    fn test_1() {
        let tuple = (1, 3);
        // The AGetter type alias might need to be defined or imported if it's not automatically resolved.
        // For now, assuming view can infer types or AGetter is accessible.
        // AGetter is `Fold<A, S, T, A, B>`, Fold is `Optic<Forget<R,S,T>, Forget<R,A,B>, S,T,A,B>`
        // These are complex types defined in profunctor.rs.
        // The `_1()` function returns a `Lens`. `into()` converts it to `Optic`.
        // This should work if `Lens`, `Optic`, `Forget`, `view`, `_1`, `_2` are in scope.
        let r = view::<_, _, _, ()>(_1().into(), tuple);
        assert_eq!(r, 1);
        let r = view::<_, _, _, ()>(_2().into(), tuple);
        assert_eq!(r, 3)
    }

    #[test]
    fn test_key() {
        let rec = Check { key: 1, other: 1 }; // Check struct needs to be in scope
        let r = view(_key().0, rec); // _key() returns Lens, .0 accesses the Optic inside
        assert_eq!(r, 1);
    }
}

#[cfg(test)]
mod profunctor_laws {
    use monadify::function::CFn;
    use monadify::Profunctor;

    // Helper identity function
    fn identity<T>(x: T) -> T {
        x
    }

    // Law 1: p.dimap(id, id) == p
    // We need a way to compare Profunctors (CFn in this case).
    // Since CFn wraps Box<dyn Fn>, direct comparison is not possible.
    // We test by applying the same input and checking for equal output.
    #[test]
    fn profunctor_identity_law() {
        // Add type annotation for x
        let p: CFn<i32, String> = CFn::new(|x: i32| x.to_string());
        let input = 123;

        // p.dimap(id, id)
        // Recreate p for lhs instead of cloning
        let p_lhs: CFn<i32, String> = CFn::new(|x: i32| x.to_string());
        let lhs_p = p_lhs.dimap(identity::<i32>, identity::<String>);

        // Apply input
        let lhs_result = lhs_p(input);
        let rhs_result = p(input); // Apply to original p

        assert_eq!(lhs_result, rhs_result);
        assert_eq!(lhs_result, "123".to_string());
    }

    // Law 2: p.dimap(f, g).dimap(h, i) == p.dimap(f . h, i . g)
    // Law 2 rewritten: p.dimap(h, i).dimap(f, g) == p.dimap(f . h, g . i)
    // Let p: CFn<B, C>
    // f: A -> B
    // g: C -> D
    // h: X -> A
    // i: D -> Y
    // p.dimap(h, i) -> CFn<X, Y>
    // p.dimap(h, i).dimap(f, g) -> This doesn't match the types easily.
    // The law is: p.dimap(h, i).dimap(f, g) == p.dimap(f . h, g . i)
    // Let p: B -> C
    // h: A -> B
    // i: C -> D
    // f: X -> A
    // g: D -> Y
    #[test]
    fn profunctor_composition_law() {
        // p: B -> C  (i32 -> String)
        let _p: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _
                                                                        // h: A -> B  (u16 -> i32)
        let h = |x: u16| x as i32 + 10;
        // i: C -> D  (String -> usize) - Simplified
        let i = |s: String| s.len();
        // f: X -> A  (u8 -> u16)
        let f = |x: u8| x as u16 * 2;
        // g: D -> Y  (usize -> usize) - Simplified
        let g = |x: usize| x + 1;

        let input: u8 = 5; // Input type X

        // LHS: p.dimap(h, i).dimap(f, g)
        let p_lhs1: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Recreate p
        let p_hi = p_lhs1.dimap(h, i); // CFn<u16, usize> (A -> D)
        let lhs_p = p_hi.dimap(f, g); // CFn<u8, usize> (X -> Y)
        let lhs_result = lhs_p(input); // Y

        // RHS: p.dimap(f . h, g . i)
        // f . h : X -> B (u8 -> i32)
        let f_dot_h = move |x: u8| h(f(x));
        // g . i : C -> Y (String -> usize)
        let g_dot_i = move |s: String| g(i(s));
        let p_rhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Recreate p
        let rhs_p = p_rhs.dimap(f_dot_h, g_dot_i); // CFn<u8, usize> (X -> Y)
        let rhs_result = rhs_p(input); // Y

        assert_eq!(lhs_result, rhs_result); // Check the law holds

        // Manual check (simplified): g(i(p(h(f(input)))))
        // f(5) = 10 (u16)
        // h(10) = 20 (i32)
        // p(20) = "Value: 20" (String)
        // i("Value: 20") = 10 (usize)
        // g(10) = 11 (usize) -> Incorrect trace, g(9) was called, result is 10
        assert_eq!(lhs_result, 10); // Verify the actual value (corrected)
    }
}

#[cfg(test)]
mod strong_laws {
    use monadify::function::CFn;
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
    // Let p: B -> C
    // f: A -> B
    // g: C -> D
    // id: X -> X (for some type X)
    #[test]
    fn strong_first_dimap_law() {
        // p: i32 -> String
        let _p: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _
                                                                        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input: (u16, X) = (5, 99); // Input type (A, X)

        // LHS: p.first().dimap(split(f, id), split(g, id))
        let p_lhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let p_first = p_lhs.first::<X>(); // CFn<(i32, X), (String, X)>
        let lhs_p = p_first.dimap(split(f, identity::<X>), split(g, identity::<X>)); // CFn<(u16, X), (usize, X)>
        let lhs_result = lhs_p(input);

        // RHS: p.dimap(f, g).first()
        let p_rhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let p_fg = p_rhs.dimap(f, g); // CFn<u16, usize>
        let rhs_p = p_fg.first::<X>(); // CFn<(u16, X), (usize, X)>
        let rhs_result = rhs_p(input);

        assert_eq!(lhs_result, rhs_result);

        // Manual check:
        // input = (5, 99)
        // f(5) = 15
        // p(15) = "Value: 15"
        // g("Value: 15") = 9 (length of "Value: 15")
        // Expected output: (9, 99)
        assert_eq!(lhs_result, (9, 99));
    }

    // Similar law for second: p.second().dimap(split(id, f), split(id, g)) == p.dimap(f, g).second()
    #[test]
    fn strong_second_dimap_law() {
        // p: i32 -> String
        let _p: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _
                                                                        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input: (X, u16) = (99, 5); // Input type (X, A)

        // LHS: p.second().dimap(split(id, f), split(id, g))
        let p_lhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let p_second = p_lhs.second::<X>(); // CFn<(X, i32), (X, String)>
        let lhs_p = p_second.dimap(split(identity::<X>, f), split(identity::<X>, g)); // CFn<(X, u16), (X, usize)>
        let lhs_result = lhs_p(input);

        // RHS: p.dimap(f, g).second()
        let p_rhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let p_fg = p_rhs.dimap(f, g); // CFn<u16, usize>
        let rhs_p = p_fg.second::<X>(); // CFn<(X, u16), (X, usize)>
        let rhs_result = rhs_p(input);

        assert_eq!(lhs_result, rhs_result);

        // Manual check:
        // input = (99, 5)
        // f(5) = 15
        // p(15) = "Value: 15"
        // g("Value: 15") = 9 (length of "Value: 15")
        // Expected output: (99, 9)
        assert_eq!(lhs_result, (99, 9));
    }

    // Other laws exist relating first/second composition, etc.
    // e.g., p.first().second() == p.dimap(swap, swap).second().first()
    // For now, testing the interaction with dimap is a good start.

    // Law: p.first().first() == p.first().dimap(assoc, inv_assoc)
    // where assoc(((a,b),c)) = (a,(b,c))
    // and inv_assoc((a,(b,c))) = ((a,b),c)
    #[test]
    fn strong_associativity_law() {
        // p: A -> B (i32 -> String)
        let _p_orig: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _

        // Types for the tuple elements
        type X = u8;
        type Y = bool;

        let input: ((i32, X), Y) = ((10, 20u8), true);

        // LHS: p.first().first()
        let p_lhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Recreate p
        let lhs = p_lhs.first::<X>().first::<Y>();
        let lhs_result = lhs(input);

        // RHS: p.first().dimap(assoc, inv_assoc)
        let p_rhs: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Recreate p
        let p_first_intermediate = p_rhs.first::<(X, Y)>(); // p.first() for type (A, (X,Y)) -> (B, (X,Y))

        let assoc = |((a, x), y): ((i32, X), Y)| (a, (x, y));
        let inv_assoc = |(b, (x, y)): (String, (X, Y))| ((b, x), y);

        let rhs = p_first_intermediate.dimap(assoc, inv_assoc);
        let rhs_result = rhs(input);

        assert_eq!(lhs_result, rhs_result);

        // Manual check:
        // input = ((10, 20), true)
        // p(10) = "Value: 10"
        // Expected output: (("Value: 10", 20), true)
        assert_eq!(lhs_result, (("Value: 10".to_string(), 20u8), true));
    }
}

#[cfg(test)]
mod choice_laws {
    use monadify::function::CFn;
    use monadify::{Choice, Profunctor};

    // Helper identity function (Removed as unused in this module)
    // fn identity<T>(x: T) -> T {
    //     x
    // }

    // Helper function for Choice laws
    fn map_result<A, B, C, F: Fn(A) -> B>(f: F, r: Result<C, A>) -> Result<C, B> {
        match r {
            Ok(c) => Ok(c),
            Err(a) => Err(f(a)),
        }
    }

    // Law: p.left().dimap(map_result(f, id), map_result(g, id)) == p.dimap(f, g).left()
    // Let p: B -> C
    // f: A -> B
    // g: C -> D
    // id: X -> X (for some type X)
    #[test]
    fn choice_left_dimap_law() {
        // p: i32 -> String
        let _p: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _
                                                                        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input_err: Result<X, u16> = Err(5); // Input type Result<X, A>
        let input_ok: Result<X, u16> = Ok(99);

        // --- LHS ---
        // Calculate LHS for Err input
        let p_lhs_err: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let lhs_p_err = p_lhs_err.left::<X>().dimap(
            move |r: Result<X, u16>| map_result(f, r),    // Added move
            move |r: Result<X, String>| map_result(g, r), // Added move
        );
        let lhs_result_err = lhs_p_err(input_err);

        // --- RHS ---
        // Calculate RHS for Err input
        let p_rhs_err: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let rhs_p_err = p_rhs_err.dimap(f, g).left::<X>();
        let rhs_result_err = rhs_p_err(input_err);

        // --- Assertions for Err ---
        assert_eq!(lhs_result_err, rhs_result_err);
        // f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result_err, Err(9));

        // Test with Ok input
        // Calculate LHS for Ok input
        let p_lhs_ok: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let lhs_p_ok = p_lhs_ok.left::<X>().dimap(
            move |r: Result<X, u16>| map_result(f, r),    // Added move
            move |r: Result<X, String>| map_result(g, r), // Added move
        );
        let lhs_result_ok = lhs_p_ok(input_ok);

        // Calculate RHS for Ok input
        let p_rhs_ok: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let rhs_p_ok = p_rhs_ok.dimap(f, g).left::<X>();
        let rhs_result_ok = rhs_p_ok(input_ok);

        // --- Assertions for Ok ---
        assert_eq!(lhs_result_ok, rhs_result_ok);
        // Manual check: Ok(99) -> Ok(99) -> Ok(99) -> Ok(99)
        assert_eq!(lhs_result_ok, Ok(99));
    }

    // Similar law for right: p.right().dimap(map_result(id, f), map_result(id, g)) == p.dimap(f, g).right()
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
        let _p: CFn<i32, String> = CFn::new(|x| format!("Value: {x}")); // Prefixed with _
                                                                        // f: u16 -> i32
        let f = |x: u16| x as i32 + 10;
        // g: String -> usize
        let g = |s: String| s.len();
        // X will be u8
        type X = u8;

        let input_ok: Result<u16, X> = Ok(5); // Input type Result<A, X>
        let input_err: Result<u16, X> = Err(99);

        // --- LHS ---
        // Calculate LHS for Ok input
        let p_lhs_ok: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let lhs_p_ok = p_lhs_ok.right::<X>().dimap(
            move |r: Result<u16, X>| map_result_right(f, r), // Added move
            move |r: Result<String, X>| map_result_right(g, r), // Added move
        );
        let lhs_result_ok = lhs_p_ok(input_ok);

        // --- RHS ---
        // Calculate RHS for Ok input
        let p_rhs_ok: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let rhs_p_ok = p_rhs_ok.dimap(f, g).right::<X>();
        let rhs_result_ok = rhs_p_ok(input_ok);

        // --- Assertions for Ok ---
        assert_eq!(lhs_result_ok, rhs_result_ok);
        // f(5) = 15, p(15) = "Value: 15", g("Value: 15") = 9
        assert_eq!(lhs_result_ok, Ok(9));

        // Test with Err input
        // Calculate LHS for Err input
        let p_lhs_err: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let lhs_p_err = p_lhs_err.right::<X>().dimap(
            move |r: Result<u16, X>| map_result_right(f, r), // Added move
            move |r: Result<String, X>| map_result_right(g, r), // Added move
        );
        let lhs_result_err = lhs_p_err(input_err);

        // Calculate RHS for Err input
        let p_rhs_err: CFn<i32, String> = CFn::new(|x| format!("Value: {x}"));
        let rhs_p_err = p_rhs_err.dimap(f, g).right::<X>();
        let rhs_result_err = rhs_p_err(input_err);

        // --- Assertions for Err ---
        assert_eq!(lhs_result_err, rhs_result_err);
        // Manual check: Err(99) -> Err(99) -> Err(99) -> Err(99)
        assert_eq!(lhs_result_err, Err(99));
    }
}
