use crate::{BlankIdBuf, LocalTerm};

use super::LocalGenerator;

/// Error returned when a [`Blank`] generator is constructed with a prefix that
/// would yield an invalid [BLANK_NODE_LABEL][s] when combined with a numeric
/// suffix.
///
/// [s]: https://www.w3.org/TR/n-triples/#grammar-production-BLANK_NODE_LABEL
#[derive(Debug, thiserror::Error)]
#[error("invalid blank-id prefix: {0:?}")]
pub struct InvalidBlankPrefix(pub String);

/// Generates numbered blank node identifiers,
/// with an optional prefix.
///
/// This generator can create `usize::MAX` unique blank node identifiers.
/// If `next` is called `usize::MAX + 1` times, it will panic.
#[derive(Default)]
pub struct Blank {
    /// Prefix string.
    prefix: String,

    /// Number of already generated identifiers.
    count: usize,
}

impl Blank {
    /// Creates a new numbered generator with no prefix.
    pub const fn new() -> Self {
        Self {
            prefix: String::new(),
            count: 0,
        }
    }

    /// Creates a new numbered generator with no prefix,
    /// starting with the given `offset` number.
    ///
    /// The returned generator can create `usize::MAX - offset` unique blank node identifiers
    /// before panicking.
    pub const fn new_with_offset(offset: usize) -> Self {
        Self {
            prefix: String::new(),
            count: offset,
        }
    }

    /// Creates a new numbered generator with the given prefix.
    ///
    /// Returns [`InvalidBlankPrefix`] if `prefix` cannot form a valid
    /// [BLANK_NODE_LABEL][s] body when followed by digits.
    ///
    /// [s]: https://www.w3.org/TR/n-triples/#grammar-production-BLANK_NODE_LABEL
    pub fn new_with_prefix(prefix: impl Into<String>) -> Result<Self, InvalidBlankPrefix> {
        Self::new_full(prefix, 0)
    }

    /// Creates a new numbered generator with the given prefix and starting
    /// `offset`. Returns [`InvalidBlankPrefix`] on rejected prefix.
    ///
    /// The returned generator can create `usize::MAX - offset` unique blank node identifiers
    /// before panicking.
    pub fn new_full(prefix: impl Into<String>, offset: usize) -> Result<Self, InvalidBlankPrefix> {
        let prefix = prefix.into();
        validate_prefix(&prefix)?;
        Ok(Self { prefix, count: offset })
    }

    /// Creates a generator without checking the prefix.
    ///
    /// # Safety
    ///
    /// `prefix` followed by any decimal digit string must form a valid
    /// [BLANK_NODE_LABEL][s] body. Violating this lets [`Self::next_blank_id`]
    /// produce a structurally invalid [`BlankIdBuf`].
    ///
    /// [s]: https://www.w3.org/TR/n-triples/#grammar-production-BLANK_NODE_LABEL
    pub const unsafe fn new_full_unchecked(prefix: String, offset: usize) -> Self {
        Self { prefix, count: offset }
    }

    /// Returns the prefix of this generator.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Returns the number of already generated identifiers.
    pub const fn count(&self) -> usize {
        self.count
    }

    pub fn next_blank_id(&mut self) -> BlankIdBuf {
        let mut buf = itoa::Buffer::new();
        let count_str = buf.format(self.count);
        let mut s = String::with_capacity(2 + self.prefix.len() + count_str.len());
        s.push_str("_:");
        s.push_str(&self.prefix);
        s.push_str(count_str);
        // Safety: prefix validated at construction; count_str is always at
        // least one ASCII digit, satisfying the trailing PN_CHARS requirement.
        let id = unsafe { BlankIdBuf::new_unchecked(s) };
        self.count += 1;
        id
    }
}

fn validate_prefix(prefix: &str) -> Result<(), InvalidBlankPrefix> {
    if prefix.is_empty() {
        return Ok(());
    }
    // The probe `_:<prefix>0` exercises every grammar constraint: seed
    // (first char of prefix) must be PN_CHARS_U|[0-9], body chars must be
    // PN_CHARS|'.', and the trailing '0' satisfies the final PN_CHARS rule.
    let mut probe = String::with_capacity(2 + prefix.len() + 1);
    probe.push_str("_:");
    probe.push_str(prefix);
    probe.push('0');
    BlankIdBuf::new(probe).map(|_| ()).map_err(|_| InvalidBlankPrefix(prefix.to_owned()))
}

impl LocalGenerator for Blank {
    fn next_local_term(&mut self) -> LocalTerm {
        LocalTerm::BlankId(self.next_blank_id())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_prefix_generates_valid_ids() {
        let mut g = Blank::new();
        for _ in 0..10 {
            let id = g.next_blank_id();
            assert!(id.as_str().starts_with("_:"));
        }
    }

    #[test]
    fn valid_prefix_accepted() {
        let mut g = Blank::new_with_prefix("foo").unwrap();
        let id = g.next_blank_id();
        assert_eq!(id.as_str(), "_:foo0");
    }

    #[test]
    fn prefix_with_dot_in_middle_accepted() {
        let mut g = Blank::new_with_prefix("a.b").unwrap();
        let id = g.next_blank_id();
        assert_eq!(id.as_str(), "_:a.b0");
    }

    #[test]
    fn invalid_prefix_rejected_starts_with_dot() {
        assert!(Blank::new_with_prefix(".bad").is_err());
    }

    #[test]
    fn invalid_prefix_rejected_starts_with_hyphen() {
        assert!(Blank::new_with_prefix("-bad").is_err());
    }

    #[test]
    fn invalid_prefix_rejected_contains_space() {
        assert!(Blank::new_with_prefix("a b").is_err());
    }

    #[test]
    fn invalid_prefix_rejected_contains_special() {
        assert!(Blank::new_with_prefix("foo!").is_err());
    }
}
