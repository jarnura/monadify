//! Property-based (`proptest`) law tests for `WriterTKind` (the Writer transformer).
//!
//! Companion to the example-based tests in
//! `tests/kind/transformers/writer.rs`. The plain `Writer<W, A>` (inner
//! `IdentityKind`) has structural `Eq` once unwrapped, so laws compare the
//! produced `(value, log)` pair directly.
//!
//! Covers the three Monad laws plus the Writer-specific laws (`tell` appends
//! monoidally, `tell(empty) == pure(())`) over `WriterTKind<Vec<i32>,
//! IdentityKind>`, with Kleisli arrows materialized from generated `linear_fn`
//! slope/intercept parameters and generated log values.

use super::linear_fn;
use monadify::applicative::kind::Applicative;
use monadify::identity::{Identity, IdentityKind};
use monadify::monad::kind::Bind;
use monadify::monoid::{Monoid, Semigroup};
use monadify::transformers::writer::{MonadWriter, Writer, WriterTKind};
use proptest::prelude::*;

type W<A> = Writer<Vec<i32>, A>;
type WKind = WriterTKind<Vec<i32>, IdentityKind>;

fn run<A>(w: W<A>) -> (A, Vec<i32>) {
    let Identity(pair) = w.run_writer_t;
    pair
}

/// A logging Kleisli arrow: maps the value by a linear fn and appends `log`.
fn arrow(slope: i32, intercept: i32, log: Vec<i32>) -> impl Fn(i32) -> W<i32> + Clone {
    move |x: i32| {
        let mut lf = linear_fn(slope, intercept);
        let v = lf(x);
        <WKind as MonadWriter<Vec<i32>, i32, IdentityKind>>::writer(v, log.clone())
    }
}

fn arb_log() -> impl Strategy<Value = Vec<i32>> {
    prop::collection::vec(any::<i32>(), 0..=8)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    // --- Monad laws (logging) ---

    /// Left identity: `bind(pure(a), f) == f(a)`.
    #[test]
    fn writer_monad_left_identity(a in any::<i32>(),
                                  (sl, ic) in (any::<i32>(), any::<i32>()), log in arb_log()) {
        let f = arrow(sl, ic, log);
        let lhs = WKind::bind(WKind::pure(a), f.clone());
        let rhs = f(a);
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// Right identity: `bind(m, pure) == m`.
    #[test]
    fn writer_monad_right_identity(v in any::<i32>(), log in arb_log()) {
        let m = || <WKind as MonadWriter<Vec<i32>, i32, IdentityKind>>::writer(v, log.clone());
        let lhs = WKind::bind(m(), WKind::pure);
        prop_assert_eq!(run(lhs), run(m()));
    }

    /// Associativity: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
    #[test]
    fn writer_monad_associativity(v in any::<i32>(), m_log in arb_log(),
                                  (sl1, ic1) in (any::<i32>(), any::<i32>()), log1 in arb_log(),
                                  (sl2, ic2) in (any::<i32>(), any::<i32>()), log2 in arb_log()) {
        let f = arrow(sl1, ic1, log1);
        let g = arrow(sl2, ic2, log2);
        let m = || <WKind as MonadWriter<Vec<i32>, i32, IdentityKind>>::writer(v, m_log.clone());

        let lhs = WKind::bind(WKind::bind(m(), f.clone()), g.clone());
        let rhs = WKind::bind(m(), move |x| WKind::bind(f(x), g.clone()));
        prop_assert_eq!(run(lhs), run(rhs));
    }

    // --- Writer-specific laws ---

    /// `tell(w1) >> tell(w2) == tell(w1 <> w2)`.
    #[test]
    fn writer_tell_appends_monoidally(w1 in arb_log(), w2 in arb_log()) {
        let tell = |w: Vec<i32>| <WKind as MonadWriter<Vec<i32>, (), IdentityKind>>::tell(w);
        let lhs = WKind::bind(tell(w1.clone()), {
            let w2 = w2.clone();
            move |_| tell(w2.clone())
        });
        let rhs = tell(w1.combine(w2));
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// `tell(empty) == pure(())`.
    #[test]
    fn writer_tell_empty_is_pure_unit(_ignored in any::<bool>()) {
        let lhs = <WKind as MonadWriter<Vec<i32>, (), IdentityKind>>::tell(Vec::empty());
        let rhs: W<()> = WKind::pure(());
        prop_assert_eq!(run(lhs), run(rhs));
    }

    /// `censor` then run == apply the rewrite to the final log.
    #[test]
    fn writer_censor_rewrites_log(w in arb_log()) {
        let tell = <WKind as MonadWriter<Vec<i32>, (), IdentityKind>>::tell(w.clone());
        let censored = <WKind as MonadWriter<Vec<i32>, (), IdentityKind>>::censor(
            |mut log: Vec<i32>| { log.reverse(); log },
            tell,
        );
        let mut expected = w;
        expected.reverse();
        prop_assert_eq!(run(censored).1, expected);
    }
}
