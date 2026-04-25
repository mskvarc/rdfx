use std::borrow::Cow;

use iri_rs::IriBuf;

use crate::BlankId;

pub enum CowId<'a> {
    BlankId(Cow<'a, BlankId>),
    Iri(Cow<'a, IriBuf>),
}
