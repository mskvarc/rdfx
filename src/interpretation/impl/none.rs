use std::borrow::Cow;

use iri_rs::{Iri, IriBuf};

use crate::{
    CowLiteral,
    Id,
    InterpretationMut,
    LiteralRef,
    LocalTerm,
    Term,
    interpretation::{Interpretation, ReverseInterpretation},
};

impl Interpretation for () {
    type Resource = LocalTerm;

    fn iri(&self, iri: Iri<&str>) -> Option<LocalTerm> {
        Some(LocalTerm::iri(iri.into()))
    }

    fn literal<'a>(&self, literal: impl Into<LiteralRef<'a>>) -> Option<LocalTerm> {
        Some(LocalTerm::literal(literal.into().to_owned()))
    }
}

impl InterpretationMut for () {
    fn insert_iri<'a>(&mut self, iri: impl Into<Cow<'a, IriBuf>>) -> Self::Resource {
        LocalTerm::iri(iri.into().into_owned())
    }

    fn insert_literal<'a>(&mut self, literal: impl Into<CowLiteral<'a>>) -> Self::Resource {
        LocalTerm::literal(literal.into().into_owned())
    }
}

impl ReverseInterpretation for () {
    type Iris<'a> = std::option::IntoIter<Cow<'a, IriBuf>>;
    type Literals<'a> = std::option::IntoIter<CowLiteral<'a>>;

    fn iris_of<'a>(&'a self, resource: &'a Self::Resource) -> Self::Iris<'a> {
        match resource {
            LocalTerm::Named(Term::Iri(iri)) => Some(Cow::Borrowed(iri)).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn literals_of<'a>(&'a self, resource: &'a Self::Resource) -> Self::Literals<'a> {
        match resource {
            LocalTerm::Named(Term::Literal(l)) => Some(l.as_cow()).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn triple_term_components(&self, resource: &LocalTerm) -> Option<(LocalTerm, LocalTerm, LocalTerm)> {
        match resource {
            LocalTerm::Triple(t) => Some((id_to_local_term(&t.0), LocalTerm::iri(t.1.clone()), t.2.clone())),
            _ => None,
        }
    }

    /// Zero-clone for the (recursive, hot) Object position; Subject and
    /// Predicate still allocate cheap wrappers (Id / IriBuf → LocalTerm).
    fn triple_term_components_view<'a>(&'a self, resource: &'a LocalTerm) -> Option<(Cow<'a, LocalTerm>, Cow<'a, LocalTerm>, Cow<'a, LocalTerm>)> {
        match resource {
            LocalTerm::Triple(t) => Some((Cow::Owned(id_to_local_term(&t.0)), Cow::Owned(LocalTerm::iri(t.1.clone())), Cow::Borrowed(&t.2))),
            _ => None,
        }
    }
}

fn id_to_local_term(id: &Id) -> LocalTerm {
    match id {
        Id::Iri(iri) => LocalTerm::iri(iri.clone()),
        Id::BlankId(b) => LocalTerm::BlankId(b.clone()),
    }
}
