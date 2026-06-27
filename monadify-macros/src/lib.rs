//! Procedural macro backing `monadify`'s `do`-notation.
//!
//! This crate exports a single procedural macro, [`mdo`], which is re-exported
//! by `monadify` under the non-default `do-notation` feature as `monadify::mdo`.
//!
//! # What it does
//!
//! `mdo!` desugars an imperative monadic block — a sequence of `let`,
//! `pat <- expr;`, `guard(expr);`, and bare `expr;` statements followed by a
//! trailing *final expression* — into nested `Marker::bind(..)` calls over
//! `monadify`'s [`Bind`](../monadify/trait.Bind.html) /
//! [`Applicative`](../monadify/trait.Applicative.html) trait hierarchy.
//!
//! The block names its Kind marker explicitly (marker inference is impossible
//! because the GAT `Of<Arg>` is not injective):
//!
//! ```text
//! use monadify::{mdo, OptionKind, Applicative};
//!
//! let r: Option<i32> = mdo! { OptionKind;
//!     x <- Some(2);
//!     y <- Some(3);
//!     guard(x + y > 0);
//!     OptionKind::pure(x + y)      // raw monadic value, == Some(5)
//! };
//! ```
//!
//! A runnable version of this example lives on `monadify::mdo` (the gated
//! re-export), where it can depend on `monadify` without a build cycle.

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Spacing, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Error, Expr, Pat, Result as SynResult, Token, Type};

/// A single desugarable statement inside an `mdo!` block.
enum Stmt {
    /// `pat <- expr` — monadic bind.
    Bind(Pat, Expr),
    /// `let …` — a plain Rust `let`, kept verbatim (tokens exclude the `;`).
    Let(TokenStream2),
    /// `guard(cond)` — filter via the `MdoGuard` helper trait.
    Guard(TokenStream2),
    /// bare `expr` — sequencing (`_ <- expr`).
    Bare(Expr),
}

/// Parsed `mdo!` block: explicit marker, statement list, and the raw final expr.
struct MdoInput {
    marker: Type,
    stmts: Vec<Stmt>,
    final_expr: Expr,
}

/// Returns `true` if `tt` is a `:` punctuation token.
fn is_colon(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ':')
}

/// Returns `true` if `tt` is a `:` punctuation token with `Spacing::Joint`
/// (i.e., the first `:` of a `::` digraph).
fn is_colon_joint(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ':' && p.spacing() == Spacing::Joint)
}

/// Returns `true` if `tt` is a `.` punctuation token.
fn is_dot(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == '.')
}

/// Rewrites every bare `pure(...)` call-head in `ts` to
/// `<marker as ::monadify::Applicative<_>>::pure`, leaving qualified forms
/// (`::pure`, `.pure`, or `pure` not followed by `(`) unchanged.
///
/// The walk is fully recursive: inner groups (parentheses, brackets, braces)
/// are descended into so that nested forms like `foo(pure(3))` and
/// `pure(pure(1))` are both handled correctly.
///
/// # Suppression rules (two-token look-behind)
///
/// The rewriter tracks the **last two emitted tokens** to distinguish the
/// three digraph pairs that share a single-character prefix with valid
/// separators:
///
/// - **`::pure` path qualifier** (suppress): the token immediately before
///   `pure` is `:`, AND the token before that is also `:` with
///   `Spacing::Joint`.  A lone `:` (struct-field colon) does NOT meet the
///   second condition and must NOT suppress — `Foo { f: pure(x) }` should
///   be rewritten.
///
/// - **`.pure` method call** (suppress): the token immediately before `pure`
///   is `.`, AND the token before that is NOT `.`.  A `..` (range / struct
///   update), where the preceding token IS `.`, must NOT suppress —
///   `..pure(base)` should be rewritten.
///
/// - **next token is NOT `(`** (suppress): `pure` is not a call expression
///   and is emitted unchanged.
fn rewrite_pure(ts: TokenStream2, marker: &Type) -> TokenStream2 {
    let mut out = TokenStream2::new();
    // Peekable so we can inspect the token after `pure` without consuming it.
    let mut iter = ts.into_iter().peekable();

    // Two-token look-behind.
    // `prev_*`  describes the most recently emitted token.
    // `prev2_*` describes the token emitted before that.
    let mut prev_is_colon = false;
    let mut prev_colon_is_joint = false; // only meaningful when prev_is_colon
    let mut prev_is_dot = false;
    let mut prev2_is_colon_joint = false; // prev2 was `:` Spacing::Joint
    let mut prev2_is_dot = false; // prev2 was `.`

    while let Some(tt) = iter.next() {
        // -- Detect bare `pure` ident eligible for rewriting ------------------
        if let TokenTree::Ident(ref id) = tt {
            if id == "pure" {
                let next_is_paren = matches!(
                    iter.peek(),
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis
                );
                if next_is_paren {
                    // `::pure`: prev is `:` AND prev2 is `:` Joint.
                    // A lone `:` (struct-field) has prev2_is_colon_joint=false.
                    let is_path_qual = prev_is_colon && prev2_is_colon_joint;
                    // `.pure`: prev is `.` AND prev2 is NOT `.`.
                    // `..pure` has prev2_is_dot=true, so it is NOT suppressed.
                    let is_method = prev_is_dot && !prev2_is_dot;

                    if !is_path_qual && !is_method {
                        // Emit qualified UFCS path, carrying the ident's span
                        // for better error locality.
                        let span = id.span();
                        let rewritten = quote_spanned! { span =>
                            <#marker as ::monadify::Applicative<_>>::pure
                        };
                        out.extend(rewritten);
                        // Update history: `pure` ident is not a `:` or `.`.
                        prev2_is_colon_joint = prev_is_colon && prev_colon_is_joint;
                        prev2_is_dot = prev_is_dot;
                        prev_is_colon = false;
                        prev_colon_is_joint = false;
                        prev_is_dot = false;
                        continue;
                    }
                }
            }
        }

        // -- Update two-token history with `tt` before emitting it ------------
        prev2_is_colon_joint = prev_is_colon && prev_colon_is_joint;
        prev2_is_dot = prev_is_dot;
        prev_is_colon = is_colon(&tt);
        prev_colon_is_joint = is_colon_joint(&tt);
        prev_is_dot = is_dot(&tt);

        // -- Recurse into groups; emit everything else verbatim ----------------
        match tt {
            TokenTree::Group(g) => {
                let inner = rewrite_pure(g.stream(), marker);
                let mut new_g = Group::new(g.delimiter(), inner);
                new_g.set_span(g.span());
                out.extend(std::iter::once(TokenTree::Group(new_g)));
            }
            other => {
                out.extend(std::iter::once(other));
            }
        }
    }

    out
}

/// Returns the index of a top-level `<-` (a `<` joined to a following `-`).
fn find_left_arrow(tokens: &[TokenTree]) -> Option<usize> {
    for i in 0..tokens.len().saturating_sub(1) {
        if let (TokenTree::Punct(a), TokenTree::Punct(b)) = (&tokens[i], &tokens[i + 1]) {
            if a.as_char() == '<' && a.spacing() == Spacing::Joint && b.as_char() == '-' {
                return Some(i);
            }
        }
    }
    None
}

/// Classify one statement segment (tokens between top-level `;`) into a [`Stmt`].
///
/// `marker` is the block's Kind marker type; it is forwarded to [`rewrite_pure`]
/// so that bare `pure(…)` calls in Bind, Bare, and Guard positions are rewritten
/// to `<marker as ::monadify::Applicative<_>>::pure(…)` before parsing.
/// Plain `let` bodies are explicitly **not** rewritten (the spec carve-out).
fn classify(tokens: Vec<TokenTree>, marker: &Type) -> SynResult<Stmt> {
    if tokens.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "empty statement in mdo! block (stray `;`?)",
        ));
    }

    // 1. `pat <- expr` — highest priority (the `<-` is unambiguous).
    if let Some(idx) = find_left_arrow(&tokens) {
        let pat_ts: TokenStream2 = tokens[..idx].iter().cloned().collect();
        let raw_expr_ts: TokenStream2 = tokens[idx + 2..].iter().cloned().collect();
        if pat_ts.is_empty() {
            return Err(Error::new(
                tokens[idx].span(),
                "mdo! bind requires a pattern before `<-`",
            ));
        }
        if raw_expr_ts.is_empty() {
            return Err(Error::new(
                tokens[idx].span(),
                "mdo! bind requires a monadic expression after `<-`",
            ));
        }
        let pat = Pat::parse_single.parse2(pat_ts)?;
        let expr_ts = rewrite_pure(raw_expr_ts, marker);
        let expr: Expr = syn::parse2(expr_ts)?;
        return Ok(Stmt::Bind(pat, expr));
    }

    // 2. `let …` — keep verbatim (no bare-pure rewrite inside plain let bodies).
    if let TokenTree::Ident(id) = &tokens[0] {
        if id == "let" {
            return Ok(Stmt::Let(tokens.into_iter().collect()));
        }
        // 3. `guard ( … )` recognised only as a whole statement.
        if id == "guard" && tokens.len() == 2 {
            if let TokenTree::Group(g) = &tokens[1] {
                if g.delimiter() == Delimiter::Parenthesis {
                    return Ok(Stmt::Guard(rewrite_pure(g.stream(), marker)));
                }
            }
        }
    }

    // 4. bare expression (sequencing) — rewrite pure before parsing.
    let raw_ts: TokenStream2 = tokens.into_iter().collect();
    let expr_ts = rewrite_pure(raw_ts, marker);
    let expr: Expr = syn::parse2(expr_ts)?;
    Ok(Stmt::Bare(expr))
}

impl Parse for MdoInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "mdo! requires a block marker followed by at least a final expression, \
                 e.g. `mdo! { OptionKind; OptionKind::pure(1) }`",
            ));
        }

        let marker: Type = input.parse().map_err(|_| {
            Error::new(
                Span::call_site(),
                "mdo! must start with a block marker type, e.g. `mdo! { OptionKind; … }`",
            )
        })?;
        if !matches!(marker, Type::Path(_)) {
            return Err(Error::new_spanned(
                &marker,
                "mdo! block marker must be a type path, e.g. `OptionKind` or `ResultKind::<String>`",
            ));
        }
        input.parse::<Token![;]>().map_err(|_| {
            Error::new(
                Span::call_site(),
                "mdo! marker must be followed by `;`, e.g. `mdo! { OptionKind; … }`",
            )
        })?;

        // Grab the remaining tokens and split on top-level `;` (semicolons inside
        // groups like `{ … }` or `( … )` are nested and never seen here).
        let rest: TokenStream2 = input.parse()?;
        let mut segments: Vec<(Vec<TokenTree>, bool)> = Vec::new();
        let mut cur: Vec<TokenTree> = Vec::new();
        for tt in rest {
            if let TokenTree::Punct(p) = &tt {
                if p.as_char() == ';' {
                    segments.push((std::mem::take(&mut cur), true));
                    continue;
                }
            }
            cur.push(tt);
        }
        if !cur.is_empty() {
            segments.push((cur, false));
        }

        if segments.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "mdo! requires at least a final expression after the marker",
            ));
        }
        // The last segment must be the trailing final expression (no `;`).
        if segments.last().map(|(_, semi)| *semi).unwrap_or(true) {
            return Err(Error::new(
                Span::call_site(),
                "mdo! block must end with a final monadic expression and no trailing `;`",
            ));
        }

        let (final_tokens, _) = segments.pop().expect("checked non-empty above");
        if let Some(idx) = find_left_arrow(&final_tokens) {
            return Err(Error::new(
                final_tokens[idx].span(),
                "the final line of an mdo! block must be a raw monadic value, not a `<-` bind",
            ));
        }
        // Apply bare-pure rewriting to the final expression before parsing it.
        let final_raw_ts: TokenStream2 = final_tokens.into_iter().collect();
        let final_expr: Expr = syn::parse2(rewrite_pure(final_raw_ts, &marker))?;

        let mut stmts = Vec::with_capacity(segments.len());
        for (tokens, _) in segments {
            stmts.push(classify(tokens, &marker)?);
        }

        Ok(MdoInput {
            marker,
            stmts,
            final_expr,
        })
    }
}

/// Procedural `do`-notation macro.
///
/// `mdo!` desugars an imperative monadic block into nested `Marker::bind(..)`
/// calls over `monadify`'s `Bind`/`Applicative` traits. The block's Kind marker
/// is named explicitly as the first token group, terminated by `;`:
///
/// ```text
/// mdo! { Marker;
///     pat <- expr;        // monadic bind
///     let pat = expr;     // plain let
///     guard(cond);        // filter (Option/Vec only)
///     expr;               // sequencing (== `_ <- expr`)
///     final_expr          // raw monadic result (NOT auto-pure-wrapped)
/// }
/// ```
///
/// Each non-final monadic right-hand side is cloned (`(expr).clone()`) because
/// `bind`'s closure is `FnMut + Clone + 'static` and `VecKind::bind` re-invokes
/// it per element. The final expression is returned raw as `Marker::Of<B>`.
///
/// # The `pure` keyword
///
/// Inside an `mdo!` block, any bare call of the form `pure(expr)` — where
/// `pure` appears as a free identifier immediately followed by a `(`-group —
/// is automatically rewritten to
/// `<Marker as ::monadify::Applicative<_>>::pure(expr)`.
///
/// The rewriter uses two-token look-behind to avoid false positives:
///
/// - `pure` is **not** rewritten when `::` -qualified: `Marker::pure(x)` is
///   left verbatim (the two preceding `:` tokens form the `::` digraph).
/// - `pure` is **not** rewritten when called as a method: `x.pure(y)` is
///   left verbatim (a lone `.` precedes `pure` and the token before that is
///   not another `.`).
/// - `pure` in a struct-field position (`Foo { f: pure(x) }`) **is**
///   rewritten — the single `:` before `pure` is distinct from `::` because
///   no Joint `:` precedes it.
/// - `pure` after `..` (`..pure(base)`) **is** rewritten — the double `.`
///   forms a range/struct-update digraph, not a method receiver.
/// - `pure::<T>(x)` is **not** rewritten — the next token after `pure` is
///   `<`, not `(`, so the call-shape guard prevents rewriting.
/// - `let`-body carve-out is statement-level only: `let x = pure(v)` inside
///   an `mdo!` block is a `Stmt::Let` and the right-hand side is **not**
///   rewritten. Use `x <- pure(v)` (bind) instead.
///
/// # Limitations
///
/// **`CFnKind` / `CFnOnceKind` are not supported.** `CFn` and `CFnOnce` wrap
/// `Box<dyn Fn(…)>` / `Box<dyn FnOnce(…)>`, which are not `Clone`. The
/// desugaring always emits `(expr).clone()` for each monadic right-hand side
/// (required because `bind`'s continuation is `FnMut + Clone + 'static`), so any
/// `mdo!` block over `CFnKind`/`CFnOnceKind` — even at depth 1 — fails to
/// compile with `E0599` (*the trait bound `CFn<…>: Clone` is not satisfied*).
/// This exclusion is by design. A future `Rc`-backed clonable function wrapper
/// (analogous to `ReaderT`'s `Rc<dyn Fn(R) -> M::Of<A>>`) could lift the
/// restriction; until then, use `ReaderTKind<R, IdentityKind>` as an alternative
/// when a function monad in a do-block is needed.
///
/// **At most one non-`Copy` external value may be captured per `mdo!` nesting
/// level.** The desugaring emits nested `move` closures bound by
/// `FnMut + Clone + 'static`, so a non-`Copy` value captured by an outer `move`
/// closure cannot also be referenced from an inner (deeper) closure — doing so
/// moves it out of the outer `FnMut`, which triggers `E0507`
/// (*cannot move out of a captured variable in an `FnMut` closure*).
///
/// In practice this bites when a do-block reads the same non-`Copy` external
/// (e.g. a `ReaderT`, `String`, or other non-`Copy` binding) at two different
/// bind depths. The bound results of monadic steps are usually `Copy` (`i32`,
/// `bool`, …) and cross nesting levels freely; the constraint applies to
/// *external* non-`Copy` values referenced inside the block.
///
/// **Workaround:** combine the multiple reads into a single step that returns a
/// tuple, so only the resulting `Copy` components cross into deeper nesting
/// levels. For example, instead of binding two separate `ReaderT`s that each
/// read a field, read both fields in one `ReaderT` returning `(a, b)`:
///
/// ```text
/// // Instead of two non-Copy reader steps at different depths, do:
/// (a, b) <- ReaderT::new(|cfg: Config| Identity((cfg.base, cfg.factor)));
/// other  <- some_other_reader;   // `a`/`b` are Copy and cross freely
/// Marker::pure(a + b + other)
/// ```
///
/// # Example
///
/// ```text
/// use monadify::{mdo, OptionKind, Applicative};
///
/// let r: Option<i32> = mdo! { OptionKind;
///     x <- Some(2);
///     y <- Some(3);
///     OptionKind::pure(x + y)
/// };
/// assert_eq!(r, Some(5));
/// ```
///
/// A compiling, runnable version of this example is documented on the
/// `monadify::mdo` re-export (see the `monadify` crate), where it can depend on
/// `monadify` itself without introducing a build cycle.
#[proc_macro]
pub fn mdo(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse::<MdoInput>(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let marker = &parsed.marker;
    // Right-fold the statements into nested binds, starting from the raw final.
    let mut acc: TokenStream2 = parsed.final_expr.to_token_stream();
    for stmt in parsed.stmts.iter().rev() {
        acc = match stmt {
            Stmt::Bind(pat, expr) => quote! {
                <#marker as ::monadify::Bind<_, _>>::bind(
                    (#expr).clone(),
                    move |#pat| { #acc }
                )
            },
            Stmt::Bare(expr) => quote! {
                <#marker as ::monadify::Bind<_, _>>::bind(
                    (#expr).clone(),
                    move |_| { #acc }
                )
            },
            Stmt::Guard(cond) => quote! {
                <#marker as ::monadify::Bind<_, _>>::bind(
                    (<#marker as ::monadify::MdoGuard>::guard(#cond)).clone(),
                    move |_| { #acc }
                )
            },
            Stmt::Let(raw) => quote! {
                { #raw ; #acc }
            },
        };
    }

    acc.into()
}
