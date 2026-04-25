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

/// Precomputed `\uXX` escape strings for every byte. Six bytes each:
/// `\`, `u`, `0`, `x`, hex_hi, hex_lo (lowercase). Bypasses `write!` macro
/// dispatch — escape emission becomes a single `f.write_str` of fixed bytes.
const IRI_ESC: [[u8; 6]; 256] = {
    let mut t = [[0u8; 6]; 256];
    let mut i = 0usize;
    while i < 256 {
        let hi = ((i >> 4) & 0xF) as u8;
        let lo = (i & 0xF) as u8;
        t[i][0] = b'\\';
        t[i][1] = b'u';
        t[i][2] = b'0';
        t[i][3] = b'x';
        t[i][4] = if hi < 10 { b'0' + hi } else { b'a' + hi - 10 };
        t[i][5] = if lo < 10 { b'0' + lo } else { b'a' + lo - 10 };
        i += 1;
    }
    t
};

#[inline]
fn iri_emit_escapes(
    f: &mut fmt::Formatter,
    mut mask: u64,
    base: usize,
    bytes: &[u8],
    start: &mut usize,
) -> fmt::Result {
    while mask != 0 {
        let pos = base + crate::swar::first_set_byte_index(mask) as usize;
        if pos > *start {
            // Safe: split is on ASCII boundary; bytes[*start..pos] is valid UTF-8.
            f.write_str(unsafe { std::str::from_utf8_unchecked(&bytes[*start..pos]) })?;
        }
        // Safe: IRI_ESC entries are six ASCII bytes.
        f.write_str(unsafe { std::str::from_utf8_unchecked(&IRI_ESC[bytes[pos] as usize]) })?;
        *start = pos + 1;
        mask &= mask - 1;
    }
    Ok(())
}

impl<T: Deref<Target = str>> RdfDisplay for IriRef<T> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<")?;

        let s = self.as_str();
        let bytes = s.as_bytes();
        let mut i = 0;
        let mut start = 0;

        while i + 8 <= bytes.len() {
            let mask = crate::swar::iri_escape_word(crate::swar::load_u64(bytes, i));
            if mask != 0 {
                iri_emit_escapes(f, mask, i, bytes, &mut start)?;
            }
            i += 8;
        }

        let tail_len = bytes.len() - i;
        if tail_len > 0 {
            let mut arr = [0u8; 8];
            arr[..tail_len].copy_from_slice(&bytes[i..]);
            let w = u64::from_le_bytes(arr);
            // Padding bytes (0x00 < 0x21) appear as escapes; mask off positions >= tail_len.
            let len_mask = (1u64 << (tail_len * 8)) - 1;
            let mask = crate::swar::iri_escape_word(w) & len_mask;
            if mask != 0 {
                iri_emit_escapes(f, mask, i, bytes, &mut start)?;
            }
        }

        if start < bytes.len() {
            f.write_str(unsafe { std::str::from_utf8_unchecked(&bytes[start..]) })?;
        }

        f.write_str(">")
    }
}

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
