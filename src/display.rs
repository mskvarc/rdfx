use std::{fmt, ops::Deref};

use iri_rs::{Iri, IriRef};
use langtag::{LangTag, LangTagBuf};

/// Display method for RDF syntax elements.
pub trait RdfDisplay {
    /// Formats the value using the given formatter.
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;

    /// Prepare the value to be formatted as an RDF syntax element.
    #[inline(always)]
    fn rdf_display(&self) -> RdfDisplayed<&Self> {
        RdfDisplayed(self)
    }
}

impl RdfDisplay for str {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\"")?;

        let bytes = self.as_bytes();
        let mut start = 0;
        while start < bytes.len() {
            let rem = &bytes[start..];
            let p1 = memchr::memchr3(b'"', b'\\', b'\n', rem);
            let p2 = memchr::memchr(b'\r', rem);
            let pos = match (p1, p2) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            match pos {
                Some(off) => {
                    // Safe: split is on an ASCII boundary (the matched byte is one of " \ \n \r,
                    // all ASCII; the prefix [start..start+off] therefore ends at a UTF-8 boundary).
                    f.write_str(unsafe { std::str::from_utf8_unchecked(&rem[..off]) })?;
                    f.write_str(match rem[off] {
                        b'"' => "\\\"",
                        b'\\' => "\\\\",
                        b'\n' => "\\n",
                        b'\r' => "\\r",
                        _ => unreachable!(),
                    })?;
                    start += off + 1;
                }
                None => {
                    f.write_str(unsafe { std::str::from_utf8_unchecked(rem) })?;
                    break;
                }
            }
        }

        f.write_str("\"")
    }
}

impl RdfDisplay for String {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().rdf_fmt(f)
    }
}

impl<T: Deref<Target = str>> RdfDisplay for IriRef<T> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<")?;

        let s = self.as_str();
        let bytes = s.as_bytes();
        let mut start = 0;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b < 0x80 && IRI_ESCAPE[b as usize] {
                if i > start {
                    // Safe: split is on ASCII boundary; bytes[start..i] is valid UTF-8.
                    f.write_str(unsafe { std::str::from_utf8_unchecked(&bytes[start..i]) })?;
                }
                write!(f, "\\u{:#04x}", b as u32)?;
                i += 1;
                start = i;
            } else {
                i += 1;
            }
        }
        if start < bytes.len() {
            f.write_str(unsafe { std::str::from_utf8_unchecked(&bytes[start..]) })?;
        }

        f.write_str(">")
    }
}

const IRI_ESCAPE: [bool; 128] = {
    let mut t = [false; 128];
    let mut i = 0;
    while i <= 0x20 {
        t[i] = true;
        i += 1;
    }
    t[b'<' as usize] = true;
    t[b'>' as usize] = true;
    t[b'"' as usize] = true;
    t[b'{' as usize] = true;
    t[b'}' as usize] = true;
    t[b'|' as usize] = true;
    t[b'^' as usize] = true;
    t[b'`' as usize] = true;
    t[b'\\' as usize] = true;
    t
};

impl<T: Deref<Target = str>> RdfDisplay for Iri<T> {
    #[inline(always)]
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_iri_ref().rdf_fmt(f)
    }
}

#[cfg(feature = "contextual")]
impl<T: Deref<Target = str>, C: ?Sized> RdfDisplayWithContext<C> for Iri<T> {
    fn rdf_fmt_with(&self, _context: &C, f: &mut fmt::Formatter) -> fmt::Result {
        self.rdf_fmt(f)
    }
}

#[cfg(feature = "contextual")]
impl<T: Deref<Target = str>, C: ?Sized> RdfDisplayWithContext<C> for IriRef<T> {
    fn rdf_fmt_with(&self, _context: &C, f: &mut fmt::Formatter) -> fmt::Result {
        self.rdf_fmt(f)
    }
}

impl RdfDisplay for LangTag {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use fmt::Display;
        self.as_str().fmt(f)
    }
}

#[cfg(feature = "contextual")]
impl<C: ?Sized> RdfDisplayWithContext<C> for LangTag {
    fn rdf_fmt_with(&self, _context: &C, f: &mut fmt::Formatter) -> fmt::Result {
        self.rdf_fmt(f)
    }
}

impl RdfDisplay for LangTagBuf {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use fmt::Display;
        self.as_str().fmt(f)
    }
}

#[cfg(feature = "contextual")]
impl<C: ?Sized> RdfDisplayWithContext<C> for LangTagBuf {
    fn rdf_fmt_with(&self, _context: &C, f: &mut fmt::Formatter) -> fmt::Result {
        self.rdf_fmt(f)
    }
}

impl<T: RdfDisplay + ?Sized> RdfDisplay for &T {
    #[inline(always)]
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::rdf_fmt(*self, f)
    }
}

/// Value ready to be formatted as an RDF syntax element.
pub struct RdfDisplayed<T>(T);

impl<T: RdfDisplay> fmt::Display for RdfDisplayed<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.rdf_fmt(f)
    }
}

#[cfg(feature = "contextual")]
pub trait RdfDisplayWithContext<C: ?Sized> {
    fn rdf_fmt_with(&self, context: &C, f: &mut fmt::Formatter) -> fmt::Result;
}

#[cfg(feature = "contextual")]
impl<T: RdfDisplayWithContext<C> + ?Sized, C: ?Sized> RdfDisplayWithContext<C> for &T {
    #[inline(always)]
    fn rdf_fmt_with(&self, context: &C, f: &mut fmt::Formatter) -> fmt::Result {
        T::rdf_fmt_with(*self, context, f)
    }
}

#[cfg(feature = "contextual")]
impl<T: RdfDisplayWithContext<C>, C: ?Sized> RdfDisplay for contextual::Contextual<T, &C> {
    #[inline(always)]
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.rdf_fmt_with(self.1, f)
    }
}

#[cfg(feature = "contextual")]
impl<T: RdfDisplayWithContext<C>, C: ?Sized> RdfDisplay for contextual::Contextual<T, &mut C> {
    #[inline(always)]
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.rdf_fmt_with(self.1, f)
    }
}

#[cfg(test)]
mod tests {
    use super::RdfDisplay;
    use iri_rs::IriRef;

    fn render(s: &str) -> String {
        let iri = IriRef::parse_unchecked(s);
        format!("{}", iri.rdf_display())
    }

    #[test]
    fn clean_ascii() {
        assert_eq!(render("http://example.org/foo"), "<http://example.org/foo>");
    }

    #[test]
    fn escapes_each_special() {
        assert_eq!(render("a<b"), "<a\\u0x3cb>");
        assert_eq!(render("a>b"), "<a\\u0x3eb>");
        assert_eq!(render("a\"b"), "<a\\u0x22b>");
        assert_eq!(render("a{b"), "<a\\u0x7bb>");
        assert_eq!(render("a}b"), "<a\\u0x7db>");
        assert_eq!(render("a|b"), "<a\\u0x7cb>");
        assert_eq!(render("a^b"), "<a\\u0x5eb>");
        assert_eq!(render("a`b"), "<a\\u0x60b>");
        assert_eq!(render("a\\b"), "<a\\u0x5cb>");
        assert_eq!(render("a b"), "<a\\u0x20b>");
        assert_eq!(render("a\tb"), "<a\\u0x09b>");
        assert_eq!(render("a\0b"), "<a\\u0x00b>");
        assert_eq!(render("a\x1fb"), "<a\\u0x1fb>");
    }

    #[test]
    fn passes_through_high_bytes() {
        assert_eq!(render("aéb"), "<aéb>");
        assert_eq!(render("a中b"), "<a中b>");
        assert_eq!(render("a🦀b"), "<a🦀b>");
    }

    #[test]
    fn length_sweep_clean() {
        for &n in &[7usize, 8, 9, 15, 16, 17, 64, 65, 256] {
            let body = "a".repeat(n);
            assert_eq!(render(&body), format!("<{body}>"), "len {n}");
        }
    }

    #[test]
    fn escape_at_every_alignment() {
        for &i in &[0usize, 7, 8, 9, 15, 16, 17] {
            let len = i + 8;
            let mut body: String = "a".repeat(len);
            body.replace_range(i..i + 1, "<");
            let mut expected = String::from("<");
            expected.push_str(&body[..i]);
            expected.push_str("\\u0x3c");
            expected.push_str(&body[i + 1..]);
            expected.push('>');
            assert_eq!(render(&body), expected, "escape at index {i}");
        }
    }

    #[test]
    fn all_escapes_back_to_back() {
        let body = "<>\"{|}^`\\";
        let expected = "<\\u0x3c\\u0x3e\\u0x22\\u0x7b\\u0x7c\\u0x7d\\u0x5e\\u0x60\\u0x5c>";
        assert_eq!(render(body), expected);
    }
}
