use replace_with::replace_with_or_abort_and_return;

use crate::{
    Resource,
    Triple,
    pattern::{CanonicalQuadPattern, ResourceOrVar, TriplePattern, quad},
};

/// Canonical triple pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalTriplePattern<T> {
    /// Any subject.
    AnySubject(AnySubject<T>),
    /// A given subject.
    GivenSubject(T, GivenSubject<T>),
}

impl<T: Resource> From<Triple<T, T, T>> for CanonicalTriplePattern<T> {
    fn from(value: Triple<T, T, T>) -> Self {
        Self::from_triple(value)
    }
}

impl<T: Resource> From<Triple<Option<T>, Option<T>, Option<T>>> for CanonicalTriplePattern<T> {
    fn from(value: Triple<Option<T>, Option<T>, Option<T>>) -> Self {
        Self::from_option_triple(value)
    }
}

impl<T: Resource, X: PartialEq> From<TriplePattern<T, X>> for CanonicalTriplePattern<T> {
    fn from(value: TriplePattern<T, X>) -> Self {
        Self::from_pattern(value)
    }
}

impl<T: Resource> CanonicalTriplePattern<T> {
    /// Builds a pattern matching exactly `triple`, in any graph.
    pub fn from_triple(triple: Triple<T, T, T>) -> Self {
        Self::GivenSubject(
            triple.0,
            GivenSubject::GivenPredicate(triple.1, GivenSubjectGivenPredicate::GivenObject(triple.2)),
        )
    }

    /// Builds a pattern from a triple whose unset positions match anything.
    pub fn from_option_triple(triple: Triple<Option<T>, Option<T>, Option<T>>) -> Self {
        match triple.0 {
            Some(s) => Self::GivenSubject(s, GivenSubject::from_option_triple(triple.1, triple.2)),
            None => Self::AnySubject(AnySubject::from_option_triple(triple.1, triple.2)),
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(pattern: TriplePattern<T, X>) -> Self {
        match pattern.0 {
            ResourceOrVar::Resource(s) => Self::GivenSubject(s, GivenSubject::from_pattern(pattern.1, pattern.2)),
            ResourceOrVar::Var(s) => Self::AnySubject(AnySubject::from_pattern(s, pattern.1, pattern.2)),
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> CanonicalQuadPattern<T> {
        match self {
            Self::AnySubject(p) => CanonicalQuadPattern::AnySubject(p.with_any_graph()),
            Self::GivenSubject(id, p) => CanonicalQuadPattern::GivenSubject(id, p.with_any_graph()),
        }
    }

    /// Returns the subject of this pattern.
    pub const fn subject(&self) -> PatternSubject<&T> {
        match self {
            Self::AnySubject(_) => PatternSubject::Any,
            Self::GivenSubject(id, _) => PatternSubject::Given(id),
        }
    }

    /// Consumes this pattern, returning its subject.
    pub fn into_subject(self) -> PatternSubject<T> {
        match self {
            Self::AnySubject(_) => PatternSubject::Any,
            Self::GivenSubject(id, _) => PatternSubject::Given(id),
        }
    }

    /// Returns the predicate of this pattern.
    pub const fn predicate(&self) -> PatternPredicate<&T> {
        match self {
            Self::AnySubject(t) => t.predicate(),
            Self::GivenSubject(_, t) => t.predicate(),
        }
    }

    /// Consumes this pattern, returning its predicate.
    pub fn into_predicate(self) -> PatternPredicate<T> {
        match self {
            Self::AnySubject(t) => t.into_predicate(),
            Self::GivenSubject(_, t) => t.into_predicate(),
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnySubject(t) => t.object(),
            Self::GivenSubject(_, t) => t.object(),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnySubject(t) => t.into_object(),
            Self::GivenSubject(_, t) => t.into_object(),
        }
    }

    /// Sets the subject of this pattern, returning the previous one.
    pub fn set_subject(&mut self, s: T) -> PatternSubject<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::AnyObject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::AnyObject)),
            ),
            Self::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::SameAsSubject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s.clone(), GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(s))),
            ),
            Self::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::SameAsPredicate)) => (
                PatternSubject::Any,
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::SameAsPredicate)),
            ),
            Self::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::GivenObject(o))) => (
                PatternSubject::Any,
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(o))),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::AnyObject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s.clone(), GivenSubject::GivenPredicate(s, GivenSubjectGivenPredicate::AnyObject)),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::SameAsSubject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s.clone(), GivenSubject::GivenPredicate(s.clone(), GivenSubjectGivenPredicate::GivenObject(s))),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::GivenObject(o))) => (
                PatternSubject::Any,
                Self::GivenSubject(s.clone(), GivenSubject::GivenPredicate(s, GivenSubjectGivenPredicate::GivenObject(o))),
            ),
            Self::AnySubject(AnySubject::GivenPredicate(p, AnySubjectGivenPredicate::AnyObject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::AnyObject)),
            ),
            Self::AnySubject(AnySubject::GivenPredicate(p, AnySubjectGivenPredicate::SameAsSubject)) => (
                PatternSubject::Any,
                Self::GivenSubject(s.clone(), GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(s))),
            ),
            Self::AnySubject(AnySubject::GivenPredicate(p, AnySubjectGivenPredicate::GivenObject(o))) => (
                PatternSubject::Any,
                Self::GivenSubject(s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(o))),
            ),
            Self::GivenSubject(current_s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::AnyObject)) => (
                PatternSubject::Given(current_s),
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::AnyObject)),
            ),
            Self::GivenSubject(current_s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::SameAsPredicate)) => (
                PatternSubject::Given(current_s),
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::SameAsPredicate)),
            ),
            Self::GivenSubject(current_s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(o))) => (
                PatternSubject::Given(current_s),
                Self::GivenSubject(s, GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(o))),
            ),
            Self::GivenSubject(current_s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::AnyObject)) => (
                PatternSubject::Given(current_s),
                Self::GivenSubject(s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::AnyObject)),
            ),
            Self::GivenSubject(current_s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(o))) => (
                PatternSubject::Given(current_s),
                Self::GivenSubject(s, GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(o))),
            ),
        })
    }

    /// Returns this pattern with its subject set to the given resource.
    pub fn with_subject(mut self, s: T) -> Self
    where
        T: Clone,
    {
        self.set_subject(s);
        self
    }

    /// Sets the predicate of this pattern, returning the previous one.
    pub fn set_predicate(&mut self, p: T) -> PatternPredicate<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::AnyObject)) => (
                PatternPredicate::SameAsSubject,
                Self::GivenSubject(p.clone(), GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::AnyObject)),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::SameAsSubject)) => (
                PatternPredicate::SameAsSubject,
                Self::GivenSubject(p.clone(), GivenSubject::GivenPredicate(p.clone(), GivenSubjectGivenPredicate::GivenObject(p))),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::GivenObject(o))) => (
                PatternPredicate::SameAsSubject,
                Self::GivenSubject(p.clone(), GivenSubject::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(o))),
            ),
            Self::AnySubject(mut current_p) => {
                let old_p = current_p.set_predicate(p);
                (old_p, Self::AnySubject(current_p))
            }
            Self::GivenSubject(s, mut current_p) => {
                let old_p = current_p.set_predicate(p);
                (old_p, Self::GivenSubject(s, current_p))
            }
        })
    }

    /// Returns this pattern with its predicate set to the given resource.
    pub fn with_predicate(mut self, p: T) -> Self
    where
        T: Clone,
    {
        self.set_predicate(p);
        self
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, o: T) -> PatternObject<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::SameAsSubject)) => (
                PatternObject::SameAsSubject,
                Self::GivenSubject(o.clone(), GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(o))),
            ),
            Self::AnySubject(AnySubject::SameAsSubject(AnySubjectGivenPredicate::SameAsSubject)) => (
                PatternObject::SameAsSubject,
                Self::GivenSubject(o.clone(), GivenSubject::GivenPredicate(o.clone(), GivenSubjectGivenPredicate::GivenObject(o))),
            ),
            Self::AnySubject(mut current_p) => {
                let old_p = current_p.set_object(o);
                (old_p, Self::AnySubject(current_p))
            }
            Self::GivenSubject(s, mut current_p) => {
                let old_p = current_p.set_object(o);
                (old_p, Self::GivenSubject(s, current_p))
            }
        })
    }

    /// Returns this pattern with its object set to the given resource.
    pub fn with_object(mut self, o: T) -> Self
    where
        T: Clone,
    {
        self.set_object(o);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern.
pub enum PatternSubject<T> {
    /// Any .
    Any,
    /// A given .
    Given(T),
}

impl<T> PatternSubject<T> {
    /// Returns the bound resource, if this position is given.
    pub const fn id(&self) -> Option<&T> {
        match self {
            Self::Any => None,
            Self::Given(id) => Some(id),
        }
    }

    /// Consumes this value, returning the bound resource, if this position
    /// is given.
    pub fn into_id(self) -> Option<T> {
        match self {
            Self::Any => None,
            Self::Given(id) => Some(id),
        }
    }
}

impl<T> PatternSubject<&T> {
    /// Clones the borrowed resource.
    pub fn cloned(self) -> PatternSubject<T>
    where
        T: Clone,
    {
        match self {
            Self::Any => PatternSubject::Any,
            Self::Given(t) => PatternSubject::Given(t.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern.
pub enum PatternPredicate<T> {
    /// Any .
    Any,
    /// A object equal to the subject.
    SameAsSubject,
    /// A given .
    Given(T),
}

impl<T> PatternPredicate<T> {
    /// Returns the bound resource, if this position is given.
    pub const fn id(&self) -> Option<&T> {
        match self {
            Self::Any => None,
            Self::SameAsSubject => None,
            Self::Given(id) => Some(id),
        }
    }

    /// Consumes this value, returning the bound resource, if this position
    /// is given.
    pub fn into_id(self) -> Option<T> {
        match self {
            Self::Any => None,
            Self::SameAsSubject => None,
            Self::Given(id) => Some(id),
        }
    }
}

impl<T> PatternPredicate<&T> {
    /// Clones the borrowed resource.
    pub fn cloned(self) -> PatternPredicate<T>
    where
        T: Clone,
    {
        match self {
            Self::Any => PatternPredicate::Any,
            Self::SameAsSubject => PatternPredicate::SameAsSubject,
            Self::Given(t) => PatternPredicate::Given(t.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern.
pub enum PatternObject<T> {
    /// Any .
    Any,
    /// A graph equal to the subject.
    SameAsSubject,
    /// A graph equal to the predicate.
    SameAsPredicate,
    /// A given .
    Given(T),
}

impl<T> PatternObject<T> {
    /// Returns the bound resource, if this position is given.
    pub const fn id(&self) -> Option<&T> {
        match self {
            Self::Given(id) => Some(id),
            _ => None,
        }
    }

    /// Consumes this value, returning the bound resource, if this position
    /// is given.
    pub fn into_id(self) -> Option<T> {
        match self {
            Self::Given(id) => Some(id),
            _ => None,
        }
    }
}

impl<T> PatternObject<&T> {
    /// Clones the borrowed resource.
    pub fn cloned(self) -> PatternObject<T>
    where
        T: Clone,
    {
        match self {
            Self::Any => PatternObject::Any,
            Self::SameAsSubject => PatternObject::SameAsSubject,
            Self::SameAsPredicate => PatternObject::SameAsPredicate,
            Self::Given(t) => PatternObject::Given(t.clone()),
        }
    }
}

impl<T: Resource + PartialEq> PatternObject<T> {
    /// Checks whether `triple` matches this pattern.
    pub fn filter_triple(&self, triple: Triple<T, T, T>) -> bool {
        match self {
            Self::Any => true,
            Self::SameAsSubject => triple.2 == triple.0,
            Self::SameAsPredicate => triple.2 == triple.1,
            Self::Given(id) => triple.2 == *id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching any subject.
pub enum AnySubject<T> {
    /// Any predicate.
    AnyPredicate(AnySubjectAnyPredicate<T>),
    /// A predicate equal to the subject.
    SameAsSubject(AnySubjectGivenPredicate<T>),
    /// A given predicate.
    GivenPredicate(T, AnySubjectGivenPredicate<T>),
}

impl<T> AnySubject<T> {
    /// Builds a pattern from a triple whose unset positions match anything.
    pub fn from_option_triple(p: Option<T>, o: Option<T>) -> Self {
        match p {
            Some(p) => Self::GivenPredicate(p, AnySubjectGivenPredicate::from_option(o)),
            None => Self::AnyPredicate(AnySubjectAnyPredicate::from_option(o)),
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(s: X, p: ResourceOrVar<T, X>, o: ResourceOrVar<T, X>) -> Self {
        match p {
            ResourceOrVar::Resource(p) => Self::GivenPredicate(p, AnySubjectGivenPredicate::from_pattern(s, o)),
            ResourceOrVar::Var(p) => {
                if p == s {
                    Self::SameAsSubject(AnySubjectGivenPredicate::from_pattern(s, o))
                } else {
                    Self::AnyPredicate(AnySubjectAnyPredicate::from_pattern(s, p, o))
                }
            }
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::AnySubject<T> {
        match self {
            Self::AnyPredicate(o) => quad::canonical::AnySubject::AnyPredicate(o.with_any_graph()),
            Self::SameAsSubject(o) => quad::canonical::AnySubject::SameAsSubject(o.with_any_graph()),
            Self::GivenPredicate(id, o) => quad::canonical::AnySubject::GivenPredicate(id, o.with_any_graph()),
        }
    }

    /// Returns the predicate of this pattern.
    pub const fn predicate(&self) -> PatternPredicate<&T> {
        match self {
            Self::AnyPredicate(_) => PatternPredicate::Any,
            Self::SameAsSubject(_) => PatternPredicate::SameAsSubject,
            Self::GivenPredicate(id, _) => PatternPredicate::Given(id),
        }
    }

    /// Consumes this pattern, returning its predicate.
    pub fn into_predicate(self) -> PatternPredicate<T> {
        match self {
            Self::AnyPredicate(_) => PatternPredicate::Any,
            Self::SameAsSubject(_) => PatternPredicate::SameAsSubject,
            Self::GivenPredicate(id, _) => PatternPredicate::Given(id),
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyPredicate(t) => t.object(),
            Self::SameAsSubject(t) => t.object(),
            Self::GivenPredicate(_, t) => t.object(),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyPredicate(t) => t.into_object(),
            Self::SameAsSubject(t) => t.into_object(),
            Self::GivenPredicate(_, t) => t.into_object(),
        }
    }

    /// Sets the predicate of this pattern, returning the previous one.
    pub fn set_predicate(&mut self, p: T) -> PatternPredicate<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnyPredicate(AnySubjectAnyPredicate::AnyObject) => (PatternPredicate::Any, Self::GivenPredicate(p, AnySubjectGivenPredicate::AnyObject)),
            Self::AnyPredicate(AnySubjectAnyPredicate::SameAsSubject) => (PatternPredicate::Any, Self::GivenPredicate(p, AnySubjectGivenPredicate::AnyObject)),
            Self::AnyPredicate(AnySubjectAnyPredicate::SameAsPredicate) => {
                (PatternPredicate::Any, Self::GivenPredicate(p.clone(), AnySubjectGivenPredicate::GivenObject(p)))
            }
            Self::AnyPredicate(AnySubjectAnyPredicate::GivenObject(o)) => {
                (PatternPredicate::Any, Self::GivenPredicate(p, AnySubjectGivenPredicate::GivenObject(o)))
            }
            Self::SameAsSubject(AnySubjectGivenPredicate::AnyObject) => {
                (PatternPredicate::SameAsSubject, Self::GivenPredicate(p, AnySubjectGivenPredicate::AnyObject))
            }
            Self::SameAsSubject(AnySubjectGivenPredicate::SameAsSubject) => (
                PatternPredicate::SameAsSubject,
                Self::GivenPredicate(p.clone(), AnySubjectGivenPredicate::GivenObject(p)),
            ),
            Self::SameAsSubject(AnySubjectGivenPredicate::GivenObject(o)) => (
                PatternPredicate::SameAsSubject,
                Self::GivenPredicate(p, AnySubjectGivenPredicate::GivenObject(o)),
            ),
            Self::GivenPredicate(current_p, o) => (PatternPredicate::Given(current_p), Self::GivenPredicate(p, o)),
        })
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, o: T) -> PatternObject<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnyPredicate(AnySubjectAnyPredicate::SameAsPredicate) => {
                (PatternObject::Any, Self::GivenPredicate(o.clone(), AnySubjectGivenPredicate::GivenObject(o)))
            }
            Self::AnyPredicate(mut current_o) => {
                let old_o = current_o.set_object(o);
                (old_o, Self::AnyPredicate(current_o))
            }
            Self::SameAsSubject(AnySubjectGivenPredicate::SameAsSubject) => (
                PatternObject::SameAsSubject,
                Self::GivenPredicate(o.clone(), AnySubjectGivenPredicate::GivenObject(o)),
            ),
            Self::SameAsSubject(mut current_o) => {
                let old_o = current_o.set_object(o);
                (old_o, Self::SameAsSubject(current_o))
            }
            Self::GivenPredicate(p, mut current_o) => {
                let old_o = current_o.set_object(o);
                (old_o, Self::GivenPredicate(p, current_o))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching any subject, any predicate.
pub enum AnySubjectAnyPredicate<T> {
    /// Any object.
    AnyObject,
    /// A object equal to the subject.
    SameAsSubject,
    /// A object equal to the predicate.
    SameAsPredicate,
    /// A given object.
    GivenObject(T),
}

impl<T> AnySubjectAnyPredicate<T> {
    /// Builds a pattern whose unset positions match anything.
    pub fn from_option(o: Option<T>) -> Self {
        match o {
            Some(o) => Self::GivenObject(o),
            None => Self::AnyObject,
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(s: X, p: X, o: ResourceOrVar<T, X>) -> Self {
        match o {
            ResourceOrVar::Resource(o) => Self::GivenObject(o),
            ResourceOrVar::Var(o) => {
                if o == s {
                    Self::SameAsSubject
                } else if o == p {
                    Self::SameAsPredicate
                } else {
                    Self::AnyObject
                }
            }
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::AnySubjectAnyPredicate<T> {
        match self {
            Self::AnyObject => quad::canonical::AnySubjectAnyPredicate::AnyObject(quad::canonical::AnySubjectAnyPredicateAnyObject::AnyGraph),
            Self::SameAsSubject => quad::canonical::AnySubjectAnyPredicate::SameAsSubject(quad::canonical::AnySubjectAnyPredicateGivenObject::AnyGraph),
            Self::SameAsPredicate => quad::canonical::AnySubjectAnyPredicate::SameAsPredicate(quad::canonical::AnySubjectAnyPredicateGivenObject::AnyGraph),
            Self::GivenObject(id) => quad::canonical::AnySubjectAnyPredicate::GivenObject(id, quad::canonical::AnySubjectAnyPredicateGivenObject::AnyGraph),
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsSubject => PatternObject::SameAsSubject,
            Self::SameAsPredicate => PatternObject::SameAsPredicate,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsSubject => PatternObject::SameAsSubject,
            Self::SameAsPredicate => PatternObject::SameAsPredicate,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, t: T) -> PatternObject<T> {
        std::mem::replace(self, Self::GivenObject(t)).into_object()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching any subject, a given predicate.
pub enum AnySubjectGivenPredicate<T> {
    /// Any object.
    AnyObject,
    /// A object equal to the subject.
    SameAsSubject,
    /// A given object.
    GivenObject(T),
}

impl<T> AnySubjectGivenPredicate<T> {
    /// Builds a pattern whose unset positions match anything.
    pub fn from_option(o: Option<T>) -> Self {
        match o {
            Some(o) => Self::GivenObject(o),
            None => Self::AnyObject,
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(s: X, o: ResourceOrVar<T, X>) -> Self {
        match o {
            ResourceOrVar::Resource(o) => Self::GivenObject(o),
            ResourceOrVar::Var(o) => {
                if o == s {
                    Self::SameAsSubject
                } else {
                    Self::AnyObject
                }
            }
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::AnySubjectGivenPredicate<T> {
        match self {
            Self::AnyObject => quad::canonical::AnySubjectGivenPredicate::AnyObject(quad::canonical::AnySubjectGivenPredicateAnyObject::AnyGraph),
            Self::SameAsSubject => quad::canonical::AnySubjectGivenPredicate::SameAsSubject(quad::canonical::AnySubjectGivenPredicateGivenObject::AnyGraph),
            Self::GivenObject(id) => quad::canonical::AnySubjectGivenPredicate::GivenObject(id, quad::canonical::AnySubjectGivenPredicateGivenObject::AnyGraph),
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsSubject => PatternObject::SameAsSubject,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsSubject => PatternObject::SameAsSubject,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, t: T) -> PatternObject<T> {
        std::mem::replace(self, Self::GivenObject(t)).into_object()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching a given subject.
pub enum GivenSubject<T> {
    /// Any predicate.
    AnyPredicate(GivenSubjectAnyPredicate<T>),
    /// A given predicate.
    GivenPredicate(T, GivenSubjectGivenPredicate<T>),
}

impl<T> GivenSubject<T> {
    /// Builds a pattern from a triple whose unset positions match anything.
    pub fn from_option_triple(p: Option<T>, o: Option<T>) -> Self {
        match p {
            Some(p) => Self::GivenPredicate(p, GivenSubjectGivenPredicate::from_option(o)),
            None => Self::AnyPredicate(GivenSubjectAnyPredicate::from_option(o)),
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(p: ResourceOrVar<T, X>, o: ResourceOrVar<T, X>) -> Self {
        match p {
            ResourceOrVar::Resource(p) => Self::GivenPredicate(p, GivenSubjectGivenPredicate::from_pattern(o)),
            ResourceOrVar::Var(p) => Self::AnyPredicate(GivenSubjectAnyPredicate::from_pattern(p, o)),
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::GivenSubject<T> {
        match self {
            Self::AnyPredicate(o) => quad::canonical::GivenSubject::AnyPredicate(o.with_any_graph()),
            Self::GivenPredicate(id, o) => quad::canonical::GivenSubject::GivenPredicate(id, o.with_any_graph()),
        }
    }

    /// Returns the predicate of this pattern.
    pub const fn predicate(&self) -> PatternPredicate<&T> {
        match self {
            Self::AnyPredicate(_) => PatternPredicate::Any,
            Self::GivenPredicate(id, _) => PatternPredicate::Given(id),
        }
    }

    /// Consumes this pattern, returning its predicate.
    pub fn into_predicate(self) -> PatternPredicate<T> {
        match self {
            Self::AnyPredicate(_) => PatternPredicate::Any,
            Self::GivenPredicate(id, _) => PatternPredicate::Given(id),
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyPredicate(t) => t.object(),
            Self::GivenPredicate(_, t) => t.object(),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyPredicate(t) => t.into_object(),
            Self::GivenPredicate(_, t) => t.into_object(),
        }
    }

    /// Sets the predicate of this pattern, returning the previous one.
    pub fn set_predicate(&mut self, p: T) -> PatternPredicate<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnyPredicate(GivenSubjectAnyPredicate::SameAsPredicate) => (
                PatternPredicate::Any,
                Self::GivenPredicate(p.clone(), GivenSubjectGivenPredicate::GivenObject(p)),
            ),
            Self::AnyPredicate(GivenSubjectAnyPredicate::AnyObject) => (PatternPredicate::Any, Self::GivenPredicate(p, GivenSubjectGivenPredicate::AnyObject)),
            Self::AnyPredicate(GivenSubjectAnyPredicate::GivenObject(o)) => {
                (PatternPredicate::Any, Self::GivenPredicate(p, GivenSubjectGivenPredicate::GivenObject(o)))
            }
            Self::GivenPredicate(current_p, o) => (PatternPredicate::Given(current_p), Self::GivenPredicate(p, o)),
        })
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, o: T) -> PatternObject<T>
    where
        T: Clone,
    {
        replace_with_or_abort_and_return(self, |this| match this {
            Self::AnyPredicate(GivenSubjectAnyPredicate::SameAsPredicate) => {
                (PatternObject::Any, Self::GivenPredicate(o.clone(), GivenSubjectGivenPredicate::GivenObject(o)))
            }
            Self::AnyPredicate(mut current_o) => {
                let old_o = current_o.set_object(o);
                (old_o, Self::AnyPredicate(current_o))
            }
            Self::GivenPredicate(p, mut current_o) => {
                let old_o = current_o.set_object(o);
                (old_o, Self::GivenPredicate(p, current_o))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching a given subject, any predicate.
pub enum GivenSubjectAnyPredicate<T> {
    /// Any object.
    AnyObject,
    /// A object equal to the predicate.
    SameAsPredicate,
    /// A given object.
    GivenObject(T),
}

impl<T> GivenSubjectAnyPredicate<T> {
    /// Builds a pattern whose unset positions match anything.
    pub fn from_option(o: Option<T>) -> Self {
        match o {
            Some(o) => Self::GivenObject(o),
            None => Self::AnyObject,
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X: PartialEq>(p: X, o: ResourceOrVar<T, X>) -> Self {
        match o {
            ResourceOrVar::Resource(o) => Self::GivenObject(o),
            ResourceOrVar::Var(o) => {
                if p == o {
                    Self::SameAsPredicate
                } else {
                    Self::AnyObject
                }
            }
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::GivenSubjectAnyPredicate<T> {
        match self {
            Self::AnyObject => quad::canonical::GivenSubjectAnyPredicate::AnyObject(quad::canonical::GivenSubjectAnyPredicateAnyObject::AnyGraph),
            Self::SameAsPredicate => quad::canonical::GivenSubjectAnyPredicate::SameAsPredicate(quad::canonical::GivenSubjectAnyPredicateGivenObject::AnyGraph),
            Self::GivenObject(id) => quad::canonical::GivenSubjectAnyPredicate::GivenObject(id, quad::canonical::GivenSubjectAnyPredicateGivenObject::AnyGraph),
        }
    }

    /// Returns the bound resource, if this position is given.
    pub const fn as_given(&self) -> Option<&T> {
        match self {
            Self::GivenObject(o) => Some(o),
            _ => None,
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsPredicate => PatternObject::SameAsPredicate,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::SameAsPredicate => PatternObject::SameAsPredicate,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, t: T) -> PatternObject<T> {
        std::mem::replace(self, Self::GivenObject(t)).into_object()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Canonical quad pattern matching a given subject, a given predicate.
pub enum GivenSubjectGivenPredicate<T> {
    /// Any object.
    AnyObject,
    /// A given object.
    GivenObject(T),
}

impl<T> GivenSubjectGivenPredicate<T> {
    /// Builds a pattern whose unset positions match anything.
    pub fn from_option(o: Option<T>) -> Self {
        match o {
            Some(o) => Self::GivenObject(o),
            None => Self::AnyObject,
        }
    }

    /// Builds a pattern from `pattern`, canonicalizing repeated
    /// variables into `SameAs` positions.
    pub fn from_pattern<X>(o: ResourceOrVar<T, X>) -> Self {
        match o {
            ResourceOrVar::Resource(o) => Self::GivenObject(o),
            ResourceOrVar::Var(_) => Self::AnyObject,
        }
    }

    /// Lifts this triple pattern into a quad pattern matching any graph.
    pub fn with_any_graph(self) -> quad::canonical::GivenSubjectGivenPredicate<T> {
        match self {
            Self::AnyObject => quad::canonical::GivenSubjectGivenPredicate::AnyObject(quad::canonical::GivenSubjectGivenPredicateAnyObject::AnyGraph),
            Self::GivenObject(id) => {
                quad::canonical::GivenSubjectGivenPredicate::GivenObject(id, quad::canonical::GivenSubjectGivenPredicateGivenObject::AnyGraph)
            }
        }
    }

    /// Returns the bound resource, if this position is given.
    pub const fn as_given(&self) -> Option<&T> {
        match self {
            Self::GivenObject(o) => Some(o),
            _ => None,
        }
    }

    /// Returns the object of this pattern.
    pub const fn object(&self) -> PatternObject<&T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Consumes this pattern, returning its object.
    pub fn into_object(self) -> PatternObject<T> {
        match self {
            Self::AnyObject => PatternObject::Any,
            Self::GivenObject(id) => PatternObject::Given(id),
        }
    }

    /// Sets the object of this pattern, returning the previous one.
    pub fn set_object(&mut self, t: T) -> PatternObject<T> {
        std::mem::replace(self, Self::GivenObject(t)).into_object()
    }
}
