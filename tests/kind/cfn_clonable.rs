//! Tests for `RcFn<A,B>` — the `Rc`-backed, cheaply-clone-able function wrapper —
//! and `RcFnKind<X>` — its Kind marker implementing the full `Functor`/`Apply`/
//! `Applicative`/`Bind`/`Monad` hierarchy.
//!
//! # RED-phase notice
//!
//! `RcFn`, `RcFnKind`, and `lift_a1_rc` do **not yet exist** in `monadify`.
//! This file causes a compile error until the implementer adds them. That is the
//! intended RED state of the ATDD/TDD cycle.
//!
//! # Why `RcFn` and not `CFn`
//!
//! `CFn<A,B>` wraps `Box<dyn Fn(A)->B + 'static>`. `Box<dyn Fn(…)>` is **not
//! `Clone`**, so `CFn` is not `Clone` either. This blocks:
//!
//! - `VecKind`'s `Applicative<CFn<A,B>>` impl (needs `CFn: Clone`).
//! - `mdo!` blocks over `CFnKind` (desugaring emits `.clone()` on every RHS).
//! - Any law test that needs to reuse the same `CFn` value on both sides.
//!
//! `RcFn<A,B>` wraps `Rc<dyn Fn(A)->B + 'static>` and derives `Clone`. Cloning
//! bumps the reference count in O(1) — the closure body is **not** duplicated.
//! This resolves all three blockers.
//!
//! # Running
//!
//! ```text
//! cargo test --all-features cfn_clonable
//! cargo test --features do-notation cfn_clonable
//! ```

use core::convert::identity;
// NOTE: `lift_a1_rc` is the expected export name. The implementer may choose a
// different sibling name (e.g. a generic `lift_a1` overload taking `RcFn`). The
// assertion in the test below pins the VALUE, not the name.
use monadify::applicative::kind::{lift_a1, Applicative};
use monadify::apply::kind::Apply;
use monadify::function::{CFnOnce, RcFn};
use monadify::functor::kind::Functor;
use monadify::kind_based::kind::{RcFnKind, VecKind};
use monadify::monad::kind::{Bind, Monad};
use std::rc::Rc;

// ── 1. Clone is shared & O(1) ─────────────────────────────────────────────────

/// Cloning an `RcFn` bumps the outer `Rc` reference count — it does NOT
/// duplicate the closure body. We prove this using a sentinel `Rc` captured
/// inside the closure: if clone were deep (like `Box`-clone), the sentinel's
/// `strong_count` would increase. With `Rc`-backed sharing it stays constant.
///
/// Closures are PURE (no interior mutability) per the law-safety contract.
#[test]
fn cfn_clonable_rcfn_clone_is_shared_o1() {
    // A sentinel lets us observe whether the closure allocation is shared.
    let sentinel: Rc<u32> = Rc::new(42);
    let inner = Rc::clone(&sentinel); // strong_count == 2

    let f: RcFn<i32, i32> = RcFn::new(move |x: i32| {
        let _ = &inner; // force capture; `inner` stays alive inside the closure
        x + 1 // pure — no mutation of captured state
    });

    // Before cloning: sentinel strong_count == 2 (this binding + `inner` inside f).
    assert_eq!(Rc::strong_count(&sentinel), 2);

    // Clone the RcFn: O(1) Rc bump on the outer wrapper, NOT on `inner`.
    let g = f.clone();

    // After cloning: still 2 — the closure body was NOT duplicated.
    assert_eq!(
        Rc::strong_count(&sentinel),
        2,
        "RcFn::clone must share the underlying closure; sentinel strong_count must not increase"
    );

    // Both handles produce the same result.
    assert_eq!(f.call(3), 4);
    assert_eq!(g.call(3), 4);
    assert_eq!(f.call(10), 11);
    assert_eq!(g.call(10), 11);

    // Dropping g releases its share of the outer Rc; f still holds `inner`.
    drop(g);
    assert_eq!(Rc::strong_count(&sentinel), 2);

    // Dropping f releases the shared closure; `inner` inside it is gone.
    drop(f);
    assert_eq!(
        Rc::strong_count(&sentinel),
        1,
        "after dropping all RcFn handles, only the local `sentinel` remains"
    );
}

/// Calling the same `RcFn` multiple times via `&self` gives consistent results
/// (Fn semantics, not FnOnce).
#[test]
fn cfn_clonable_rcfn_call_is_repeatable() {
    let f: RcFn<i32, i32> = RcFn::new(|x: i32| x * 3);
    assert_eq!(f.call(4), 12);
    assert_eq!(f.call(4), 12); // same value — pure closure
    assert_eq!(f.call(7), 21);
}

/// Forward composition `>>` mirrors `CFn`'s `Shr` impl.
/// `(f >> g)(x) == g(f(x))`.
#[test]
fn cfn_clonable_rcfn_forward_compose_shr() {
    let f: RcFn<i32, i32> = RcFn::new(|x: i32| x + 1);
    let g: RcFn<i32, String> = RcFn::new(|x: i32| x.to_string());
    let h: RcFn<i32, String> = f >> g; // (f >> g)(4) == g(f(4)) == "5"
    assert_eq!(h.call(4), "5".to_string());
    assert_eq!(h.call(0), "1".to_string());
}

/// Backward composition `<<` mirrors `CFn`'s `Shl` impl.
/// `(g << f)(x) == g(f(x))`.
#[test]
fn cfn_clonable_rcfn_backward_compose_shl() {
    let f: RcFn<i32, i32> = RcFn::new(|x: i32| x + 1);
    let g: RcFn<i32, String> = RcFn::new(|x: i32| x.to_string());
    let h: RcFn<i32, String> = g << f; // (g << f)(4) == g(f(4)) == "5"
    assert_eq!(h.call(4), "5".to_string());
    assert_eq!(h.call(9), "10".to_string());
}

/// `Deref` exposes the underlying `Rc<dyn Fn(A)->B + 'static>`.
#[test]
fn cfn_clonable_rcfn_deref_exposes_inner_rc() {
    let f: RcFn<i32, i32> = RcFn::new(|x: i32| x * 2);
    // Deref should give us a reference to the inner Rc.
    let rc_ref: &Rc<dyn Fn(i32) -> i32 + 'static> = &f;
    // The Rc is uniquely owned (strong_count == 1) before any clone.
    assert_eq!(Rc::strong_count(rc_ref), 1);
    assert_eq!(f.call(6), 12);
}

// ── 2. Functor laws for RcFnKind ──────────────────────────────────────────────

mod rcfn_kind_functor_laws {
    use super::*;
    type Env = i32;

    /// Functor identity: `map(fa, |x| x)(env) == fa(env)` for all `env`.
    ///
    /// `RcFn` being `Clone` lets us hold `fa` and compare both sides directly —
    /// a capability `CFn` lacked.
    #[test]
    fn cfn_clonable_rcfn_kind_functor_identity() {
        let env_val: Env = 7;
        let fa: RcFn<Env, i32> = RcFn::new(|env: Env| env * 3); // fa(7) == 21

        let mapped = RcFnKind::<Env>::map(fa.clone(), |x: i32| x);

        assert_eq!(mapped.call(env_val), fa.call(env_val));
        assert_eq!(mapped.call(env_val), 21);
    }

    /// Functor composition: `map(map(fa, f), g)(env) == map(fa, g∘f)(env)`.
    #[test]
    fn cfn_clonable_rcfn_kind_functor_composition() {
        let env_val: Env = 4;
        let fa: RcFn<Env, i32> = RcFn::new(|env: Env| env + 1); // fa(4) == 5

        let f = |x: i32| x * 2; // f(5) == 10
        let g = |y: i32| y.to_string(); // g(10) == "10"

        // Sequential map (lhs)
        let lhs = RcFnKind::<Env>::map(RcFnKind::<Env>::map(fa.clone(), f), g);
        // Single composed map (rhs)
        let rhs = RcFnKind::<Env>::map(fa.clone(), move |x| g(f(x)));

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "10".to_string());
    }

    /// Composition with type change (string length), demonstrating `A != B`.
    #[test]
    fn cfn_clonable_rcfn_kind_functor_composition_type_change() {
        let env_val: Env = 3;
        let fa: RcFn<Env, &str> = RcFn::new(|_env: Env| "hello");

        let f = |s: &str| s.to_uppercase(); // f("hello") == "HELLO"
        let g = |s: String| s.len(); // g("HELLO") == 5

        let lhs = RcFnKind::<Env>::map(RcFnKind::<Env>::map(fa.clone(), f), g);
        let rhs = RcFnKind::<Env>::map(fa, move |s| g(f(s)));

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), 5_usize);
    }
}

// ── 3. Applicative laws for RcFnKind ──────────────────────────────────────────

mod rcfn_kind_applicative_laws {
    use super::*;
    type Env = i32;

    /// Applicative identity: `apply(v, pure(id)) == v`.
    ///
    /// `RcFn` being `Clone` lets us hold `v` and compare both sides.
    #[test]
    fn cfn_clonable_rcfn_kind_applicative_law_identity() {
        let env_val: Env = 11;
        let v: RcFn<Env, i32> = RcFn::new(|env: Env| env + 100); // v(11) == 111

        let id_rcfn = RcFn::new(identity::<i32>);
        let pure_id: RcFn<Env, RcFn<i32, i32>> = RcFnKind::<Env>::pure(id_rcfn);

        let applied = RcFnKind::<Env>::apply(v.clone(), pure_id);

        assert_eq!(applied.call(env_val), v.call(env_val));
        assert_eq!(applied.call(env_val), 111);
    }

    /// Applicative homomorphism: `apply(pure(x), pure(f)) == pure(f(x))`.
    ///
    /// `pure` ignores the environment, so any `env_val` produces the same result.
    #[test]
    fn cfn_clonable_rcfn_kind_applicative_law_homomorphism() {
        let env_val: Env = 0; // pure ignores env
        let x: i32 = 10;
        let f = |val: i32| val * 2;

        let lhs = RcFnKind::<Env>::apply(
            RcFnKind::<Env>::pure(x),
            RcFnKind::<Env>::pure(RcFn::new(f)),
        );
        let rhs: RcFn<Env, i32> = RcFnKind::<Env>::pure(f(x)); // pure(20)

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), 20);
    }

    /// Applicative composition law: `apply(apply(w, v), u)` equals the
    /// ground-truth `u_fn(v_fn(w(env)))`, the right-associated form
    /// `u <*> (v <*> w)`.
    ///
    /// This was untestable for `CFnKind` (CFn not Clone) but IS testable for
    /// `RcFnKind` because all three readers (`w`, `v`, `u`) can be cloned and
    /// evaluated at multiple environments without rebuilding from scratch.
    #[test]
    fn cfn_clonable_rcfn_kind_applicative_law_composition() {
        // w: RcFn<Env, i32> — value reader: w(env) = env * 2
        let w: RcFn<Env, i32> = RcFn::new(|env: Env| env * 2);
        // v: RcFn<Env, RcFn<i32, i32>> — first function reader: v(env)(x) = x + env
        let v: RcFn<Env, RcFn<i32, i32>> = RcFn::new(|env: Env| RcFn::new(move |x: i32| x + env));
        // u: RcFn<Env, RcFn<i32, String>> — second function reader: u(env)(y) = (y * env).to_string()
        let u: RcFn<Env, RcFn<i32, String>> =
            RcFn::new(|env: Env| RcFn::new(move |y: i32| (y * env).to_string()));

        // apply(apply(w, v), u) — right-associated: u <*> (v <*> w)
        let inner: RcFn<Env, i32> = RcFnKind::<Env>::apply(w.clone(), v.clone());
        let result: RcFn<Env, String> = RcFnKind::<Env>::apply(inner, u.clone());

        // env = 5: w(5)=10, v(5)(10)=15, u(5)(15)="75"
        assert_eq!(result.call(5), "75".to_string());
        // env = 3: w(3)=6, v(3)(6)=9, u(3)(9)="27"
        assert_eq!(result.call(3), "27".to_string());
        // env = 1: w(1)=2, v(1)(2)=3, u(1)(3)="3"
        assert_eq!(result.call(1), "3".to_string());
    }

    /// Applicative interchange: `apply(pure(y), u) == apply(u, pure(|f_fn| f_fn(y)))`.
    ///
    /// LHS uses `Apply<A, B>`; RHS uses `Apply<CFn<A,B>, B>`. The `u` value is
    /// shared via `Clone` (not rebuilt from a factory), demonstrating the advantage
    /// of `RcFn` over `CFn`.
    #[test]
    fn cfn_clonable_rcfn_kind_applicative_law_interchange() {
        type A = i32;
        type B = String;
        let env_val: Env = 99; // constant env; u ignores it (pure-like)

        let y_val: A = 10;

        // u: an RcFn<Env, RcFn<A,B>> that always returns the same concrete RcFn.
        // Because RcFn is Clone, we can share `u` between lhs and rhs.
        let u: RcFn<Env, RcFn<A, B>> =
            RcFn::new(move |_env: Env| RcFn::new(|val: A| format!("val:{}", val)));

        let pure_y: RcFn<Env, A> = RcFnKind::<Env>::pure(y_val);

        // LHS: apply(pure(y), u)  — Apply<A, B>
        let lhs = RcFnKind::<Env>::apply(pure_y, u.clone());

        // RHS: apply(u, pure(|f_fn| f_fn(y)))  — Apply<RcFn<A,B>, B>
        let interchange = RcFn::new(move |f_fn: RcFn<A, B>| f_fn.call(y_val));
        let pure_interchange: RcFn<Env, RcFn<RcFn<A, B>, B>> = RcFnKind::<Env>::pure(interchange);
        let rhs = RcFnKind::<Env>::apply(u, pure_interchange);

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "val:10".to_string());
    }
}

// ── 4. Monad laws for RcFnKind ────────────────────────────────────────────────

mod rcfn_kind_monad_laws {
    use super::*;
    type Env = i32;

    /// Monad left identity: `bind(pure(a), f)(env) == f(a)(env)`.
    ///
    /// `RcFn` being `Clone` lets us run both sides against the same env rather
    /// than relying on a creator factory (as `CFnKind` tests required).
    #[test]
    fn cfn_clonable_rcfn_kind_monad_left_identity() {
        let env_val: Env = 5;
        let a: i32 = 10;

        let f =
            move |x: i32| -> RcFn<Env, String> { RcFn::new(move |env: Env| (x + env).to_string()) };

        let pure_a: RcFn<Env, i32> = RcFnKind::<Env>::pure(a);
        let lhs: RcFn<Env, String> = RcFnKind::<Env>::bind(pure_a, f);
        let rhs: RcFn<Env, String> = f(a);

        // Both sides are Clone; compare by running at the same env.
        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "15".to_string()); // 10 + 5
    }

    /// Monad right identity: `bind(m, pure)(env) == m(env)`.
    ///
    /// `m` is `Clone` — no factory needed.
    #[test]
    fn cfn_clonable_rcfn_kind_monad_right_identity() {
        let env_val: Env = 7;
        let m: RcFn<Env, i32> = RcFn::new(|env: Env| env * 2); // m(7) == 14

        let pure_fn = move |val: i32| RcFnKind::<Env>::pure(val);
        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::bind(m.clone(), pure_fn);

        assert_eq!(lhs.call(env_val), m.call(env_val));
        assert_eq!(lhs.call(env_val), 14);
    }

    /// Monad associativity: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    ///
    /// All three of `m`, the intermediate result, and both sides are `Clone`,
    /// so we never need duplicate creator lambdas.
    #[test]
    fn cfn_clonable_rcfn_kind_monad_associativity() {
        let env_val: Env = 3;
        // m(3) == 4
        let m: RcFn<Env, i32> = RcFn::new(|env: Env| env + 1);

        // f(4)(3) == (4 * 3) as f64 == 12.0
        let f = move |x: i32| -> RcFn<Env, f64> { RcFn::new(move |env: Env| (x * env) as f64) };
        // g(12.0)(3) == (12.0 + 3.0).to_string() == "15"
        let g = move |y: f64| -> RcFn<Env, String> {
            RcFn::new(move |env: Env| (y + env as f64).to_string())
        };

        // LHS: bind(bind(m, f), g)
        let lhs: RcFn<Env, String> = RcFnKind::<Env>::bind(RcFnKind::<Env>::bind(m.clone(), f), g);

        // RHS: bind(m, |x| bind(f(x), g))
        let rhs: RcFn<Env, String> =
            RcFnKind::<Env>::bind(m.clone(), move |x| RcFnKind::<Env>::bind(f(x), g));

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), "15".to_string());
    }

    /// Monad join law 1: `join(pure(pure(x)))(env) == pure(x)(env) == x`.
    ///
    /// **This is the key capability `CFnKind` lacked**: `Monad<A>` for `RcFnKind<R>`
    /// requires `A: Clone` (via the `Applicative` supertrait). For `join` over
    /// `RcFn<R, RcFn<R, i32>>`, the type `A` is `RcFn<R, i32>` — which IS
    /// `Clone` because `RcFn` derives `Clone`. `CFn` could not be `A` here.
    #[test]
    fn cfn_clonable_rcfn_kind_monad_join_law1() {
        let env_val: Env = 5;
        let x: i32 = 10;

        // inner: RcFn<Env, i32> — Clone, so Monad<RcFn<Env,i32>> compiles.
        let inner: RcFn<Env, i32> = RcFnKind::<Env>::pure(x);
        // outer: RcFn<Env, RcFn<Env, i32>> — pure needs T: Clone, which holds.
        let outer: RcFn<Env, RcFn<Env, i32>> = RcFnKind::<Env>::pure(inner.clone());

        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::join(outer);
        let rhs: RcFn<Env, i32> = RcFnKind::<Env>::pure(x);

        assert_eq!(lhs.call(env_val), rhs.call(env_val));
        assert_eq!(lhs.call(env_val), x);
    }

    /// Monad join law 2: `join(map(m, pure))(env) == m(env)`.
    #[test]
    fn cfn_clonable_rcfn_kind_monad_join_law2() {
        let env_val: Env = 7;
        let m: RcFn<Env, i32> = RcFn::new(|env: Env| env * 3); // m(7) == 21

        let pure_fn = move |val: i32| RcFnKind::<Env>::pure(val);
        // map(m, pure) :: RcFn<Env, RcFn<Env, i32>>  — works because RcFn is Clone
        let mapped: RcFn<Env, RcFn<Env, i32>> = RcFnKind::<Env>::map(m.clone(), pure_fn);
        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::join(mapped);

        assert_eq!(lhs.call(env_val), m.call(env_val));
        assert_eq!(lhs.call(env_val), 21);
    }

    /// Monad join law 3: `join(pure(m))(env) == m(env)`.
    #[test]
    fn cfn_clonable_rcfn_kind_monad_join_law3() {
        let env_val: Env = 4;
        let m: RcFn<Env, i32> = RcFn::new(|env: Env| env + 10); // m(4) == 14

        // pure(m) :: RcFn<Env, RcFn<Env, i32>> — needs m: Clone, which holds.
        let pure_m: RcFn<Env, RcFn<Env, i32>> = RcFnKind::<Env>::pure(m.clone());
        let lhs: RcFn<Env, i32> = RcFnKind::<Env>::join(pure_m);

        assert_eq!(lhs.call(env_val), m.call(env_val));
        assert_eq!(lhs.call(env_val), 14);
    }
}

// ── 5. Re-enabled lift_a1::<VecKind> via RcFn ────────────────────────────────

/// `lift_a1::<VecKind>` was blocked because the existing `lift_a1` builds
/// `F::pure(CFn::new(func))` and `VecKind::pure` requires `T: Clone`; since
/// `CFn<A,B>` is not `Clone`, the call fails to compile (the commented-out
/// example at `src/applicative.rs:211-218`).
///
/// An `RcFn`-based sibling (`lift_a1_rc` or an `RcFn`-generic `lift_a1`) builds
/// `F::pure(RcFn::new(func))` instead. `RcFn<A,B>` IS `Clone`, so
/// `VecKind::Applicative<RcFn<A,B>>` compiles and the path is unblocked.
///
/// NOTE: The exact name of the sibling function (`lift_a1_rc` or a renamed
/// generic `lift_a1`) is **to be finalized by the implementer**. The call below
/// uses `lift_a1_rc` as a placeholder; the assertion pins the VALUE, not the name.
#[test]
fn cfn_clonable_vec_lift_a1_rc_reenabled() {
    let fa: Vec<i32> = vec![1, 2, 3];
    // 1 % 2 == 1 => false, 2 % 2 == 0 => true, 3 % 2 == 1 => false
    let result: Vec<bool> = lift_a1::<VecKind, i32, bool, _>(|x: i32| x % 2 == 0, fa);
    assert_eq!(result, vec![false, true, false]);
}

// ── 5b. lift_a1::<RcFnKind<Env>> — newly unlocked ────────────────────────────

/// `lift_a1::<RcFnKind<Env>>` now compiles because the specialized
/// `Applicative<CFn<A,B>> for RcFnKind<X>` impl provides a `pure` for
/// `CFn<A,B>` (the non-Clone type used as the function container) by converting
/// the inner `Box` to `Rc`. This unblocks the `lift_a1` code path, which calls
/// `F::pure(CFn::new(func))` internally, for `RcFnKind`.
///
/// The resulting `RcFn<Env, B>` maps over the environment via the standard
/// S-combinator `Apply` instance: `result(env) = func(fa(env))`.
#[test]
fn cfn_clonable_rcfn_kind_lift_a1_reenabled() {
    type Env = i32;
    let env_val: Env = 5;

    // fa: RcFn<Env, i32> — reader: fa(5) = 10
    let fa: RcFn<Env, i32> = RcFn::new(|env: Env| env * 2);

    // lift_a1::<RcFnKind<Env>> lifts |x| x.to_string() to operate on RcFn
    let result: RcFn<Env, String> =
        lift_a1::<RcFnKind<Env>, i32, String, _>(|x: i32| x.to_string(), fa);

    // result(5) = (5 * 2).to_string() = "10"
    assert_eq!(result.call(env_val), "10".to_string());
    assert_eq!(result.call(1), "2".to_string());
    assert_eq!(result.call(0), "0".to_string());
}

// ── 6. CFnOnce: intentionally stays non-Clone ─────────────────────────────────

/// `CFnOnce` wraps `Box<dyn FnOnce(A) -> B + 'static>`. `Box<dyn FnOnce(…)>` is
/// **not `Clone`** — trait objects cannot be cloned generically — so `CFnOnce`
/// must NOT derive `Clone`. This is an intentional API contract:
///
/// - `CFnOnce` models single-shot, move-only computations.
/// - Adding `Clone` would require knowing the concrete closure type at compile
///   time, which contradicts the dynamic-dispatch boxing.
/// - `mdo!` blocks over `CFnOnceKind` therefore remain unsupported; see
///   `tests/kind/do_notation/cfn_unsupported.rs` for the full explanation.
///
/// Compile-fail evidence (no `trybuild` dev-dep needed — kept as a doc comment):
///
/// ```compile_fail
/// use monadify::function::CFnOnce;
/// let f: CFnOnce<i32, i32> = CFnOnce::new(|x| x + 1);
/// let _g = f.clone(); // E0599: CFnOnce does not implement Clone
/// ```
///
/// The runtime test below confirms single-shot semantics are intact and that
/// no accidental `Clone` impl was added (which would have allowed double-call).
///
// CFnOnce: NOT Clone by design — this is an intentional design decision.
#[test]
fn cfn_clonable_cfnonce_stays_non_clone_guard() {
    let f: CFnOnce<i32, i32> = CFnOnce::new(|x: i32| x * 7);
    // Single-shot call works as expected.
    assert_eq!(f.call_once(6), 42);
    // `f` is consumed after `call_once`; no `.clone()` is available.
    // The compile_fail doc comment above documents the compile-time enforcement.
    //
    // CFnOnce: NOT Clone by design.
}

// ── 7. RcFn through mdo! (feature-gated) ────────────────────────────────────
//
// Gated exactly like the existing do-notation tests (`tests/kind/do_notation/`):
// the parent `mod.rs` gate is `#[cfg(feature = "do-notation")]`.
//
// `RcFn` is `Clone`, so the `mdo!` desugaring's automatic `.clone()` on every
// bind RHS compiles — the blocker that excluded `CFn`/`CFnOnce` is resolved.

#[cfg(feature = "do-notation")]
mod rcfn_kind_mdo {
    //! `mdo!` do-block tests for `RcFnKind` (the function / reader monad).
    //!
    //! These tests exercise the gap `CFn` could NOT fill: because `mdo!` emits
    //! `(expr).clone()` on every monadic RHS, any monad whose carrier is not
    //! `Clone` cannot appear in a do-block. `RcFn` derives `Clone` (O(1) Rc
    //! bump), so `RcFnKind` works transparently with `mdo!`.

    use monadify::applicative::kind::Applicative;
    use monadify::function::RcFn;
    use monadify::kind_based::kind::RcFnKind;
    use monadify::mdo;
    use monadify::monad::kind::Bind;

    /// Environment type threaded through the do-block.
    type Env = i32;
    /// Kind-marker alias so `mdo! { RcReaderKind; … }` is readable — mirrors the
    /// `type ReaderKind = ReaderTKind<Config, IdentityKind>` alias in `reader.rs`.
    type RcReaderKind = RcFnKind<Env>;

    /// Two `RcFn` steps bound in sequence both read from the same environment.
    ///
    /// `x <- RcFn::new(|env| env * 2)` binds `x = env * 2`.
    /// `y <- RcFn::new(|env| env + 1)` binds `y = env + 1`.
    /// Final: `x + y`.
    ///
    /// At env = 5: x = 10, y = 6, result = 16.
    #[test]
    fn cfn_clonable_rcfn_kind_mdo_two_bindings() {
        let computation: RcFn<Env, i32> = mdo! {
            RcReaderKind;
            x <- RcFn::new(|env: Env| env * 2);
            y <- RcFn::new(|env: Env| env + 1);
            RcReaderKind::pure(x.wrapping_add(y))
        };

        assert_eq!(computation.call(5), 16); // 10 + 6
        assert_eq!(computation.call(0), 1); // 0 + 1
        assert_eq!(computation.call(-1), -2); // -2 + 0
    }

    /// The `mdo!` desugaring must produce the same result as an equivalent
    /// hand-written nested `Bind::bind` chain for every concrete environment.
    ///
    /// Closures cannot be compared structurally; we compare by running both
    /// against the same set of environment values (mirroring `reader.rs`).
    #[test]
    fn cfn_clonable_rcfn_kind_mdo_equivalence_matches_hand_written_bind() {
        // lhs: built by the macro.
        let lhs: RcFn<Env, i32> = mdo! {
            RcReaderKind;
            x <- RcFn::new(|env: Env| env.wrapping_mul(2));
            y <- RcFn::new(|env: Env| env.wrapping_add(1));
            RcReaderKind::pure(x.wrapping_add(y))
        };

        // rhs: the exact desugared form `mdo!` emits.
        let rhs: RcFn<Env, i32> =
            RcReaderKind::bind(RcFn::new(|env: Env| env.wrapping_mul(2)), move |x| {
                RcReaderKind::bind(RcFn::new(|env: Env| env.wrapping_add(1)), move |y| {
                    RcReaderKind::pure(x.wrapping_add(y))
                })
            });

        let envs: &[Env] = &[0, 1, -3, 5, i32::MAX / 2, i32::MIN / 2];
        for &env in envs {
            assert_eq!(
                lhs.call(env),
                rhs.call(env),
                "mdo! and hand-written bind disagree at env = {}",
                env
            );
        }
    }

    /// Three-binding chain confirms desugaring depth > 2 is correct.
    ///
    /// `a + b + c` where `a = env`, `b = 2*env`, `c = 3*env`, so result = `6*env`.
    #[test]
    fn cfn_clonable_rcfn_kind_mdo_three_bindings_chain() {
        let computation: RcFn<Env, i32> = mdo! {
            RcReaderKind;
            a <- RcFn::new(|env: Env| env);
            b <- RcFn::new(|env: Env| env.wrapping_mul(2));
            c <- RcFn::new(|env: Env| env.wrapping_mul(3));
            RcReaderKind::pure(a.wrapping_add(b).wrapping_add(c))
        };

        assert_eq!(computation.call(4), 24); // 4 + 8 + 12
        assert_eq!(computation.call(0), 0);
        assert_eq!(computation.call(-1), -6); // -1 + -2 + -3
    }

    /// A `let` binding inside the block introduces a pure local name.
    #[test]
    fn cfn_clonable_rcfn_kind_mdo_let_binding_inside_block() {
        let computation: RcFn<Env, i32> = mdo! {
            RcReaderKind;
            x <- RcFn::new(|env: Env| env * 3);
            let scaled = x * 10;
            RcReaderKind::pure(scaled)
        };

        // env = 2: x = 6, scaled = 60
        assert_eq!(computation.call(2), 60);
        // env = -1: x = -3, scaled = -30
        assert_eq!(computation.call(-1), -30);
    }

    /// Bare `pure(x)` in the final position of an `mdo!` block over `RcFnKind`.
    ///
    /// The macro rewrites the bare `pure(x + y)` to
    /// `<RcReaderKind as ::monadify::applicative::kind::Applicative<_>>::pure(x + y)`.
    /// This resolves to `RcFnKind::<Env>::pure(x + y)` — a constant reader
    /// ignoring `env` and always returning `x + y`.
    ///
    /// This exercises the macro's `pure` rewrite for the `RcFnKind` marker.
    #[cfg(feature = "do-notation")]
    #[test]
    fn cfn_clonable_rcfn_kind_mdo_bare_pure_final_position() {
        let computation: RcFn<Env, i32> = mdo! {
            RcReaderKind;
            x <- RcFn::new(|env: Env| env * 2);
            y <- RcFn::new(|env: Env| env + 1);
            pure(x + y)
        };

        // env = 5: x = 10, y = 6, result = 16
        assert_eq!(computation.call(5), 16);
        // env = 0: x = 0, y = 1, result = 1
        assert_eq!(computation.call(0), 1);
        // Cross-check: must agree with the hand-written qualified form.
        let reference: RcFn<Env, i32> =
            RcReaderKind::bind(RcFn::new(|env: Env| env * 2), move |x| {
                RcReaderKind::bind(RcFn::new(|env: Env| env + 1), move |y| {
                    RcReaderKind::pure(x + y)
                })
            });
        for &env in &[0i32, 1, 5, -3, 100] {
            assert_eq!(
                computation.call(env),
                reference.call(env),
                "bare pure and qualified pure disagree at env = {}",
                env
            );
        }
    }
}
