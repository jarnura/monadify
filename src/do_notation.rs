//! Support items for the `do-notation` feature's [`mdo!`](crate::mdo) macro.
//!
//! This module is compiled only when the non-default `do-notation` feature is
//! enabled. It currently provides [`MdoGuard`], the minimal "monadic zero"
//! helper that backs `guard(..)` statements inside `mdo!` blocks. It is
//! intentionally implemented **only** for instances with a lawful empty element
//! ([`OptionKind`] and [`VecKind`]); using `guard` under any other marker is a
//! deliberate compile error.

use crate::kind_based::kind::{Kind1, OptionKind, VecKind};

/// Private module implementing the sealed-trait pattern for [`MdoGuard`].
///
/// The `Sealed` trait is intentionally **not** re-exported, so downstream
/// crates cannot name it and therefore cannot satisfy the `MdoGuard: Sealed`
/// supertrait bound. This restricts `MdoGuard` implementations to the markers
/// sealed here ([`OptionKind`] and [`VecKind`]).
mod sealed {
    use crate::kind_based::kind::{OptionKind, VecKind};

    /// Sealing marker trait. Not re-exported, so foreign impls are impossible.
    pub trait Sealed {}

    impl Sealed for OptionKind {}
    impl Sealed for VecKind {}
}

/// Monadic zero used by `guard(..)` inside [`mdo!`](crate::mdo).
///
/// `guard(cond)` desugars to a `bind` over `Self::guard(cond)`: it yields
/// `pure(())` when `cond` is `true` and the instance's lawful zero when `false`,
/// short-circuiting the rest of the block.
///
/// Implemented only for [`OptionKind`] (`None` on `false`) and [`VecKind`]
/// (`vec![]` on `false`) — the two instances with a genuine empty element.
///
/// This trait is **sealed**: it has a private `Sealed` supertrait that only
/// `monadify` can implement, so downstream crates cannot add their own
/// `MdoGuard` instances.
pub trait MdoGuard: Kind1 + sealed::Sealed {
    /// Returns `pure(())` when `cond` is `true`, or this instance's zero when
    /// `cond` is `false`.
    fn guard(cond: bool) -> Self::Of<()>;
}

impl MdoGuard for OptionKind {
    fn guard(cond: bool) -> Option<()> {
        if cond {
            Some(())
        } else {
            None
        }
    }
}

impl MdoGuard for VecKind {
    fn guard(cond: bool) -> Vec<()> {
        if cond {
            vec![()]
        } else {
            Vec::new()
        }
    }
}
