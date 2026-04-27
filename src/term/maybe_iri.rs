use crate::{Id, LocalTerm, Term};
use iri_rs::{Iri, IriBuf};

/// Types that may represent an iri.
pub trait MaybeIri {
    /// Inner iri type.
    type Iri;
}

impl MaybeIri for Id {
    type Iri = IriBuf;
}

impl MaybeIri for Term {
    type Iri = IriBuf;
}

impl MaybeIri for LocalTerm {
    type Iri = IriBuf;
}

/// Types that may have an iri representation that can be borrowed.
pub trait TryAsIri: MaybeIri {
    /// Returns a reference to the iri value, if any.
    fn try_as_iri(&self) -> Option<Iri<&str>>;

    fn is_iri(&self) -> bool {
        self.try_as_iri().is_some()
    }
}

impl TryAsIri for Id {
    fn try_as_iri(&self) -> Option<Iri<&str>> {
        self.as_iri()
    }
}

impl TryAsIri for Term {
    fn try_as_iri(&self) -> Option<Iri<&str>> {
        self.as_iri()
    }
}

impl TryAsIri for LocalTerm {
    fn try_as_iri(&self) -> Option<Iri<&str>> {
        self.as_iri()
    }
}

/// Types that can be turned into an iri.
pub trait TryIntoIri: MaybeIri + Sized {
    fn try_into_iri(self) -> Result<IriBuf, Self>;
}

impl TryIntoIri for Id {
    fn try_into_iri(self) -> Result<IriBuf, Self> {
        match self {
            Self::Iri(iri) => Ok(iri),
            other => Err(other),
        }
    }
}

impl TryIntoIri for Term {
    fn try_into_iri(self) -> Result<IriBuf, Self> {
        Term::try_into_iri(self)
    }
}

impl TryIntoIri for LocalTerm {
    fn try_into_iri(self) -> Result<IriBuf, Self> {
        match self {
            Self::Named(Term::Iri(iri)) => Ok(iri),
            other => Err(other),
        }
    }
}

/// Types that can be constructed from an iri.
pub trait FromIri: MaybeIri {
    /// Builds a value from an iri.
    fn from_iri(iri: IriBuf) -> Self;
}

impl FromIri for Id {
    fn from_iri(iri: IriBuf) -> Self {
        Self::Iri(iri)
    }
}

impl FromIri for Term {
    fn from_iri(iri: IriBuf) -> Self {
        Self::Iri(iri)
    }
}

impl FromIri for LocalTerm {
    fn from_iri(iri: IriBuf) -> Self {
        Self::iri(iri)
    }
}
