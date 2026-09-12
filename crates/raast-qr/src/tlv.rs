//! Zero-allocation, panic-free Tag-Length-Value (TLV) scanner for EMVCo payloads.

use crate::error::RaastError;

/// Represents a borrowed, validated TLV element within an EMVCo payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTlv<'a> {
    pub tag: &'a str,
    pub length: usize,
    pub value: &'a str,
}

impl<'a> RawTlv<'a> {
    /// Constructs a sub-TLV iterator over the contents of this tag's value.
    #[inline]
    pub fn sub_tlvs(&self) -> TlvIter<'a> {
        TlvIter::new(self.value)
    }
}

/// An iterator that yields `Result<RawTlv<'a>, RaastError>` without allocating memory.
#[derive(Debug, Clone)]
pub struct TlvIter<'a> {
    remaining: &'a str,
}

impl<'a> TlvIter<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Self {
        Self { remaining: input }
    }

    /// Returns the unparsed remainder of the input slice.
    #[inline]
    pub fn remaining(&self) -> &'a str {
        self.remaining
    }
}

impl<'a> Iterator for TlvIter<'a> {
    type Item = Result<RawTlv<'a>, RaastError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        // Must have at least 4 bytes for Tag (2) and Length (2)
        if self.remaining.len() < 4 {
            self.remaining = "";
            return Some(Err(RaastError::MalformedTlv("truncated tag/length header")));
        }

        // Validate character boundaries before slicing to eliminate panic vectors
        if !self.remaining.is_char_boundary(2) || !self.remaining.is_char_boundary(4) {
            self.remaining = "";
            return Some(Err(RaastError::MalformedTlv(
                "header contains non-ASCII characters",
            )));
        }

        let tag = &self.remaining[..2];
        let len_str = &self.remaining[2..4];

        // Length must be two ASCII decimal digits
        let length: usize = match len_str.parse() {
            Ok(val) => val,
            Err(_) => {
                self.remaining = "";
                return Some(Err(RaastError::MalformedTlv("non-numeric length field")));
            }
        };

        // EMVCo lengths must be between 1 and 99
        if length == 0 {
            self.remaining = "";
            return Some(Err(RaastError::MalformedTlv("length cannot be zero")));
        }

        let total_tlv_len = 4 + length;
        if self.remaining.len() < total_tlv_len {
            self.remaining = "";
            return Some(Err(RaastError::MalformedTlv(
                "value length exceeds available bytes",
            )));
        }

        // Ensure slicing at character boundary for valid UTF-8
        if !self.remaining.is_char_boundary(total_tlv_len) {
            self.remaining = "";
            return Some(Err(RaastError::MalformedTlv(
                "value length splits UTF-8 code point",
            )));
        }

        let value = &self.remaining[4..total_tlv_len];
        self.remaining = &self.remaining[total_tlv_len..];

        Some(Ok(RawTlv { tag, length, value }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_tlv_sequence() {
        let input = "00020101021252045411";
        let tlvs: Result<Vec<RawTlv>, RaastError> = TlvIter::new(input).collect();
        let tlvs = tlvs.expect("Parsing should succeed");

        assert_eq!(tlvs.len(), 3);
        assert_eq!(
            tlvs[0],
            RawTlv {
                tag: "00",
                length: 2,
                value: "01"
            }
        );
        assert_eq!(
            tlvs[1],
            RawTlv {
                tag: "01",
                length: 2,
                value: "12"
            }
        );
        assert_eq!(
            tlvs[2],
            RawTlv {
                tag: "52",
                length: 4,
                value: "5411"
            }
        );
    }

    #[test]
    fn test_non_ascii_header_does_not_panic() {
        let input = "🦀0201";
        let mut iter = TlvIter::new(input);
        assert!(matches!(
            iter.next(),
            Some(Err(RaastError::MalformedTlv(_)))
        ));
    }

    #[test]
    fn test_nested_sub_tlv_parsing() {
        let input = "26290008pk.raast0113+923367865823";
        let mut iter = TlvIter::new(input);

        let parent = iter.next().unwrap().unwrap();
        assert_eq!(parent.tag, "26");
        assert_eq!(parent.length, 29);

        let sub_tlvs: Result<Vec<RawTlv>, RaastError> = parent.sub_tlvs().collect();
        let sub_tlvs = sub_tlvs.unwrap();
        assert_eq!(sub_tlvs.len(), 2);
        assert_eq!(
            sub_tlvs[0],
            RawTlv {
                tag: "00",
                length: 8,
                value: "pk.raast"
            }
        );
        assert_eq!(
            sub_tlvs[1],
            RawTlv {
                tag: "01",
                length: 13,
                value: "+923367865823"
            }
        );
    }
}
