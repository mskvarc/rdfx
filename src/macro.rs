#[macro_export]
#[doc(hidden)]
macro_rules! unexpected_token {
    () => {};
}

// ==========================================================================
// Generalized RDF macros (RDF 1.1 §A / RDF 1.2 §A.1) — no position bounds.
//
// `generalized_triple!` builds `GeneralizedTriple<LocalTerm>`. Use these for
// generalized RDF where literals/blanks may appear in any position. RDF 1.2
// triple terms `<<( s p o )>>` are accepted in any position; their body is
// strict (built via `triple!`).
// ==========================================================================

/// Creates a generalized RDF triple ([`GeneralizedTriple<LocalTerm>`](crate::GeneralizedTriple)).
#[macro_export]
macro_rules! generalized_triple {
    {
        @from ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::triple($crate::triple!($($body)*)),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $id:ident $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* ($crate::LocalTerm::from($id)),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal $($rest:tt)*
    } => {
        $crate::generalized_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::plain($value.to_owned())),) $($rest)*)
    };
    {
        @from ($($acc:tt)*)
    } => {
        <$crate::GeneralizedTriple>::new($($acc)*)
    };
    {
        @from ($($acc:tt)*) $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        $($t:tt)*
    } => {
        $crate::generalized_triple!(@from () $($t)*)
    };
}

/// Creates an array of generalized triples.
#[macro_export]
macro_rules! generalized_triples {
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] << $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* <<] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] >> $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* >>] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] @ $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* @] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] / $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* /] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ( $($body:tt)* ) $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* ( $($body)* )] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] []
    } => {
        $crate::generalized_triples!(@triples_from [] $($acc)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        @triples_from [$($acc:tt)*] ($($triple:tt)*) $($rest:tt)*
    } => {
        $crate::generalized_triples!(@triples_from [$($acc)* $crate::generalized_triple!($($triple)*),] $($rest)*)
    };
    {
        @triples_from [$($acc:tt)*]
    } => {
        [$($acc)*]
    };
    {
        $($t:tt)*
    } => {
        $crate::generalized_triples!(@tokenize [] [] $($t)*)
    };
}

/// Creates a generalized RDF quad ([`GeneralizedQuad<LocalTerm>`](crate::GeneralizedQuad)).
#[macro_export]
macro_rules! generalized_quad {
    {
        @from ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::triple($crate::triple!($($body)*)),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $id:ident $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* ($crate::LocalTerm::from($id)),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        )),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal $($rest:tt)*
    } => {
        $crate::generalized_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::plain($value.to_owned())),) $($rest)*)
    };
    {
        @from ($s:expr, $p:expr, $o:expr,)
    } => {
        <$crate::GeneralizedQuad>::new($s, $p, $o, None)
    };
    {
        @from ($s:expr, $p:expr, $o:expr, $g:expr,)
    } => {
        <$crate::GeneralizedQuad>::new($s, $p, $o, Some($g))
    };
    {
        @from ($($acc:tt)*) $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        $($t:tt)*
    } => {
        $crate::generalized_quad!(@from () $($t)*)
    };
}

/// Creates a generalized RDF quad pattern.
///
/// Variables `?ident` are introduced via [`ResourceOrVar::Var`](crate::pattern::ResourceOrVar::Var);
/// other terms via [`ResourceOrVar::Resource`](crate::pattern::ResourceOrVar::Resource).
#[macro_export]
macro_rules! generalized_quad_pattern {
    {
        @from ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::triple($crate::triple!($($body)*))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) ? $id:ident $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $id:ident $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($id),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri)))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap())),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        ))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        ))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal @ $lang:literal $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        ))),) $($rest)*)
    };
    {
        @from ($($acc:tt)*) $value:literal $($rest:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::plain($value.to_owned()))),) $($rest)*)
    };
    {
        @from ($s:expr, $p:expr, $o:expr,)
    } => {
        <$crate::GeneralizedQuad<$crate::pattern::ResourceOrVar<$crate::LocalTerm, _>>>::new($s, $p, $o, None)
    };
    {
        @from ($s:expr, $p:expr, $o:expr, $g:expr,)
    } => {
        <$crate::GeneralizedQuad<$crate::pattern::ResourceOrVar<$crate::LocalTerm, _>>>::new($s, $p, $o, Some($g))
    };
    {
        @from ($($acc:tt)*) $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        $($t:tt)*
    } => {
        $crate::generalized_quad_pattern!(@from () $($t)*)
    };
}

/// Creates an array of generalized quads.
#[macro_export]
macro_rules! generalized_quads {
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] << $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* <<] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] >> $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* >>] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] @ $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* @] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] / $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* /] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ( $($body:tt)* ) $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* ( $($body)* )] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] []
    } => {
        $crate::generalized_quads!(@quads_from [] $($acc)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        @quads_from [$($acc:tt)*] ($($triple:tt)*) $($rest:tt)*
    } => {
        $crate::generalized_quads!(@quads_from [$($acc)* $crate::generalized_quad!($($triple)*),] $($rest)*)
    };
    {
        @quads_from [$($acc:tt)*]
    } => {
        [$($acc)*]
    };
    {
        $($t:tt)*
    } => {
        $crate::generalized_quads!(@tokenize [] [] $($t)*)
    };
}

// ==========================================================================
// Strict RDF 1.1 / 1.2 macros — position-aware, reject invalid forms at compile time.
//
// `triple!` builds `Triple<Id, IriBuf, LocalTerm>` (= `LexicalTriple`).
// Subjects: `<iri>` or `_:label`. Predicates: `<iri>` only. Objects: any term,
// including RDF 1.2 triple terms `<<( s p o )>>` and directional language
// strings `"v" @ "lang" / "ltr"|"rtl"`.
// `quad!` adds a graph-name position accepting IRI or blank.
// ==========================================================================

/// Creates a strict RDF 1.1 / 1.2 triple ([`Triple<Id, IriBuf, LocalTerm>`](crate::Triple)).
///
/// Subjects must be IRIs or blank nodes; predicates must be IRIs; objects may
/// be IRIs, blank nodes, literals, or RDF 1.2 triple terms `<<( s p o )>>`.
/// Literal subjects, literal predicates, blank predicates, and triple terms
/// outside object position are rejected at compile time.
#[macro_export]
macro_rules! triple {
    // --- subject ---
    { @sub ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF; use generalized_triple! for unrestricted positions")
    };
    { @sub ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::triple!(@pred ($($acc)* $crate::Id::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    { @sub ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::triple!(@pred ($($acc)* $crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
    };
    { @sub ($($acc:tt)*) $id:ident $($rest:tt)* } => {
        $crate::triple!(@pred ($($acc)* $id,) $($rest)*)
    };
    { @sub ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a typed literal")
    };
    { @sub ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a literal")
    };

    // --- predicate ---
    { @pred ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF; use generalized_triple! for unrestricted positions")
    };
    { @pred ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::triple!(@obj ($($acc)* $crate::IriBuf::from($crate::iri!($iri)),) $($rest)*)
    };
    { @pred ($($acc:tt)*) $id:ident $($rest:tt)* } => {
        $crate::triple!(@obj ($($acc)* $id,) $($rest)*)
    };
    { @pred ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a blank node")
    };
    { @pred ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a typed literal")
    };
    { @pred ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a literal")
    };

    // --- object (terminal) ---
    { @obj ($($acc:tt)*) << ( $($body:tt)* ) >> } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::triple($crate::triple!($($body)*)),))
    };
    { @obj ($($acc:tt)*) < $iri:literal > } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),))
    };
    { @obj ($($acc:tt)*) _ : $id:literal } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),))
    };
    { @obj ($($acc:tt)*) $value:literal ^^ $ty:literal } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        )),))
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        )),))
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        )),))
    };
    { @obj ($($acc:tt)*) $value:literal } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::literal($crate::Literal::plain($value.to_owned())),))
    };
    { @obj ($($acc:tt)*) $id:ident } => {
        $crate::triple!(@end ($($acc)* $crate::LocalTerm::from($id),))
    };

    { @end ($s:expr, $p:expr, $o:expr,) } => {
        <$crate::Triple<$crate::Id, $crate::IriBuf, $crate::LocalTerm>>::new($s, $p, $o)
    };

    { $($t:tt)* } => {
        $crate::triple!(@sub () $($t)*)
    };
}

/// Creates a strict RDF 1.1 / 1.2 quad ([`Quad<Id, IriBuf, LocalTerm, Id>`](crate::Quad)).
///
/// Same shape as [`triple!`] with an optional graph-name (IRI or blank node).
#[macro_export]
macro_rules! quad {
    // --- subject ---
    { @sub ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF; use generalized_quad! for unrestricted positions")
    };
    { @sub ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad!(@pred ($($acc)* $crate::Id::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    { @sub ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::quad!(@pred ($($acc)* $crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
    };
    { @sub ($($acc:tt)*) $id:ident $($rest:tt)* } => {
        $crate::quad!(@pred ($($acc)* $id,) $($rest)*)
    };
    { @sub ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a typed literal")
    };
    { @sub ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a literal")
    };

    // --- predicate ---
    { @pred ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF; use generalized_quad! for unrestricted positions")
    };
    { @pred ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad!(@obj ($($acc)* $crate::IriBuf::from($crate::iri!($iri)),) $($rest)*)
    };
    { @pred ($($acc:tt)*) $id:ident $($rest:tt)* } => {
        $crate::quad!(@obj ($($acc)* $id,) $($rest)*)
    };
    { @pred ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a blank node")
    };
    { @pred ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a typed literal")
    };
    { @pred ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a literal")
    };

    // --- object (then maybe graph) ---
    { @obj ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::triple($crate::triple!($($body)*)),) $($rest)*)
    };
    { @obj ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        )),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        )),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        )),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::literal($crate::Literal::plain($value.to_owned())),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $id:ident $($rest:tt)* } => {
        $crate::quad!(@graph ($($acc)* $crate::LocalTerm::from($id),) $($rest)*)
    };

    // --- graph (optional) ---
    { @graph ($($acc:tt)*) << ( $($body:tt)* ) >> } => {
        ::std::compile_error!("triple terms (RDF 1.2) are not allowed in graph-name position in strict RDF; use generalized_quad! for unrestricted positions")
    };
    { @graph ($($acc:tt)*) < $iri:literal > } => {
        $crate::quad!(@end_some ($($acc)* $crate::Id::iri($crate::IriBuf::from($crate::iri!($iri))),))
    };
    { @graph ($($acc:tt)*) _ : $id:literal } => {
        $crate::quad!(@end_some ($($acc)* $crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap()),))
    };
    { @graph ($($acc:tt)*) $id:ident } => {
        $crate::quad!(@end_some ($($acc)* $id,))
    };
    { @graph ($($acc:tt)*) $value:literal ^^ $ty:literal } => {
        ::std::compile_error!("graph name must be IRI or blank node, not a typed literal")
    };
    { @graph ($($acc:tt)*) $value:literal } => {
        ::std::compile_error!("graph name must be IRI or blank node, not a literal")
    };
    { @graph ($($acc:tt)*) } => {
        $crate::quad!(@end_none ($($acc)*))
    };

    { @end_some ($s:expr, $p:expr, $o:expr, $g:expr,) } => {
        <$crate::Quad<$crate::Id, $crate::IriBuf, $crate::LocalTerm, $crate::Id>>::new($s, $p, $o, Some($g))
    };
    { @end_none ($s:expr, $p:expr, $o:expr,) } => {
        <$crate::Quad<$crate::Id, $crate::IriBuf, $crate::LocalTerm, $crate::Id>>::new($s, $p, $o, None)
    };

    { $($t:tt)* } => {
        $crate::quad!(@sub () $($t)*)
    };
}

/// Creates an array of strict RDF 1.1 / 1.2 triples.
#[macro_export]
macro_rules! triples {
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] << $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* <<] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] >> $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* >>] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] @ $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* @] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] / $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* /] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ( $($body:tt)* ) $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* ( $($body)* )] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
    } => {
        $crate::triples!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] []
    } => {
        $crate::triples!(@triples_from [] $($acc)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        @triples_from [$($acc:tt)*] ($($triple:tt)*) $($rest:tt)*
    } => {
        $crate::triples!(@triples_from [$($acc)* $crate::triple!($($triple)*),] $($rest)*)
    };
    {
        @triples_from [$($acc:tt)*]
    } => {
        [$($acc)*]
    };
    {
        $($t:tt)*
    } => {
        $crate::triples!(@tokenize [] [] $($t)*)
    };
}

/// Creates an array of strict RDF 1.1 / 1.2 quads.
#[macro_export]
macro_rules! quads {
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] << $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* <<] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] >> $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* >>] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] @ $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* @] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] / $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* /] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] ( $($body:tt)* ) $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* ( $($body)* )] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
    } => {
        $crate::quads!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
    };
    {
        @tokenize [$($acc:tt)*] []
    } => {
        $crate::quads!(@quads_from [] $($acc)*)
    };
    {
        @tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
    } => {
        $crate::unexpected_token!($t)
    };
    {
        @quads_from [$($acc:tt)*] ($($quad:tt)*) $($rest:tt)*
    } => {
        $crate::quads!(@quads_from [$($acc)* $crate::quad!($($quad)*),] $($rest)*)
    };
    {
        @quads_from [$($acc:tt)*]
    } => {
        [$($acc)*]
    };
    {
        $($t:tt)*
    } => {
        $crate::quads!(@tokenize [] [] $($t)*)
    };
}

/// Creates a strict triple pattern `Triple<ResourceOrVar<Id, _>, ResourceOrVar<IriBuf, _>, ResourceOrVar<LocalTerm, _>>`.
///
/// Position-aware: subjects are IRI/blank wrapped as `Id`; predicates are IRIs
/// wrapped as `IriBuf`; objects are arbitrary terms wrapped as `LocalTerm` —
/// including RDF 1.2 triple terms `<<( s p o )>>`.
/// Variables are introduced with `?ident`.
#[macro_export]
macro_rules! triple_pattern {
    // subject
    { @sub ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF")
    };
    { @sub ($($acc:tt)*) ? $id:ident $($rest:tt)* } => {
        $crate::triple_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    { @sub ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::triple_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::iri($crate::IriBuf::from($crate::iri!($iri)))),) $($rest)*)
    };
    { @sub ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::triple_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap())),) $($rest)*)
    };
    { @sub ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a typed literal")
    };
    { @sub ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a literal")
    };

    // predicate
    { @pred ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF")
    };
    { @pred ($($acc:tt)*) ? $id:ident $($rest:tt)* } => {
        $crate::triple_pattern!(@obj ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    { @pred ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::triple_pattern!(@obj ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    { @pred ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a blank node")
    };
    { @pred ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a typed literal")
    };
    { @pred ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a literal")
    };

    // object
    { @obj ($($acc:tt)*) << ( $($body:tt)* ) >> } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::triple($crate::triple!($($body)*))),))
    };
    { @obj ($($acc:tt)*) ? $id:ident } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Var($id),))
    };
    { @obj ($($acc:tt)*) < $iri:literal > } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri)))),))
    };
    { @obj ($($acc:tt)*) _ : $id:literal } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap())),))
    };
    { @obj ($($acc:tt)*) $value:literal ^^ $ty:literal } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        ))),))
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        ))),))
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        ))),))
    };
    { @obj ($($acc:tt)*) $value:literal } => {
        $crate::triple_pattern!(@end ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::plain($value.to_owned()))),))
    };

    { @end ($s:expr, $p:expr, $o:expr,) } => {
        <$crate::Triple<
            $crate::pattern::ResourceOrVar<$crate::Id, _>,
            $crate::pattern::ResourceOrVar<$crate::IriBuf, _>,
            $crate::pattern::ResourceOrVar<$crate::LocalTerm, _>,
        >>::new($s, $p, $o)
    };

    { $($t:tt)* } => {
        $crate::triple_pattern!(@sub () $($t)*)
    };
}

/// Creates a strict quad pattern.
#[macro_export]
macro_rules! quad_pattern {
    // subject
    { @sub ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF")
    };
    { @sub ($($acc:tt)*) ? $id:ident $($rest:tt)* } => {
        $crate::quad_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    { @sub ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::iri($crate::IriBuf::from($crate::iri!($iri)))),) $($rest)*)
    };
    { @sub ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@pred ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap())),) $($rest)*)
    };
    { @sub ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a typed literal")
    };
    { @sub ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("subject must be IRI or blank node, not a literal")
    };

    // predicate
    { @pred ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        ::std::compile_error!("triple terms (RDF 1.2) are only allowed in object position in strict RDF")
    };
    { @pred ($($acc:tt)*) ? $id:ident $($rest:tt)* } => {
        $crate::quad_pattern!(@obj ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    { @pred ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad_pattern!(@obj ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
    };
    { @pred ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a blank node")
    };
    { @pred ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a typed literal")
    };
    { @pred ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        ::std::compile_error!("predicate must be IRI, not a literal")
    };

    // object
    { @obj ($($acc:tt)*) << ( $($body:tt)* ) >> $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::triple($crate::triple!($($body)*))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) ? $id:ident $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
    };
    { @obj ($($acc:tt)*) < $iri:literal > $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri)))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) _ : $id:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap())),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::typed(
            $value.to_owned(),
            $crate::Datatype::new($crate::IriBuf::from($crate::iri!($ty))).expect("compile-time IRI is not a reserved language-string datatype")
        ))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal / $dir:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::dir_lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47"),
            { const __DIR: $crate::Direction = match $crate::Direction::parse_const($dir) { Some(d) => d, None => panic!("invalid direction literal: must be \"ltr\" or \"rtl\"") }; __DIR }
        ))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal @ $lang:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::lang_string(
            $value.to_owned(),
            <$crate::langtag::LangTagBuf as ::std::str::FromStr>::from_str($lang).expect("compile-time lang tag is well-formed BCP47")
        ))),) $($rest)*)
    };
    { @obj ($($acc:tt)*) $value:literal $($rest:tt)* } => {
        $crate::quad_pattern!(@graph ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::plain($value.to_owned()))),) $($rest)*)
    };

    // graph (optional)
    { @graph ($($acc:tt)*) << ( $($body:tt)* ) >> } => {
        ::std::compile_error!("triple terms (RDF 1.2) are not allowed in graph-name position in strict RDF")
    };
    { @graph ($($acc:tt)*) ? $id:ident } => {
        $crate::quad_pattern!(@end_some ($($acc)* $crate::pattern::ResourceOrVar::Var($id),))
    };
    { @graph ($($acc:tt)*) < $iri:literal > } => {
        $crate::quad_pattern!(@end_some ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::iri($crate::IriBuf::from($crate::iri!($iri)))),))
    };
    { @graph ($($acc:tt)*) _ : $id:literal } => {
        $crate::quad_pattern!(@end_some ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::Id::blank_id($crate::BlankIdBuf::from_suffix($id).unwrap())),))
    };
    { @graph ($($acc:tt)*) $value:literal ^^ $ty:literal } => {
        ::std::compile_error!("graph name must be IRI or blank node, not a typed literal")
    };
    { @graph ($($acc:tt)*) $value:literal } => {
        ::std::compile_error!("graph name must be IRI or blank node, not a literal")
    };
    { @graph ($($acc:tt)*) } => {
        $crate::quad_pattern!(@end_none ($($acc)*))
    };

    { @end_some ($s:expr, $p:expr, $o:expr, $g:expr,) } => {
        <$crate::Quad<
            $crate::pattern::ResourceOrVar<$crate::Id, _>,
            $crate::pattern::ResourceOrVar<$crate::IriBuf, _>,
            $crate::pattern::ResourceOrVar<$crate::LocalTerm, _>,
            $crate::pattern::ResourceOrVar<$crate::Id, _>,
        >>::new($s, $p, $o, Some($g))
    };
    { @end_none ($s:expr, $p:expr, $o:expr,) } => {
        <$crate::Quad<
            $crate::pattern::ResourceOrVar<$crate::Id, _>,
            $crate::pattern::ResourceOrVar<$crate::IriBuf, _>,
            $crate::pattern::ResourceOrVar<$crate::LocalTerm, _>,
            $crate::pattern::ResourceOrVar<$crate::Id, _>,
        >>::new($s, $p, $o, None)
    };

    { $($t:tt)* } => {
        $crate::quad_pattern!(@sub () $($t)*)
    };
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use iri_rs::iri;

    #[test]
    fn generalized_triple_macro() {
        let _ = generalized_triple! [
            _:"foo" <"https://example.org/#foo"> "foo"
        ];
    }

    #[test]
    fn generalized_triples_macro() {
        let term = iri_rs::IriBuf::from(iri!("https://example.org/#iri"));
        let _ = generalized_triples! [
            _:"foo" <"https://example.org/#foo"> "foo" .
            <"https://example.org/#bar"> _:"bar" "bar"^^"https://example.org/#datatype" .
            <"https://example.org/#baz"> term "value" .
        ];
    }

    #[test]
    fn generalized_quad_macro() {
        let _ = generalized_quad! [
            _:"foo" <"https://example.org/#foo"> "foo"
        ];
    }

    #[test]
    fn generalized_quads_macro() {
        let term = iri_rs::IriBuf::from(iri!("https://example.org/#iri"));
        let _ = generalized_quads! [
            _:"foo" <"https://example.org/#foo"> "foo" .
            <"https://example.org/#bar"> _:"bar" "bar"^^"https://example.org/#datatype" .
            <"https://example.org/#baz"> term "value" <"https://example.org/#graph"> .
        ];
    }

    #[test]
    fn strict_triple_macro() {
        let _ = triple! [ <"https://example.org/s"> <"https://example.org/p"> <"https://example.org/o"> ];
        let _ = triple! [ _:"b0" <"https://example.org/p"> "lit" ];
        let _ = triple! [ _:"b0" <"https://example.org/p"> "lit"^^"https://example.org/dt" ];
    }

    #[test]
    fn strict_quad_macro() {
        let _ = quad! [ <"https://example.org/s"> <"https://example.org/p"> <"https://example.org/o"> ];
        let _ = quad! [ _:"b0" <"https://example.org/p"> "lit" <"https://example.org/g"> ];
    }

    #[test]
    fn strict_triples_macro() {
        let _ = triples! [
            _:"b0" <"https://example.org/p"> "v" .
            <"https://example.org/s"> <"https://example.org/p"> _:"b1" .
        ];
    }

    #[test]
    fn strict_quads_macro() {
        let _ = quads! [
            _:"b0" <"https://example.org/p"> "v" .
            <"https://example.org/s"> <"https://example.org/p"> _:"b1" <"https://example.org/g"> .
        ];
    }

    // ---- RDF 1.2 smoke tests ----

    #[test]
    fn strict_triple_with_triple_term_object() {
        let t = triple! [
            <"https://example.org/a"> <"https://example.org/says">
                <<( <"https://example.org/s"> <"https://example.org/p"> <"https://example.org/o"> )>>
        ];
        assert!(t.object().is_triple());
    }

    #[test]
    fn strict_triple_nested_triple_term_object() {
        let t = triple! [
            <"https://example.org/a"> <"https://example.org/says">
                <<(
                    <"https://example.org/b"> <"https://example.org/says">
                    <<( <"https://example.org/s"> <"https://example.org/p"> <"https://example.org/o"> )>>
                )>>
        ];
        assert!(t.object().is_triple());
    }

    #[test]
    fn strict_triple_dir_lang_string_object() {
        let t = triple! [
            <"https://example.org/s"> <"https://example.org/p"> "hello" @ "en" / "ltr"
        ];
        let lit = t.object().as_literal().expect("dir-lang object");
        assert!(lit.type_.is_dir_lang_string());
        assert_eq!(lit.type_.direction(), Some(crate::Direction::Ltr));
    }

    #[test]
    fn generalized_triple_literal_subject_inside_triple_term() {
        // `generalized_triple!` accepts triple terms in any position; the
        // *body* is built via strict `triple!`, so this exercises the strict
        // body path, not a fully-generalized inner body.
        let _ = generalized_triple! [
            <"https://example.org/a"> <"https://example.org/p">
                <<( <"https://example.org/s"> <"https://example.org/p"> "literal-object" )>>
        ];
    }

    #[test]
    fn triples_array_with_triple_term() {
        let _ = triples! [
            <"https://example.org/a"> <"https://example.org/says">
                <<( <"https://example.org/s"> <"https://example.org/p"> <"https://example.org/o"> )>> .
            <"https://example.org/b"> <"https://example.org/p"> "x" .
        ];
    }
}
