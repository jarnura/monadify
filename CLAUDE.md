# CLAUDE.md

Guidance for working in the `monadify` repository.

## What this is

`monadify` is a **zero-dependency Rust library** of functional-programming
abstractions — Functor, Apply, Applicative, Bind/Monad, Profunctor/Strong/Choice,
plus profunctor-encoded optics. v0.1.1, MIT, edition 2021, MSRV 1.66.

The defining design simulates **Higher-Kinded Types (HKTs)** using Generic
Associated Types: a `Kind` trait with `type Of<Arg>` plus lightweight marker
structs (`OptionKind`, `VecKind`, `ResultKind<E>`, `CFnKind<X>`, `CFnOnceKind<X>`,
`RcFnKind<X>`, `IdentityKind`, `ReaderTKind`, `StateTKind`, `WriterTKind`, `ExceptTKind`). Traits are generic over the *marker*, and
`Self::Of<A>` resolves to the concrete type (e.g. `OptionKind::Of<i32> == Option<i32>`).
Core infrastructure lives in `src/kind_based/kind.rs`.

## Layout

- `src/kind_based/kind.rs` — the `Kind`/`Kind1` traits and all marker structs.
- `src/functor.rs`, `apply.rs`, `applicative.rs`, `monad.rs` — the trait hierarchy
  `Functor → Apply → Applicative → Bind/Monad`, defined over Kind markers.
- `src/profunctor.rs` — `Profunctor`/`Strong`/`Choice` and van-Laarhoven optics
  (`Lens`/`Getter`/`Fold`, `_1`/`_2`/`_key`, `view`, `lcmap`/`rmap`, `Forget`).
- `src/function.rs` — `CFn`, `RcFn`, `CFnOnce` function wrappers + composition (`>>`/`<<`).
- `src/identity.rs` — `Identity` monad. `src/transformers/reader.rs` — `ReaderT` +
  `MonadReader` (`ask`/`local`). `src/transformers/state.rs` — `StateT` +
  `MonadState` (`state`/`get`/`put`/`modify`/`gets`) + runners
  `run`/`eval`/`exec_state_t`. `src/transformers/writer.rs` — `WriterT` +
  `MonadWriter` (`tell`/`writer`/`listen`/`censor`) + runner `exec_writer_t`.
  `src/transformers/except.rs` — `ExceptT` + `MonadError`
  (`throw_error`/`catch_error`/`lift_either`) + error-channel map `with_except_t`.
  `ExceptT` also provides inherent `ok`/`throw`/`from_result`/`catch` as the
  ergonomic concrete forms of the `MonadError` surface (the trait stays for generic
  code).
  `src/transformers/trans.rs` — `MonadTrans` (`lift`, impl'd for all four
  transformers). `src/monoid.rs` — `Semigroup`/`Monoid` (`combine`/`empty`).
- **Ergonomic inherent forms (all transformers).** To avoid the verbose
  `<Marker as MonadX<…>>::method()` UFCS at concrete call sites, each transformer
  exposes inherent forms of its `MonadX` surface that delegate to the trait with
  identical bounds (the trait stays for code generic over the inner monad):
  constructors whose value type is fixed by the result live on the **Kind marker**
  (`ReaderTKind::ask`; `StateTKind::{state,get,put,modify,gets}`;
  `WriterTKind::{tell,writer}`; plus `ExceptT::{ok,throw,from_result}` on the type),
  while computation-consuming ops are chainable **methods**
  (`ReaderT::local`, `WriterT::{listen,censor}`, `ExceptT::{catch,with_except_t}`).
  `src/utils.rs` — `fn0!`..`fn3!` macros.
- `src/legacy/` — the older associated-type implementation, behind the `legacy`
  feature flag (kept for comparison/benchmarking; not the default).
- `tests/{kind,legacy}/` — law-verifying test suites. `benches/compare.rs` —
  criterion benchmarks comparing kind-based vs native vs legacy.

Concrete instances implementing the full hierarchy (where lawful): `Option`,
`Result`, `Vec`, `Identity`, `RcFn`, `CFnOnce`, `ReaderT`, `StateT`, `WriterT`,
`ExceptT`. `StateT`'s
`Apply`/`Bind`/`Monad` require the **inner** monad to be `Bind`/`Monad` (state is
threaded — a sequential dependency), unlike `ReaderT` whose `Apply` needs only an
inner `Apply`; its four `MonadState` laws (get-get, get-put, put-get, put-put) are
first-class and law-tested. `WriterT` is in the **`ReaderT` family** (its `Apply`
needs only an inner `Apply`) but adds a `W: Monoid` constraint on the log: `pure`
seeds the log at `Monoid::empty`, every sequencing step `combine`s logs, and its
key law `tell(w1) >> tell(w2) ≡ tell(w1 <> w2)` is law-tested. `ExceptT` is a
**hybrid**: it has a `WriterT`-shaped *value* carrier (`run_except_t:
M::Of<Result<A, E>>`, manual `Clone` bounded on the projected inner type) but
sits in the **`StateT` bound-family** — its `apply`/`bind`/`join` require the
inner monad to be `Bind`/`Monad`, because short-circuiting on `Err` is a
sequential data dependency and an inner-`Apply`-only `apply` would break the
Applicative–Monad consistency law (matching Haskell's `instance (Monad m) =>
Applicative (ExceptT e m)`). It imposes the **lightest payload constraint** of
any transformer: `E: 'static` only — no `Monoid` (unlike `WriterT`'s `W`), no
`Clone` (unlike `StateT`'s `S`). Its `MonadError` surface is `throw_error`
(primitive), `catch_error`, and `lift_either`, with the catch laws (catch-throw,
catch-pure) and throw-left-zero law-tested. `ExceptT` also provides inherent
`ok`/`throw`/`from_result`/`catch` as the ergonomic concrete forms of the
`MonadError` surface (the trait stays for generic code). `MonadTrans::lift` embeds an inner
`M::Of<A>` into any of the four transformers, adding no effect (`ExceptT`'s
`lift` wraps the value in `Ok`).

## Conventions

- **Laws are first-class:** every trait documents *and* tests its laws (Functor
  identity/composition, Applicative, Monad left/right-identity + associativity,
  Profunctor). New instances must add law tests.
- Map/bind closures require `FnMut(A) -> B + Clone + 'static`.
- **Function wrappers:** `CFn` is the unique-ownership, non-`Clone` wrapper (Box-backed).
  `RcFn` is the shared-ownership, `Clone`-able alternative (Rc-backed); it re-enables
  `lift_a1::<VecKind>` and works in `mdo!`. `CFnOnce` is intentionally non-`Clone`
  (FnOnce semantics cannot be cloned). For `Clone`-dependent helpers, use `RcFn` or
  bound generically over types that are `Clone`.
- `#![deny(missing_docs)]` is enforced — all public items need docs.

## Quality gates

`scripts/pre-commit.sh` runs `cargo fmt --check`, `cargo clippy --all-features -- -D warnings`,
and `cargo test --all-features`. CI (`.github/workflows/rust.yml`) mirrors these in
separate jobs — `fmt`, `clippy` (`--all-features -- -D warnings`), `test` (default +
`legacy` feature matrix, incl. doc-tests), and an `msrv` job checking against Rust
1.66. The `clippy` job also lints `--benches` so `benches/compare.rs` can't
silently rot. Still **not** enforced in CI (remaining gaps): full
`clippy --all-targets` (the test files have ~50 auto-fixable lints, mostly
`clone_on_copy`) and `cargo doc -D warnings` (one rustdoc link warning).

```bash
cargo test                  # default kind-based suite
cargo test --features legacy # legacy suite
cargo bench                 # criterion benchmarks
```

## Memory (Hindsight)

> **MANDATORY (user directive, 2026-06-27):** The Hindsight bank `monadify` is the
> **sole memory of record**. At the **start of every session**, before other work,
> `recall(tags=["project:monadify"])`; at the **end of durable work**, `retain` new
> facts there. Use Hindsight for **all** long-term storage and retrieval — do **not**
> accumulate durable knowledge in the file-based auto-memory (`MEMORY.md`), which
> holds only a one-line breadcrumb pointing here. If the `hindsight` MCP server is
> unavailable (e.g. an unauthenticated headless/cron run), say so explicitly rather
> than silently falling back to file memory.

This project has a dedicated [Hindsight](https://github.com/vectorize-io/hindsight)
memory bank (`monadify`) holding the project's long-term context: git history
(3 epochs — 2022-genesis, 2023-typeclasses, 2025-hkt), architecture/conventions
seeded from this file, and the dependency list. It is exposed as an MCP server
named `hindsight`.

- **Config lives at local scope**, not in the repo — the server URL + bearer
  token are in `~/.claude.json` (project-scoped `mcpServers`), so the secret is
  never committed. There is intentionally no `.mcp.json` in the tree. To check:
  `claude mcp get hindsight`. The MCP tools load automatically at session start.
- **At the start of work, `recall`** relevant context, scoped by tag, e.g.
  `recall(query="…", tags=["project:monadify"])`. Useful tags: `source:git-log`
  (provenance), `source:CLAUDE.md` (architecture/conventions, `type:directive`),
  `source:Cargo.toml` (dependencies); epoch tags like `epoch:2025-hkt`.
- **At the end of durable work, `retain`** new facts with
  `tags=["project:monadify", …]` so the bank stays current. Memories tagged
  `status:uncommitted` describe in-progress WIP and go stale once committed —
  refresh them after committing.

## Babysitter

This project is onboarded for **babysitter** orchestration; the project profile
lives at `.a5c/project-profile.json` (written by `project-install`).

### Methodology

- **Primary — hypothesis-driven-development:** treat every typeclass law (Functor
  identity/composition, Applicative, Monad left/right-identity + associativity,
  Profunctor) as a falsifiable hypothesis and verify it with property-based tests
  (`proptest`) across generated inputs, layered on the existing example-based law
  tests. Use this for the top correctness/law-coverage goal.
- **Backbone — atdd-tdd:** write the law/property test first (RED), then implement
  the instance (GREEN), matching the project rule that an instance ships with its
  matching law test. Use for every new trait impl or concrete instance.
- **For big changes — evolutionary:** drive encoding/typeclass changes as small,
  reviewable, law-test-green increments instead of big-bang 26–50 file rewrites.
  Use when touching the high-churn core trait hierarchy or the HKT encoding.

### Recommended skills / agents

Must-have skills: **rust-testing** (proptest/property patterns + coverage),
**tdd** (write-test-first enforcement), **verify** (one-shot fmt + clippy
`--all-targets` + default/legacy test matrix + MSRV + doc build), **rust-patterns**
(idiomatic Rust/GAT/trait-bound guidance, incl. the `CFn`-not-`Clone` constraint),
**rust-review** (ownership/lifetime/coherence review of core trait edits).

Must-have agents: **tdd-guide** (drives the law-test-first loop), **test-generator**
(generates property/law tests for new and existing instances), **rust-reviewer**
(mandatory review of the high-churn core trait files), **rust-build-resolver**
(minimal-diff fixes for cargo/borrow-checker/feature-matrix breakage),
**architect** (designs around `CFn`-not-`Clone` and shapes new transformers
like StateT/WriterT before coding).

### Autonomy

**Semi-autonomous:** proceed on routine work (research, tests, refactors, docs),
but break for review at phase boundaries and before commits to `main`. ALWAYS
break on destructive-git operations and crates.io publish/release.

### CI/CD

Babysitter is kept **LOCAL / on-demand** — it is intentionally NOT wired into
GitHub Actions. The existing `.github/workflows/rust.yml` remains the automated
Rust quality gate (fmt, clippy, default+legacy test matrix, MSRV 1.66).

### How to run

- `/babysitter:babysit` — orchestrate a run (or use the babysitter CLI).
- `/babysitter:plan` — plan a run without executing it.
