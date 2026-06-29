# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-06-29

This release builds out a full **monad-transformer stack** on top of the
kind-based HKT encoding, plus a crate-wide ergonomics and idiomatic-Rust pass.

### Added

- **`StateT` monad transformer** with `MonadState` (`state`/`get`/`put`/`modify`/`gets`)
  and the runners `run`/`eval`/`exec_state_t`. Its four `MonadState` laws
  (get-get, get-put, put-get, put-put) are law-tested. `Apply`/`Bind`/`Monad`
  require the inner monad to be `Bind`/`Monad` (state threading is a sequential
  dependency).
- **`WriterT` monad transformer** with `MonadWriter` (`tell`/`writer`/`listen`/`censor`)
  and the runner `exec_writer_t`, plus the **`Semigroup`/`Monoid`** typeclasses
  (`combine`/`empty`) for the log. In the `ReaderT` bound-family (its `Apply` needs
  only an inner `Apply`) but adds a `W: Monoid` constraint; the key law
  `tell(w1) >> tell(w2) ≡ tell(w1 <> w2)` is law-tested.
- **`ExceptT` monad transformer** with `MonadError` (`throw_error`/`catch_error`/`lift_either`)
  and the error-channel map `with_except_t`. A hybrid: a `WriterT`-shaped value
  carrier (`M::Of<Result<A, E>>`) sitting in the `StateT` bound-family (short-circuit
  on `Err` is a sequential dependency). Lightest payload constraint of any
  transformer — `E: 'static` only. The catch laws (catch-throw, catch-pure) and
  throw-left-zero are law-tested.
- **`MonadTrans::lift`** embedding an inner `M::Of<A>` into any of the four
  transformers (`ExceptT`'s `lift` wraps in `Ok`).
- **Inherent ergonomic forms** for every transformer's `MonadX` surface, so
  concrete call sites avoid the verbose `<Marker as MonadX<…>>::method()` UFCS:
  constructors on the Kind marker (`ReaderTKind::ask`;
  `StateTKind::{state,get,put,modify,gets}`; `WriterTKind::{tell,writer}`;
  `ExceptT::{ok,throw,from_result}`) and chainable methods (`ReaderT::local`,
  `WriterT::{listen,censor}`, `ExceptT::{catch,with_except_t}`, `XKind::lift`).
  The `MonadX` traits remain for code generic over the inner monad.
- **Transformer stacking**: all four Kind marker structs now impl unconditional
  `Clone` (they are zero-sized `PhantomData`), which is the prerequisite for
  nesting one transformer's inner Kind as the next's marker.
- **Real-world runnable examples** for `StateT`, `WriterT`, and `ExceptT`, plus
  `examples/interpreter_full_stack.rs` — a tiny expression interpreter over the
  full `ReaderT`/`StateT`/`WriterT`/`ExceptT` stack. Examples that use `mdo!` are
  gated behind `required-features = ["do-notation"]`.

### Changed

- Examples now use the `mdo!` do-notation macro instead of explicit `bind` chains.
- Crate-wide idiomatic-Rust sweep: `#[must_use]` annotations, `PhantomData`
  collapse, `Option::flatten`, `const fn` where applicable, and doctest tidying.
- `cargo doc --no-deps --all-features` and `clippy --all-targets --all-features`
  are now clean across the whole tree.

## [0.2.0]

- Removed the `CFn`/`CFnKind` and `CFnOnce` multi-call function markers from the
  default surface in favour of `RcFn` (shared-ownership, `Clone`-able) for
  `Clone`-dependent helpers. Introduced the kind-based HKT encoding as the default.

## [0.1.1]

- `mdo!` do-notation via the `monadify-macros` proc-macro crate (`do-notation` feature).

## [0.1.0]

- Initial release: Functor / Apply / Applicative / Bind / Monad hierarchy,
  Profunctor / Strong / Choice, profunctor-encoded optics, and function wrappers.

[0.3.0]: https://github.com/jarnura/monadify/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jarnura/monadify/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jarnura/monadify/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jarnura/monadify/releases/tag/v0.1.0
