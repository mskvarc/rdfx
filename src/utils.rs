use std::marker::PhantomData;

use crate::{IsGraph, IsObject, IsPredicate, IsSubject, Quad, Triple};

pub struct InfallibleIterator<I>(pub I);

impl<I: Iterator> Iterator for InfallibleIterator<I> {
    type Item = Result<I::Item, std::convert::Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Ok)
    }
}

pub struct TripleToQuadIterator<I, G>(I, PhantomData<G>);

impl<I, G> TripleToQuadIterator<I, G> {
    pub const fn new(inner: I) -> Self {
        Self(inner, PhantomData)
    }
}

impl<S, P, O, G, I> Iterator for TripleToQuadIterator<I, G>
where
    S: IsSubject,
    P: IsPredicate,
    O: IsObject,
    G: IsGraph,
    I: Iterator<Item = Triple<S, P, O>>,
{
    type Item = Quad<S, P, O, G>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|t| t.into_quad(None))
    }
}

pub struct OptionIterator<I>(pub Option<I>);

impl<I: Iterator> Iterator for OptionIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(I::next)
    }
}
