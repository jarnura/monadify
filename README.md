# monadify: Functional Programming Constructs in Rust

`monadify` is a Rust library that provides implementations of common functional programming constructs, with a primary focus on monads and related concepts like Functors, Applicatives, and Profunctors. The goal is to offer a practical exploration of these patterns in idiomatic Rust, serving as both a learning resource and a potentially reusable library component.

## Core Concepts Implemented

The library defines and implements the following core functional programming traits:

*   **`Functor`**: Types that can be mapped over. Provides `map(self, f: A -> B) -> F<B>`.
    *   Implemented for `Option<A>`, `Result<A, E>`, `Vec<A>`, `CFn<X, A>`, `CFnOnce<X, A>`.
*   **`Apply`**: Extends `Functor`. Provides `apply(self, f: F<A -> B>) -> F<B>` for applying a wrapped function to a wrapped value.
    *   Implemented for `Option<A>`, `Result<A, E>`, `Vec<A>`.
*   **`Applicative`**: Extends `Apply`. Provides `pure(x: A) -> F<A>` for lifting a value into the applicative context.
    *   Implemented for `Option<A>`, `Result<A, E>`, `Vec<A>`.
*   **`Bind`**: Extends `Apply`. Provides `bind(self, f: A -> F<B>) -> F<B>` (also known as `flatMap` or `>>=`) for sequencing operations.
    *   Implemented for `Option<A>`, `Result<A, E>`, `Vec<A>`.
*   **`Monad`**: A marker trait that groups `Applicative` and `Bind`.
    *   Implemented for `Option<A>`, `Result<A, E>`, `Vec<A>`.
*   **`Profunctor`**: Bifunctors contravariant in the first argument and covariant in the second. Provides `dimap(self, f: X -> A, g: B -> Y) -> P<X, Y>`.
    *   Implemented for `CFn<A, B>` and `CFnOnce<A, B>`.
*   **`Strong`**: Extends `Profunctor`. Provides `first` and `second` for operating on product types (tuples).
    *   Implemented for `CFn<A, B>`.
*   **`Choice`**: Extends `Profunctor`. Provides `left` and `right` for operating on sum types (`Result`).
    *   Implemented for `CFn<A, B>`.

The library also includes function wrappers for heap-allocated closures:
- **`CFn<A, B>`**: A `Box`-backed, unique-ownership, non-`Clone` wrapper.
- **`RcFn<A, B>`**: An `Rc`-backed, shared-ownership, `Clone`-able sibling that unblocks `lift_a1::<VecKind>` and enables `mdo!` over function monads. Cloning is O(1).
- **`CFnOnce<A, B>`**: A `Box`-backed wrapper for once-callable closures (intentionally not `Clone`).

The library also includes various helper functions and macros (e.g., `lift2`, `lift_a1`, `fn0!`, `fn1!`, `_1`, `_2`, `view`) for working with these abstractions. Optical structures like `Lens` and `Getter` (using `Profunctor` encoding) are also explored.

## Project Goals
- To explore and understand monads and other functional patterns from a practical Rust implementation perspective.
- To create a reusable library of these structures in idiomatic Rust.
- To serve as an educational resource for learning about functional programming concepts in Rust.

## Usage Example

Here's a quick example of using the `Functor` trait with `Option` (Kind-based is now the default):

```rust
use monadify::{Functor, OptionKind}; // Import Kind-based Functor and marker

let some_value: Option<i32> = Some(10);
// For Kind-based, Functor<A,B> is on the marker OptionKind
let mapped_value = OptionKind::map(some_value, |x| x * 2);
assert_eq!(mapped_value, Some(20));

let no_value: Option<i32> = None;
let mapped_none = OptionKind::map(no_value, |x: i32| x * 2);
assert_eq!(mapped_none, None);
```

And an example using `Bind` (often called `flat_map`):

```rust
use monadify::{Bind, OptionKind}; // Import Kind-based Bind and marker

fn try_parse_and_double(s: &str) -> Option<i32> {
    s.parse::<i32>().ok().map(|n| n * 2)
}

let opt_str: Option<String> = Some("5".to_string());

// For Kind-based, Bind<A,B> is on the marker OptionKind
    // The closure takes String because OptionKind::Of<String> is Option<String>
    let result = OptionKind::bind(
        opt_str,
        |st: String| try_parse_and_double(&st) // Our function A -> F::Of<B>
    );
    assert_eq!(result, Some(10));

    let opt_invalid_str: Option<String> = Some("hello".to_string());
    let result_invalid = OptionKind::bind(
        opt_invalid_str,
        |st: String| try_parse_and_double(&st)
    );
    assert_eq!(result_invalid, None);
```

For more detailed examples, please refer to the documentation comments within the source code and the test files in the `tests/` directory.

## Do Notation

The library includes an optional **`do-notation`** feature that provides the `mdo!` macro, inspired by Haskell's `do` expressions. It lets you write monadic computations in a flat, imperative style instead of nested closures.

**Enable with**: `--features do-notation` (optional feature; zero-dependency by default)

**Syntax**: `mdo! { Marker; pat <- expr; ...; final_expr }`
- Marker must be explicit (e.g., `OptionKind`, `ResultKind::<E>`) — type inference is impossible
- Each `pat <- expr` is a monadic bind; `expr` is cloned once per bind step
- `guard(cond)` filters elements (Option/Vec only; short-circuits on failure)
- `let binding = expr;` introduces pure local bindings
- Final expression is returned raw (not auto-wrapped with `pure`)

**Quick example**:
```rust,ignore
use monadify::{mdo, OptionKind, Applicative};

let result: Option<i32> = mdo! {
    OptionKind;
    x <- Some(2);
    y <- Some(3);
    guard(x + y > 0);      // filters; short-circuits if false
    pure(x + y)            // bare `pure(...)` resolves to OptionKind::pure, == Some(5)
};
assert_eq!(result, Some(5));
```

**Real-world examples**: See `examples/` directory:
- `validation.rs` — Validation pipelines with short-circuit on first error (Option/Result)
- `reader_config.rs` — Environment threading (ReaderT + Config); "real power" demo
- `list_comprehension.rs` — List comprehensions with `guard` filtering (Vec)

Run with: `cargo run --example validation --features do-notation`

### State monad examples

These demonstrate the `StateT` State monad threading state through a computation
via `mdo!` do-notation — each `<-` bind both reads and updates the implicit state:

- `state_stack_machine.rs` — RPN / stack calculator (state = the operand stack)
- `state_unique_id.rs` — fresh-id / gensym generator (state = a counter)
- `state_rng_lcg.rs` — deterministic LCG pseudo-random generator (state = the seed)
- `state_bank_account.rs` — running-balance ledger (state = the balance)

Run any of them with (each requires the `do-notation` feature):
```bash
cargo run --example state_stack_machine --features do-notation
cargo run --example state_unique_id    --features do-notation
cargo run --example state_rng_lcg      --features do-notation
cargo run --example state_bank_account --features do-notation
```

**Limitations and notes**:
- `pure` is a reserved free-call head inside `mdo!` blocks (rewritten to `Marker::pure`); use `::pure`-qualified or `.pure()` method syntax to bypass
- `CFn` / `CFnOnce` unsupported (not `Clone`); use `RcFn` instead for a `Clone`-able, shared-ownership function monad
- At most one non-`Copy` external value per `mdo!` nesting level (closure capture constraint)

See [`monadify::mdo`](https://docs.rs/monadify) documentation for full details.

### Writer monad examples

These demonstrate the `WriterT` Writer monad accumulating a monoidal log through a
computation via `mdo!` do-notation — `tell` appends to the log while the result is
threaded as usual:

- `writer_audit_log.rs` — order/payment pipeline whose `Vec<String>` audit trail is the Writer log (`tell`)
- `writer_cost_monoid.rs` — running cost accumulated via a user-defined `Sum(u64)` monoid (your own `Semigroup`/`Monoid` impl)
- `writer_listen_censor.rs` — scoped, redacting logger using `listen` (capture a sub-computation's log) and `censor` (rewrite the log)
- `writer_eval_trace.rs` — arithmetic evaluator accumulating a `String` step-by-step trace

Run any of them with (each requires the `do-notation` feature):
```bash
cargo run --example writer_audit_log     --features do-notation
cargo run --example writer_cost_monoid   --features do-notation
cargo run --example writer_listen_censor --features do-notation
cargo run --example writer_eval_trace    --features do-notation
```

### Except monad examples

These demonstrate the `ExceptT` Except monad short-circuiting on the first error
via `mdo!` do-notation — a failed step aborts the computation, and `catch_error`
can recover:

- `except_form_validation.rs` — user-registration form validation that short-circuits on the first invalid field
- `except_safe_calculator.rs` — calculator that throws on divide-by-zero / domain errors and recovers via `catch_error`
- `except_config_loader.rs` — parse config string fields, unifying parse errors via `with_except_t`
- `except_order_pipeline.rs` — order pipeline (validate → reserve → charge) that aborts on the first failure

Run any of them with (each requires the `do-notation` feature):
```bash
cargo run --example except_form_validation --features do-notation
cargo run --example except_safe_calculator --features do-notation
cargo run --example except_config_loader   --features do-notation
cargo run --example except_order_pipeline   --features do-notation
```

## Building the Project

To build the library:
```bash
cargo build
```

## Running Tests

The library includes a comprehensive test suite to verify the laws of `Functor`, `Applicative`, `Monad`, etc.
To run the default Kind-based tests:
```bash
cargo test
```
This suite includes over 120 tests covering Kind-based implementations (for `Option`, `Result`, `Vec`, `Identity`, `CFn`, `CFnOnce`, `ReaderT`) and `Profunctor` laws.

To run tests for the legacy (non-HKT) implementations, use the `legacy` feature flag:
```bash
cargo test --features legacy
```
This suite includes over 80 tests for the legacy versions, also all passing.

## Running Benchmarks

Performance benchmarks for core operations are available using `criterion.rs`. To run the benchmarks:
```bash
cargo bench
```
The benchmark results can be found in `target/criterion/report/index.html`.
Key findings from initial benchmarks:
- `Functor::map` and `Bind::bind` for `Option` and `Result` show negligible overhead compared to native methods.
- `Apply::apply` (which involves `Box::new` for `CFn`) has a small, consistent overhead (around 2-4 ns).
- `Vec` operations show more overhead due to by-value semantics and heap allocations for `CFn` in some cases.

## License

This project is licensed under the terms of the [MIT License](./LICENSE).
