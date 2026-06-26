# CLAUDE.md

Guidance for working in the `monadify` repository.

## What this is

`monadify` is a **zero-dependency Rust library** of functional-programming
abstractions — Functor, Apply, Applicative, Bind/Monad, Profunctor/Strong/Choice,
plus profunctor-encoded optics. v0.1.1, MIT, edition 2021, MSRV 1.66.

The defining design simulates **Higher-Kinded Types (HKTs)** using Generic
Associated Types: a `Kind` trait with `type Of<Arg>` plus lightweight marker
structs (`OptionKind`, `VecKind`, `ResultKind<E>`, `CFnKind<X>`, `CFnOnceKind<X>`,
`IdentityKind`, `ReaderTKind`). Traits are generic over the *marker*, and
`Self::Of<A>` resolves to the concrete type (e.g. `OptionKind::Of<i32> == Option<i32>`).
Core infrastructure lives in `src/kind_based/kind.rs`.

## Layout

- `src/kind_based/kind.rs` — the `Kind`/`Kind1` traits and all marker structs.
- `src/functor.rs`, `apply.rs`, `applicative.rs`, `monad.rs` — the trait hierarchy
  `Functor → Apply → Applicative → Bind/Monad`, defined over Kind markers.
- `src/profunctor.rs` — `Profunctor`/`Strong`/`Choice` and van-Laarhoven optics
  (`Lens`/`Getter`/`Fold`, `_1`/`_2`/`_key`, `view`, `lcmap`/`rmap`, `Forget`).
- `src/function.rs` — `CFn`/`CFnOnce` boxed-closure wrappers + composition (`>>`/`<<`).
- `src/identity.rs` — `Identity` monad. `src/transformers/reader.rs` — `ReaderT` +
  `MonadReader` (`ask`/`local`). `src/utils.rs` — `fn0!`..`fn3!` macros.
- `src/legacy/` — the older associated-type implementation, behind the `legacy`
  feature flag (kept for comparison/benchmarking; not the default).
- `tests/{kind,legacy}/` — law-verifying test suites. `benches/compare.rs` —
  criterion benchmarks comparing kind-based vs native vs legacy.

Concrete instances implementing the full hierarchy (where lawful): `Option`,
`Result`, `Vec`, `Identity`, `CFn`/`CFnOnce`, `ReaderT`.

## Conventions

- **Laws are first-class:** every trait documents *and* tests its laws (Functor
  identity/composition, Applicative, Monad left/right-identity + associativity,
  Profunctor). New instances must add law tests.
- Map/bind closures require `FnMut(A) -> B + Clone + 'static`.
- **Known constraint:** `CFn` is **not `Clone`**, which blocks some abstractions
  (e.g. `lift_a1` for `VecKind` is commented out in `applicative.rs`). Keep this
  in mind before adding `Clone`-dependent generic helpers.
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
