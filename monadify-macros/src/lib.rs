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
use proc_macro2::{Delimiter, Spacing, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
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
fn classify(tokens: Vec<TokenTree>) -> SynResult<Stmt> {
    if tokens.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "empty statement in mdo! block (stray `;`?)",
        ));
    }

    // 1. `pat <- expr` — highest priority (the `<-` is unambiguous).
    if let Some(idx) = find_left_arrow(&tokens) {
        let pat_ts: TokenStream2 = tokens[..idx].iter().cloned().collect();
        let expr_ts: TokenStream2 = tokens[idx + 2..].iter().cloned().collect();
        if pat_ts.is_empty() {
            return Err(Error::new(
                tokens[idx].span(),
                "mdo! bind requires a pattern before `<-`",
            ));
        }
        if expr_ts.is_empty() {
            return Err(Error::new(
                tokens[idx].span(),
                "mdo! bind requires a monadic expression after `<-`",
            ));
        }
        let pat = Pat::parse_single.parse2(pat_ts)?;
        let expr: Expr = syn::parse2(expr_ts)?;
        return Ok(Stmt::Bind(pat, expr));
    }

    // 2. `let …` — keep verbatim.
    if let TokenTree::Ident(id) = &tokens[0] {
        if id == "let" {
            return Ok(Stmt::Let(tokens.into_iter().collect()));
        }
        // 3. `guard ( … )` recognised only as a whole statement.
        if id == "guard" && tokens.len() == 2 {
            if let TokenTree::Group(g) = &tokens[1] {
                if g.delimiter() == Delimiter::Parenthesis {
                    return Ok(Stmt::Guard(g.stream()));
                }
            }
        }
    }

    // 4. bare expression (sequencing).
    let expr: Expr = syn::parse2(tokens.into_iter().collect())?;
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
        let final_expr: Expr = syn::parse2(final_tokens.into_iter().collect())?;

        let mut stmts = Vec::with_capacity(segments.len());
        for (tokens, _) in segments {
            stmts.push(classify(tokens)?);
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
