use crate::{Literal, LiteralRef, LocalTerm, Term};

/// Types that may represent a literal value.
pub trait MaybeLiteral {
    /// Inner literal type.
    type Literal;
}

impl MaybeLiteral for Literal {
    type Literal = Self;
}

impl MaybeLiteral for Term {
    type Literal = Literal;
}

impl MaybeLiteral for LocalTerm {
    type Literal = Literal;
}

/// Types that may have a literal representation that can be borrowed.
pub trait TryAsLiteral: MaybeLiteral {
    /// Returns a reference to the literal value, if any.
    fn try_as_literal(&self) -> Option<LiteralRef<'_>>;

    fn is_literal(&self) -> bool {
        self.try_as_literal().is_some()
    }
}

impl TryAsLiteral for Term {
    fn try_as_literal(&self) -> Option<LiteralRef<'_>> {
        self.as_literal()
    }
}

impl TryAsLiteral for LocalTerm {
    fn try_as_literal(&self) -> Option<LiteralRef<'_>> {
        self.as_literal()
    }
}

/// Types that can be turned into a literal.
pub trait TryIntoLiteral: MaybeLiteral + Sized {
    fn try_into_literal(self) -> Result<Literal, Self>;
}

impl TryIntoLiteral for Term {
    fn try_into_literal(self) -> Result<Literal, Self> {
        Term::try_into_literal(self).map_err(Self::Iri)
    }
}

impl TryIntoLiteral for LocalTerm {
    fn try_into_literal(self) -> Result<Literal, Self> {
        match self {
            Self::Named(Term::Literal(l)) => Ok(l),
            other => Err(other),
        }
    }
}

/// Types that can be constructed from a literal.
pub trait FromLiteral: MaybeLiteral {
    /// Builds a value from a literal.
    fn from_literal(l: Literal) -> Self;
}

impl FromLiteral for Term {
    fn from_literal(l: Literal) -> Self {
        Self::Literal(l)
    }
}

impl FromLiteral for LocalTerm {
    fn from_literal(l: Literal) -> Self {
        Self::literal(l)
    }
}
