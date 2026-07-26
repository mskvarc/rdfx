use crate::{
    GeneralizedQuad,
    vocabulary::{Vocabulary, VocabularyMut},
};

use super::{
    IdInterpretation,
    IdInterpretationMut,
    IdsOf,
    LiteralInterpretation,
    LiteralInterpretationMut,
    ReverseBlankIdInterpretation,
    ReverseIdInterpretation,
    ReverseIdInterpretationMut,
    ReverseIriInterpretation,
    ReverseLiteralInterpretation,
    TraversableInterpretation,
};

/// Borrowed-reference form of an uninterpreted term, parameterised over the
/// vocabulary's IRI / blank-id / literal id types. Replaces the old
/// `Term<Id<&I, &B>, &L>` once `Term`/`Id` became concrete.
pub enum BorrowedTerm<'a, I, B, L> {
    Iri(&'a I),
    BlankId(&'a B),
    Literal(&'a L),
}

impl<'a, I, B, L> Clone for BorrowedTerm<'a, I, B, L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, I, B, L> Copy for BorrowedTerm<'a, I, B, L> {}

/// Owned-form uninterpreted term used by [`ReverseTermInterpretationMut::assign_term`].
pub enum OwnedTerm<I, B, L> {
    Iri(I),
    BlankId(B),
    Literal(L),
}

impl<I, B, L> __seal::Sealed for BorrowedTerm<'_, I, B, L> {}
impl<I, B, L> crate::IsSubject for BorrowedTerm<'_, I, B, L> {}
impl<I, B, L> crate::IsPredicate for BorrowedTerm<'_, I, B, L> {}
impl<I, B, L> crate::IsObject for BorrowedTerm<'_, I, B, L> {}
impl<I, B, L> crate::IsGraph for BorrowedTerm<'_, I, B, L> {}

use crate::__seal;

/// RDF Term interpretation.
pub trait TermInterpretation<I, B, L>: IdInterpretation<I, B> + LiteralInterpretation<L> {
    fn lexical_term_interpretation(
        &self,
        vocabulary: &impl Vocabulary<Iri = I, BlankId = B, Literal = L>,
        term: crate::LocalTermRef<'_>,
    ) -> Option<Self::Resource> {
        match term {
            crate::LocalTermRef::Named(crate::TermRef::Iri(i)) => self.lexical_iri_interpretation(vocabulary, i),
            crate::LocalTermRef::Named(crate::TermRef::Literal(l)) => self.lexical_literal_interpretation(vocabulary, l),
            crate::LocalTermRef::BlankId(b) => self.lexical_blank_id_interpretation(vocabulary, b),
            crate::LocalTermRef::Triple(_) => None,
        }
    }
}

impl<I, B, L, T: IdInterpretation<I, B> + LiteralInterpretation<L>> TermInterpretation<I, B, L> for T {}

pub trait TermInterpretationMut<I, B, L>: IdInterpretationMut<I, B> + LiteralInterpretationMut<L> {
    fn interpret_term(&mut self, term: OwnedTerm<I, B, L>) -> Self::Resource {
        match term {
            OwnedTerm::Iri(i) => self.interpret_iri(i),
            OwnedTerm::BlankId(b) => self.interpret_blank_id(b),
            OwnedTerm::Literal(l) => self.interpret_literal(l),
        }
    }

    fn interpret_lexical_term(
        &mut self,
        vocabulary: &mut impl VocabularyMut<Iri = I, BlankId = B, Literal = L>,
        term: crate::LocalTermRef<'_>,
    ) -> Option<Self::Resource> {
        match term {
            crate::LocalTermRef::Named(crate::TermRef::Iri(i)) => Some(self.interpret_lexical_iri(vocabulary, i)),
            crate::LocalTermRef::Named(crate::TermRef::Literal(l)) => Some(self.interpret_lexical_literal(vocabulary, l)),
            crate::LocalTermRef::BlankId(b) => Some(self.interpret_lexical_blank_id(vocabulary, b)),
            crate::LocalTermRef::Triple(_) => None,
        }
    }
}

impl<I, B, L, T: IdInterpretationMut<I, B> + LiteralInterpretationMut<L>> TermInterpretationMut<I, B, L> for T {}

pub type TermOf<'a, I> =
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>;

pub type UninterpretedQuadRef<'a, I> = GeneralizedQuad<
    super::id::BorrowedId<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId>,
    &'a <I as ReverseIriInterpretation>::Iri,
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
    super::id::BorrowedId<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId>,
>;

pub type UninterpretedGrdfQuadRef<'a, I> = GeneralizedQuad<
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
    BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
>;

pub trait ReverseTermInterpretation: ReverseIdInterpretation + ReverseLiteralInterpretation {
    fn terms_of_resource<'a>(&'a self, id: &'a Self::Resource) -> TermsOf<'a, Self> {
        TermsOf {
            ids: self.id_iris_of(id),
            literals: ReverseLiteralInterpretation::literals_of(self, id),
        }
    }

    fn term_of<'a>(&'a self, id: &'a Self::Resource) -> Option<TermOf<'a, Self>> {
        self.terms_of_resource(id).next()
    }

    fn has_term(&self, id: &Self::Resource) -> bool {
        self.term_of(id).is_some()
    }

    fn quads_of<'a>(&'a self, quad: GeneralizedQuad<&'a Self::Resource, &'a Self::Resource, &'a Self::Resource, &'a Self::Resource>) -> QuadsOf<'a, Self> {
        QuadsOf {
            s: self.id_iris_of(quad.0),
            p: ReverseIriInterpretation::iris_of(self, quad.1),
            o: self.terms_of_resource(quad.2),
            g: quad.3.map(|g| self.id_iris_of(g)),
            pogs: None,
        }
    }

    fn grdf_quads_of<'a>(
        &'a self,
        quad: GeneralizedQuad<&'a Self::Resource, &'a Self::Resource, &'a Self::Resource, &'a Self::Resource>,
    ) -> GrdfQuadsOf<'a, Self> {
        GrdfQuadsOf {
            s: self.terms_of_resource(quad.0),
            p: self.terms_of_resource(quad.1),
            o: self.terms_of_resource(quad.2),
            g: quad.3.map(|g| self.terms_of_resource(g)),
            pogs: None,
        }
    }
}

impl<I: ?Sized + ReverseIdInterpretation + ReverseLiteralInterpretation> ReverseTermInterpretation for I {}

pub struct QuadsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: IdsOf<'a, I>,
    p: <I as ReverseIriInterpretation>::Iris<'a>,
    o: TermsOf<'a, I>,
    g: Option<IdsOf<'a, I>>,
    pogs: Option<PogsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for QuadsOf<'a, I> {
    type Item = UninterpretedQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.pogs.as_mut() {
                Some(pogs) => match pogs.next() {
                    Some(quad) => break Some(quad),
                    None => self.pogs = None,
                },
                None => match self.s.next() {
                    Some(s) => {
                        self.pogs = Some(PogsOf {
                            s,
                            p: self.p.clone(),
                            o: self.o.clone(),
                            g: self.g.clone(),
                            ogs: None,
                        })
                    }
                    None => break None,
                },
            }
        }
    }
}

struct PogsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: super::id::BorrowedId<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId>,
    p: <I as ReverseIriInterpretation>::Iris<'a>,
    o: TermsOf<'a, I>,
    g: Option<IdsOf<'a, I>>,
    ogs: Option<OgsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for PogsOf<'a, I> {
    type Item = UninterpretedQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.ogs.as_mut() {
                Some(ogs) => match ogs.next() {
                    Some(quad) => break Some(quad),
                    None => self.ogs = None,
                },
                None => match self.p.next() {
                    Some(p) => {
                        self.ogs = Some(OgsOf {
                            s: self.s,
                            p,
                            o: self.o.clone(),
                            g: self.g.clone(),
                            gs: None,
                        })
                    }
                    None => break None,
                },
            }
        }
    }
}

struct OgsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: super::id::BorrowedId<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId>,
    p: &'a <I as ReverseIriInterpretation>::Iri,
    o: TermsOf<'a, I>,
    g: Option<IdsOf<'a, I>>,
    gs: Option<GsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for OgsOf<'a, I> {
    type Item = UninterpretedQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.gs.as_mut() {
                Some(gs) => match gs.next() {
                    Some(quad) => break Some(quad),
                    None => self.gs = None,
                },
                None => match self.o.next() {
                    Some(o) => match self.g.clone() {
                        Some(g) => self.gs = Some(GsOf { s: self.s, p: self.p, o, g }),
                        None => break Some(GeneralizedQuad(self.s, self.p, o, None)),
                    },
                    None => break None,
                },
            }
        }
    }
}

struct GsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: super::id::BorrowedId<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId>,
    p: &'a <I as ReverseIriInterpretation>::Iri,
    o: BorrowedTerm<'a, <I as ReverseIriInterpretation>::Iri, <I as ReverseBlankIdInterpretation>::BlankId, <I as ReverseLiteralInterpretation>::Literal>,
    g: IdsOf<'a, I>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for GsOf<'a, I> {
    type Item = UninterpretedQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.g.next().map(|g| GeneralizedQuad(self.s, self.p, self.o, Some(g)))
    }
}

pub struct GrdfQuadsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: TermsOf<'a, I>,
    p: TermsOf<'a, I>,
    o: TermsOf<'a, I>,
    g: Option<TermsOf<'a, I>>,
    pogs: Option<GrdfPogsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for GrdfQuadsOf<'a, I> {
    type Item = UninterpretedGrdfQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.pogs.as_mut() {
                Some(pogs) => match pogs.next() {
                    Some(quad) => break Some(quad),
                    None => self.pogs = None,
                },
                None => match self.s.next() {
                    Some(s) => {
                        self.pogs = Some(GrdfPogsOf {
                            s,
                            p: self.p.clone(),
                            o: self.o.clone(),
                            g: self.g.clone(),
                            ogs: None,
                        })
                    }
                    None => break None,
                },
            }
        }
    }
}

struct GrdfPogsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: TermOf<'a, I>,
    p: TermsOf<'a, I>,
    o: TermsOf<'a, I>,
    g: Option<TermsOf<'a, I>>,
    ogs: Option<GrdfOgsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for GrdfPogsOf<'a, I> {
    type Item = UninterpretedGrdfQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.ogs.as_mut() {
                Some(ogs) => match ogs.next() {
                    Some(quad) => break Some(quad),
                    None => self.ogs = None,
                },
                None => match self.p.next() {
                    Some(p) => {
                        self.ogs = Some(GrdfOgsOf {
                            s: self.s,
                            p,
                            o: self.o.clone(),
                            g: self.g.clone(),
                            gs: None,
                        })
                    }
                    None => break None,
                },
            }
        }
    }
}

struct GrdfOgsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: TermOf<'a, I>,
    p: TermOf<'a, I>,
    o: TermsOf<'a, I>,
    g: Option<TermsOf<'a, I>>,
    gs: Option<GrdfGsOf<'a, I>>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for GrdfOgsOf<'a, I> {
    type Item = UninterpretedGrdfQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.gs.as_mut() {
                Some(gs) => match gs.next() {
                    Some(quad) => break Some(quad),
                    None => self.gs = None,
                },
                None => match self.o.next() {
                    Some(o) => match self.g.clone() {
                        Some(g) => self.gs = Some(GrdfGsOf { s: self.s, p: self.p, o, g }),
                        None => break Some(GeneralizedQuad(self.s, self.p, o, None)),
                    },
                    None => break None,
                },
            }
        }
    }
}

struct GrdfGsOf<'a, I: ?Sized + ReverseTermInterpretation> {
    s: TermOf<'a, I>,
    p: TermOf<'a, I>,
    o: TermOf<'a, I>,
    g: TermsOf<'a, I>,
}

impl<'a, I: ?Sized + ReverseTermInterpretation> Iterator for GrdfGsOf<'a, I> {
    type Item = UninterpretedGrdfQuadRef<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.g.next().map(|g| GeneralizedQuad(self.s, self.p, self.o, Some(g)))
    }
}

pub trait ReverseLiteralInterpretationMut: ReverseLiteralInterpretation {
    /// Assigns the given literal to the given resource.
    fn assign_literal(&mut self, resource: &Self::Resource, literal: Self::Literal) -> bool;
}

/// Mutable reverse term interpretation.
pub trait ReverseTermInterpretationMut: ReverseIdInterpretationMut + ReverseLiteralInterpretationMut {
    /// Assigns the given term to the given resource.
    fn assign_term(&mut self, resource: &Self::Resource, term: OwnedTerm<Self::Iri, Self::BlankId, Self::Literal>) -> bool {
        match term {
            OwnedTerm::Iri(i) => self.assign_iri(resource, i),
            OwnedTerm::BlankId(b) => self.assign_blank_id(resource, b),
            OwnedTerm::Literal(l) => self.assign_literal(resource, l),
        }
    }

    /// Assigns a term to all the interpreted resources.
    fn assign_terms(&mut self, mut f: impl FnMut(&Self, &Self::Resource) -> Option<OwnedTerm<Self::Iri, Self::BlankId, Self::Literal>>)
    where
        Self::Resource: Clone,
        Self: TraversableInterpretation,
    {
        let mut terms = Vec::new();
        for r in self.resources() {
            if let Some(term) = f(self, r) {
                terms.push((r.clone(), term))
            }
        }

        for (r, term) in terms {
            self.assign_term(&r, term);
        }
    }
}

impl<I: ReverseIdInterpretationMut + ReverseLiteralInterpretationMut> ReverseTermInterpretationMut for I {}

/// Iterator over the terms of an interpreted resource.
pub struct TermsOf<'a, I: 'a + ?Sized + ReverseTermInterpretation> {
    ids: IdsOf<'a, I>,
    literals: <I as ReverseLiteralInterpretation>::Literals<'a>,
}

impl<'a, I: 'a + ?Sized + ReverseTermInterpretation> Clone for TermsOf<'a, I> {
    fn clone(&self) -> Self {
        Self {
            ids: self.ids.clone(),
            literals: self.literals.clone(),
        }
    }
}

impl<'a, I: 'a + ?Sized + ReverseTermInterpretation> Iterator for TermsOf<'a, I> {
    type Item = TermOf<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ids
            .next()
            .map(|borrowed| match borrowed {
                super::id::BorrowedId::Iri(i) => BorrowedTerm::Iri(i),
                super::id::BorrowedId::BlankId(b) => BorrowedTerm::BlankId(b),
            })
            .or_else(|| self.literals.next().map(BorrowedTerm::Literal))
    }
}
