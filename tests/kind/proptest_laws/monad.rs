//! Monad law property tests for the kind-based data instances.
//!
//! Covers the **left-identity**, **right-identity** and **associativity** laws
//! over the four property-testable Monad instances per the gap matrix — twelve
//! cells total (Option, Result<i32, String>, Vec, Identity).
//!
//! `Vec`'s `bind` is `flat_map`, which moves each element through the Kleisli
//! arrow without ever cloning a `CFn`, so `Vec` is property-testable for all
//! three laws. `CFn` / `CFnOnce` / `ReaderT` are function-typed and remain
//! example-only.
//!
//! Companion to the example-based tests in `tests/kind/monad.rs`; the shared
//! strategies live in the parent module (`super`) and are reused here, not
//! redefined.
//!
//! Laws (with `bind(m, f)` spelling `m >>= f`):
//! - **left identity:** `bind(pure(a), f) == f(a)`
//! - **right identity:** `bind(m, pure) == m`
//! - **associativity:** `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`
//!
//! The Kleisli arrows `f`/`g` return values *in* the monad and are materialized
//! from generated scalar parameters (a presence/Ok flag, `linear_fn`
//! slope/intercept pairs, and — for `Vec` — a small generated-length result),
//! rebuilt fresh per use because `CFn` is not `Clone`. `wrapping_*` arithmetic
//! (via `linear_fn`) avoids overflow panics on arbitrary `i32` inputs. The
//! generated arms deliberately include the short-circuiting cases (`None`,
//! `Err`, and the empty `Vec`).

use super::{
    arb_identity_i32, arb_linear_closure_params, arb_option_i32, arb_result_i32_string,
    arb_vec_i32, linear_fn,
};
use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
use monadify::monad::kind::Bind;
use proptest::prelude::*;

type TestError = String;

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Option (Kleisli arrow short-circuits via `None`) ---

    /// Monad left identity for `Option`: `bind(pure(a), f) == f(a)`.
    #[test]
    fn option_monad_left_identity(
        a in any::<i32>(),
        (fa, fb) in arb_linear_closure_params(),
        present in any::<bool>(),
    ) {
        let f = move |x: i32| -> Option<i32> {
            if present {
                let mut lf = linear_fn(fa, fb);
                Some(lf(x))
            } else {
                None
            }
        };
        let lhs = OptionKind::bind(OptionKind::pure(a), f);
        let rhs = f(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Monad right identity for `Option`: `bind(m, pure) == m`.
    #[test]
    fn option_monad_right_identity(m in arb_option_i32()) {
        let lhs = OptionKind::bind(m, |x: i32| OptionKind::pure(x));
        prop_assert_eq!(lhs, m);
    }

    /// Monad associativity for `Option`:
    /// `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn option_monad_associativity(
        m in arb_option_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
        f_present in any::<bool>(),
        g_present in any::<bool>(),
    ) {
        let f = move |x: i32| -> Option<i32> {
            if f_present {
                let mut lf = linear_fn(fa, fb);
                Some(lf(x))
            } else {
                None
            }
        };
        let g = move |y: i32| -> Option<i32> {
            if g_present {
                let mut lg = linear_fn(ga, gb);
                Some(lg(y))
            } else {
                None
            }
        };

        let lhs = OptionKind::bind(OptionKind::bind(m, f), g);

        let f2 = f;
        let g2 = g;
        let rhs = OptionKind::bind(m, move |x: i32| OptionKind::bind(f2(x), g2));

        prop_assert_eq!(lhs, rhs);
    }

    // --- Result<i32, String> (Kleisli arrow short-circuits via `Err`) ---

    /// Monad left identity for `Result<i32, String>`: `bind(pure(a), f) == f(a)`.
    #[test]
    fn result_monad_left_identity(
        a in any::<i32>(),
        (fa, fb) in arb_linear_closure_params(),
        f_ok in any::<bool>(),
        e in ".*",
    ) {
        let f = move |x: i32| -> Result<i32, TestError> {
            if f_ok {
                let mut lf = linear_fn(fa, fb);
                Ok(lf(x))
            } else {
                Err(e.clone())
            }
        };
        let lhs = ResultKind::<TestError>::bind(ResultKind::<TestError>::pure(a), f.clone());
        let rhs = f(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Monad right identity for `Result<i32, String>`: `bind(m, pure) == m`.
    #[test]
    fn result_monad_right_identity(m in arb_result_i32_string()) {
        let lhs =
            ResultKind::<TestError>::bind(m.clone(), |x: i32| ResultKind::<TestError>::pure(x));
        prop_assert_eq!(lhs, m);
    }

    /// Monad associativity for `Result<i32, String>`:
    /// `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn result_monad_associativity(
        m in arb_result_i32_string(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
        f_ok in any::<bool>(),
        g_ok in any::<bool>(),
        ef in ".*",
        eg in ".*",
    ) {
        let f = move |x: i32| -> Result<i32, TestError> {
            if f_ok {
                let mut lf = linear_fn(fa, fb);
                Ok(lf(x))
            } else {
                Err(ef.clone())
            }
        };
        let g = move |y: i32| -> Result<i32, TestError> {
            if g_ok {
                let mut lg = linear_fn(ga, gb);
                Ok(lg(y))
            } else {
                Err(eg.clone())
            }
        };

        let lhs = ResultKind::<TestError>::bind(
            ResultKind::<TestError>::bind(m.clone(), f.clone()),
            g.clone(),
        );

        let f2 = f.clone();
        let g2 = g.clone();
        let rhs = ResultKind::<TestError>::bind(m, move |x: i32| {
            ResultKind::<TestError>::bind(f2(x), g2.clone())
        });

        prop_assert_eq!(lhs, rhs);
    }

    // --- Vec (flat_map cross product; includes the empty-vec case) ---

    /// Monad left identity for `Vec`: `bind(pure(a), f) == f(a)`. The Kleisli
    /// arrow yields a small generated-length `Vec` (including the empty case).
    #[test]
    fn vec_monad_left_identity(
        a in any::<i32>(),
        fparams in prop::collection::vec(arb_linear_closure_params(), 0..=3),
    ) {
        let f = move |x: i32| -> Vec<i32> {
            fparams
                .iter()
                .map(|&(p, q)| {
                    let mut lf = linear_fn(p, q);
                    lf(x)
                })
                .collect()
        };
        let lhs = VecKind::bind(VecKind::pure(a), f.clone());
        let rhs = f(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Monad right identity for `Vec`: `bind(m, pure) == m`.
    #[test]
    fn vec_monad_right_identity(m in arb_vec_i32()) {
        let lhs = VecKind::bind(m.clone(), |x: i32| VecKind::pure(x));
        prop_assert_eq!(lhs, m);
    }

    /// Monad associativity for `Vec`:
    /// `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))` — exercises the
    /// `flat_map` cross product, including empty intermediate results.
    #[test]
    fn vec_monad_associativity(
        m in arb_vec_i32(),
        fparams in prop::collection::vec(arb_linear_closure_params(), 0..=3),
        gparams in prop::collection::vec(arb_linear_closure_params(), 0..=3),
    ) {
        let f = move |x: i32| -> Vec<i32> {
            fparams
                .iter()
                .map(|&(p, q)| {
                    let mut lf = linear_fn(p, q);
                    lf(x)
                })
                .collect()
        };
        let g = move |y: i32| -> Vec<i32> {
            gparams
                .iter()
                .map(|&(p, q)| {
                    let mut lg = linear_fn(p, q);
                    lg(y)
                })
                .collect()
        };

        let lhs = VecKind::bind(VecKind::bind(m.clone(), f.clone()), g.clone());

        let f2 = f.clone();
        let g2 = g.clone();
        let rhs = VecKind::bind(m, move |x: i32| VecKind::bind(f2(x), g2.clone()));

        prop_assert_eq!(lhs, rhs);
    }

    // --- Identity (no short-circuit arm) ---

    /// Monad left identity for `Identity`: `bind(pure(a), f) == f(a)`.
    #[test]
    fn identity_monad_left_identity(
        a in any::<i32>(),
        (fa, fb) in arb_linear_closure_params(),
    ) {
        let f = move |x: i32| -> Identity<i32> {
            let mut lf = linear_fn(fa, fb);
            Identity(lf(x))
        };
        let lhs = IdentityKind::bind(IdentityKind::pure(a), f);
        let rhs = f(a);
        prop_assert_eq!(lhs, rhs);
    }

    /// Monad right identity for `Identity`: `bind(m, pure) == m`.
    #[test]
    fn identity_monad_right_identity(i in arb_identity_i32()) {
        let lhs = IdentityKind::bind(i.clone(), |x: i32| IdentityKind::pure(x));
        prop_assert_eq!(lhs, i);
    }

    /// Monad associativity for `Identity`:
    /// `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn identity_monad_associativity(
        i in arb_identity_i32(),
        (fa, fb) in arb_linear_closure_params(),
        (ga, gb) in arb_linear_closure_params(),
    ) {
        let f = move |x: i32| -> Identity<i32> {
            let mut lf = linear_fn(fa, fb);
            Identity(lf(x))
        };
        let g = move |y: i32| -> Identity<i32> {
            let mut lg = linear_fn(ga, gb);
            Identity(lg(y))
        };

        let lhs = IdentityKind::bind(IdentityKind::bind(i.clone(), f), g);

        let f2 = f;
        let g2 = g;
        let rhs = IdentityKind::bind(i, move |x: i32| IdentityKind::bind(f2(x), g2));

        prop_assert_eq!(lhs, rhs);
    }
}
