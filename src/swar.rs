//! Portable SWAR primitives for ASCII byte-classification hot paths.
//!
//! Operates on `u64` chunks loaded as little-endian. Bit-position math
//! (`trailing_zeros() >> 3`) is endian-independent because LE load fixes
//! the byte ordering in the word.

const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
const ONES: u64 = 0x0101_0101_0101_0101;

#[inline(always)]
pub(crate) const fn broadcast(b: u8) -> u64 {
    (b as u64).wrapping_mul(ONES)
}

#[inline(always)]
pub(crate) const fn high_bit_set(w: u64) -> u64 {
    w & HIGH_BITS
}

#[inline(always)]
pub(crate) const fn has_byte_eq(w: u64, b: u8) -> u64 {
    // Per-byte correct equality: high bit set iff `w_byte == b`. Works for all byte values.
    // Saturating zero-detect avoids cross-byte borrow propagation that corrupts the
    // naive `(x - ONES) & !x & HIGH_BITS` formula when adjacent bytes match.
    let diff = w ^ broadcast(b);
    let y = (diff | HIGH_BITS).wrapping_sub(ONES);
    !(y | diff) & HIGH_BITS
}

#[inline(always)]
pub(crate) const fn has_byte_lt(w: u64, n: u8) -> u64 {
    // Per-byte correct `b < n`, requires `n <= 0x80`. Forces each byte's high bit
    // before subtraction so per-byte arithmetic stays within the byte (no inter-byte
    // borrow). The `!w` filter discards bytes whose original high bit was set
    // (those are >= 0x80 >= n, so never `< n`).
    let r = (w | HIGH_BITS).wrapping_sub(broadcast(n));
    !r & !w & HIGH_BITS
}

#[inline(always)]
pub(crate) const fn has_byte_in_range(w: u64, lo: u8, hi: u8) -> u64 {
    has_byte_lt(w, hi + 1) & !has_byte_lt(w, lo)
}

#[inline(always)]
pub(crate) const fn first_set_byte_index(mask: u64) -> u32 {
    mask.trailing_zeros() >> 3
}

const ASCII_BLANKID_VALID: [bool; 128] = {
    let mut t = [false; 128];
    let mut i = b'A';
    while i <= b'Z' {
        t[i as usize] = true;
        i += 1;
    }
    let mut i = b'a';
    while i <= b'z' {
        t[i as usize] = true;
        i += 1;
    }
    let mut i = b'0';
    while i <= b'9' {
        t[i as usize] = true;
        i += 1;
    }
    t[b'_' as usize] = true;
    t[b':' as usize] = true;
    t[b'.' as usize] = true;
    t[b'-' as usize] = true;
    t
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BodyScan {
    AllValidAscii,
    InvalidAt(usize),
    NonAsciiAt(usize),
}

#[inline(always)]
pub(crate) fn load_u64(bytes: &[u8], i: usize) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[i..i + 8]);
    u64::from_le_bytes(arr)
}

#[inline(always)]
const fn blankid_invalid_mask(w: u64) -> u64 {
    let lower = w | broadcast(0x20);
    let is_alpha = has_byte_in_range(lower, b'a', b'z');
    let is_digit = has_byte_in_range(w, b'0', b'9');
    let is_dot = has_byte_eq(w, b'.');
    let is_minus = has_byte_eq(w, b'-');
    let is_under = has_byte_eq(w, b'_');
    let is_colon = has_byte_eq(w, b':');
    let valid = is_alpha | is_digit | is_dot | is_minus | is_under | is_colon;
    !valid & HIGH_BITS
}

pub(crate) fn blankid_body_scan(body: &[u8]) -> BodyScan {
    let mut i = 0;
    while i + 8 <= body.len() {
        let w = load_u64(body, i);
        let hi = high_bit_set(w);
        if hi != 0 {
            return BodyScan::NonAsciiAt(i + first_set_byte_index(hi) as usize);
        }
        let invalid = blankid_invalid_mask(w);
        if invalid != 0 {
            return BodyScan::InvalidAt(i + first_set_byte_index(invalid) as usize);
        }
        i += 8;
    }
    while i < body.len() {
        let b = body[i];
        if b >= 0x80 {
            return BodyScan::NonAsciiAt(i);
        }
        if !ASCII_BLANKID_VALID[b as usize] {
            return BodyScan::InvalidAt(i);
        }
        i += 1;
    }
    BodyScan::AllValidAscii
}

/// Per-word IRI escape mask: high bit set per byte that needs to be escaped
/// (controls + space `0x00..=0x20`, plus `"`, `<`, `>`, `\\`, `^`, `` ` ``,
/// `{`, `|`, `}`). High-bit input bytes (UTF-8 multi-byte) are never escapes.
#[inline(always)]
pub(crate) const fn iri_escape_word(w: u64) -> u64 {
    let lt21 = has_byte_lt(w, 0x21);
    let sp = has_byte_eq(w, 0x22)
        | has_byte_eq(w, 0x3C)
        | has_byte_eq(w, 0x3E)
        | has_byte_eq(w, 0x5C)
        | has_byte_eq(w, 0x5E)
        | has_byte_eq(w, 0x60)
        | has_byte_eq(w, 0x7B)
        | has_byte_eq(w, 0x7C)
        | has_byte_eq(w, 0x7D);
    lt21 | sp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_replicates_byte() {
        assert_eq!(broadcast(0x00), 0);
        assert_eq!(broadcast(0xFF), !0);
        assert_eq!(broadcast(0x01), ONES);
        assert_eq!(broadcast(0x80), HIGH_BITS);
    }

    #[test]
    fn has_byte_eq_finds_match() {
        let w = u64::from_le_bytes([b'a', b'b', b'c', b'.', b'e', b'f', b'g', b'h']);
        assert!(has_byte_eq(w, b'.') != 0);
        assert!(has_byte_eq(w, b'.') == 0x80 << 24);
        assert_eq!(has_byte_eq(w, b'z'), 0);
    }

    #[test]
    fn has_byte_lt_filters_high_bytes() {
        let w = u64::from_le_bytes([0x05, 0x80, 0x10, 0xFF, 0x00, 0x20, 0x21, 0x7F]);
        let m = has_byte_lt(w, 0x21);
        // bytes < 0x21 with high bit unset: 0x05, 0x10, 0x00, 0x20 → indices 0, 2, 4, 5
        let positions: Vec<u32> = (0..8u32)
            .filter(|i| (m >> (i * 8 + 7)) & 1 == 1)
            .collect();
        assert_eq!(positions, vec![0, 2, 4, 5]);
    }

    #[test]
    fn has_byte_in_range_alpha() {
        let w = u64::from_le_bytes([b'A', b'Z', b'a', b'z', b'0', b'_', b'@', b'`']);
        let m = has_byte_in_range(w | broadcast(0x20), b'a', b'z');
        let positions: Vec<u32> = (0..8u32)
            .filter(|i| (m >> (i * 8 + 7)) & 1 == 1)
            .collect();
        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn first_set_byte_index_works() {
        assert_eq!(first_set_byte_index(0x80), 0);
        assert_eq!(first_set_byte_index(0x80 << 24), 3);
        assert_eq!(first_set_byte_index(0x80 << 56), 7);
    }

    #[test]
    fn blankid_scan_all_valid() {
        assert_eq!(blankid_body_scan(b""), BodyScan::AllValidAscii);
        assert_eq!(blankid_body_scan(b"abc"), BodyScan::AllValidAscii);
        assert_eq!(blankid_body_scan(b"abcdefgh"), BodyScan::AllValidAscii);
        assert_eq!(blankid_body_scan(b"abc.def"), BodyScan::AllValidAscii);
        assert_eq!(
            blankid_body_scan(b"ABCdefgh01234.-_:abcdefghij"),
            BodyScan::AllValidAscii
        );
    }

    #[test]
    fn blankid_scan_invalid() {
        assert_eq!(blankid_body_scan(b"abc!def"), BodyScan::InvalidAt(3));
        assert_eq!(
            blankid_body_scan(b"abcdefgh!"),
            BodyScan::InvalidAt(8)
        );
        assert_eq!(blankid_body_scan(b" "), BodyScan::InvalidAt(0));
    }

    #[test]
    fn blankid_scan_non_ascii() {
        let body = "abcé".as_bytes();
        assert_eq!(blankid_body_scan(body), BodyScan::NonAsciiAt(3));
        let body = "abcdefghé".as_bytes();
        assert_eq!(blankid_body_scan(body), BodyScan::NonAsciiAt(8));
        let body = "é".as_bytes();
        assert_eq!(blankid_body_scan(body), BodyScan::NonAsciiAt(0));
    }

    #[test]
    fn iri_escape_word_clean_ascii() {
        let w = u64::from_le_bytes(*b"abcdefgh");
        assert_eq!(iri_escape_word(w), 0);
    }

    #[test]
    fn iri_escape_word_finds_specials() {
        for &b in &[
            b'<', b'>', b'"', b'{', b'}', b'|', b'^', b'`', b'\\', b' ', b'\t', 0x00, 0x1F,
            0x20,
        ] {
            let arr = [b'a', b, b'a', b'a', b'a', b'a', b'a', b'a'];
            let w = u64::from_le_bytes(arr);
            let mask = iri_escape_word(w);
            assert_ne!(mask, 0, "byte {b:#x}");
            assert_eq!(first_set_byte_index(mask), 1, "byte {b:#x}");
        }
    }

    #[test]
    fn iri_escape_word_passes_high_bytes() {
        // UTF-8 multi-byte bytes (>= 0x80) are never escapes.
        let arr = [0xC3, 0xA9, 0xE4, 0xB8, 0xAD, 0xF0, 0x9F, 0xA6];
        let w = u64::from_le_bytes(arr);
        assert_eq!(iri_escape_word(w), 0);
    }

    #[test]
    fn iri_escape_word_multiple_in_chunk() {
        // Escapes at positions 1 and 4: should set both high bits.
        let arr = [b'a', b'<', b'b', b'c', b'>', b'd', b'e', b'f'];
        let w = u64::from_le_bytes(arr);
        let mask = iri_escape_word(w);
        assert_ne!(mask, 0);
        let first = first_set_byte_index(mask);
        let cleared = mask & (mask - 1);
        let second = first_set_byte_index(cleared);
        assert_eq!(first, 1);
        assert_eq!(second, 4);
    }
}
