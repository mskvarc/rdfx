use crate::{BlankId, BlankIdBuf, Id, LocalTerm, Term};

/// Types that may represent a blank node identifier.
pub trait MaybeBlankId {
    /// Inner blank node identifier type.
    type BlankId;
}

impl MaybeBlankId for Id {
    type BlankId = BlankIdBuf;
}

impl MaybeBlankId for Term {
    type BlankId = BlankIdBuf;
}

impl MaybeBlankId for LocalTerm {
    type BlankId = BlankIdBuf;
}

/// Types that may have a blank node identifier representation that can be
/// borrowed.
pub trait TryAsBlankId: MaybeBlankId {
    /// Returns a reference to the blank node identifier value, if any.
    fn try_as_blank(&self) -> Option<&BlankId>;

    fn is_blank(&self) -> bool {
        self.try_as_blank().is_some()
    }
}

impl TryAsBlankId for Id {
    fn try_as_blank(&self) -> Option<&BlankId> {
        self.as_blank_id()
    }
}

impl TryAsBlankId for Term {
    fn try_as_blank(&self) -> Option<&BlankId> {
        None
    }
}

impl TryAsBlankId for LocalTerm {
    fn try_as_blank(&self) -> Option<&BlankId> {
        self.as_blank_id()
    }
}

/// Types that can be turned into a blank node identifier.
pub trait TryIntoBlankId: MaybeBlankId + Sized {
    fn try_into_blank(self) -> Result<BlankIdBuf, Self>;
}

impl TryIntoBlankId for Id {
    fn try_into_blank(self) -> Result<BlankIdBuf, Self> {
        match self {
            Self::BlankId(b) => Ok(b),
            other => Err(other),
        }
    }
}

impl TryIntoBlankId for Term {
    fn try_into_blank(self) -> Result<BlankIdBuf, Self> {
        Err(self)
    }
}

impl TryIntoBlankId for LocalTerm {
    fn try_into_blank(self) -> Result<BlankIdBuf, Self> {
        match self {
            Self::BlankId(b) => Ok(b),
            other => Err(other),
        }
    }
}

/// Types that can be constructed from a blank node identifier.
pub trait FromBlankId: MaybeBlankId {
    /// Builds a value from a blank node identifier.
    fn from_blank(b: BlankIdBuf) -> Self;
}

impl FromBlankId for Id {
    fn from_blank(b: BlankIdBuf) -> Self {
        Self::BlankId(b)
    }
}

impl FromBlankId for LocalTerm {
    fn from_blank(b: BlankIdBuf) -> Self {
        Self::BlankId(b)
    }
}
