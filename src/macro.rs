#[macro_export]
#[doc(hidden)]
macro_rules! unexpected_token {
    () => {};
}

/// Creates a gRDF triple.
#[macro_export]
macro_rules! grdf_triple {
	// Parse a triple.
	{
		@from ($($acc:tt)*) $id:ident $($rest:tt)*
	} => {
		$crate::grdf_triple!(@from ($($acc)* ($crate::LocalTerm::from($id)),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
	} => {
		$crate::grdf_triple!(@from ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
	} => {
		$crate::grdf_triple!(@from ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
	} => {
		$crate::grdf_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::iri!($ty))
			)
		)),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal $($rest:tt)*
	} => {
		$crate::grdf_triple!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::XSD_STRING)
			)
		)),) $($rest)*)
	};
	{
		@from ($($acc:tt)*)
	} => {
		<$crate::Triple>::new($($acc)*)
	};
	{
		@from ($($acc:tt)*) $t:tt $($rest:tt)*
	} => {
		$crate::unexpected_token!($t)
	};
	// Main rule
	{
		$($t:tt)*
	} => {
		$crate::grdf_triple!(@from () $($t)*)
	};
}

/// Creates an array of triples.
#[macro_export]
macro_rules! grdf_triples {
	// Tokenize the triples.
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] []
	} => {
		$crate::grdf_triples!(@triples_from [] $($acc)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
	} => {
		$crate::unexpected_token!($t)
	};
	// Parse a tokenized triple list.
	{
		@triples_from [$($acc:tt)*] ($($triple:tt)*) $($rest:tt)*
	} => {
		$crate::grdf_triples!(@triples_from [$($acc)* $crate::grdf_triple!($($triple)*),] $($rest)*)
	};
	{
		@triples_from [$($acc:tt)*]
	} => {
		[$($acc)*]
	};

	// Main rule.
	{
		$($t:tt)*
	} => {
		$crate::grdf_triples!(@tokenize [] [] $($t)*)
	};
}

/// Creates a gRDF quad.
#[macro_export]
macro_rules! grdf_quad {
	// Parse a quad.
	{
		@from ($($acc:tt)*) $id:ident $($rest:tt)*
	} => {
		$crate::grdf_quad!(@from ($($acc)* ($crate::LocalTerm::from($id)),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
	} => {
		$crate::grdf_quad!(@from ($($acc)* $crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri))),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
	} => {
		$crate::grdf_quad!(@from ($($acc)* $crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap()),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
	} => {
		$crate::grdf_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::iri!($ty))
			)
		)),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal $($rest:tt)*
	} => {
		$crate::grdf_quad!(@from ($($acc)* $crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::XSD_STRING)
			)
		)),) $($rest)*)
	};
	{
		@from ($s:expr, $p:expr, $o:expr,)
	} => {
		<$crate::Quad>::new($s, $p, $o, None)
	};
	{
		@from ($s:expr, $p:expr, $o:expr, $g:expr,)
	} => {
		<$crate::Quad>::new($s, $p, $o, Some($g))
	};
	{
		@from ($($acc:tt)*) $t:tt $($rest:tt)*
	} => {
		$crate::unexpected_token!($t)
	};
	// Main rule
	{
		$($t:tt)*
	} => {
		$crate::grdf_quad!(@from () $($t)*)
	};
}

/// Creates a gRDF quad pattern.
///
/// The type of the returned value is `Quad<ResourceOrVar<Term, _>>`.
///
/// This is similar to [`grdf_quad`], but with the addition of variables, which
/// can be introduced using the syntax `?ident`. The variable `ident` must be
/// an identifier to some Rust variable holding the pattern variable's value.
#[macro_export]
macro_rules! grdf_quad_pattern {
	// Parse a quad pattern.
	{
		@from ($($acc:tt)*) ? $id:ident $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Var($id),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $id:ident $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($id),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) < $iri:literal > $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::iri($crate::IriBuf::from($crate::iri!($iri)))),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) _ : $id:literal $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::BlankId($crate::BlankIdBuf::from_suffix($id).unwrap())),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal ^^ $ty:literal $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::iri!($ty))
			)
		))),) $($rest)*)
	};
	{
		@from ($($acc:tt)*) $value:literal $($rest:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from ($($acc)* $crate::pattern::ResourceOrVar::Resource($crate::LocalTerm::literal($crate::Literal::new(
			$value.to_owned(),
			$crate::LiteralType::Any(
				$crate::IriBuf::from($crate::XSD_STRING)
			)
		))),) $($rest)*)
	};
	{
		@from ($s:expr, $p:expr, $o:expr,)
	} => {
		<$crate::Quad<$crate::pattern::ResourceOrVar<$crate::Term, _>>>::new($s, $p, $o, None)
	};
	{
		@from ($s:expr, $p:expr, $o:expr, $g:expr,)
	} => {
		<$crate::Quad<$crate::pattern::ResourceOrVar<$crate::Term, _>>>::new($s, $p, $o, Some($g))
	};
	{
		@from ($($acc:tt)*) $t:tt $($rest:tt)*
	} => {
		$crate::unexpected_token!($t)
	};
	// Main rule
	{
		$($t:tt)*
	} => {
		$crate::grdf_quad_pattern!(@from () $($t)*)
	};
}

/// Creates an array of quads.
#[macro_export]
macro_rules! grdf_quads {
	// Tokenize the quads.
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $i:ident $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* $i] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] < $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* <] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] > $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* >] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] _ $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* _] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] : $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* :] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] ^ $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* ^] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $l:literal $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)*] [$($current)* $l] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] . $($rest:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [$($acc)* ( $($current)* )] [] $($rest)*)
	};
	{
		@tokenize [$($acc:tt)*] []
	} => {
		$crate::grdf_quads!(@quads_from [] $($acc)*)
	};
	{
		@tokenize [$($acc:tt)*] [$($current:tt)*] $t:tt $($rest:tt)*
	} => {
		$crate::unexpected_token!($t)
	};
	// Parse a tokenized triple list.
	{
		@quads_from [$($acc:tt)*] ($($triple:tt)*) $($rest:tt)*
	} => {
		$crate::grdf_quads!(@quads_from [$($acc)* $crate::grdf_quad!($($triple)*),] $($rest)*)
	};
	{
		@quads_from [$($acc:tt)*]
	} => {
		[$($acc)*]
	};

	// Main rule.
	{
		$($t:tt)*
	} => {
		$crate::grdf_quads!(@tokenize [] [] $($t)*)
	};
}

// Macro tests will be re-added in Phase G after grdf_* are renamed to
// generalized_* (building GeneralizedTriple/GeneralizedQuad) and strict
// `triple!`/`quad!` macros are introduced.
