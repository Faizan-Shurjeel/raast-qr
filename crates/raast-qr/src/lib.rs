#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod crc;
pub mod error;
pub mod tlv;
pub mod raast;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod builder;

pub use error::RaastError;
pub use raast::{Currency, InitiationMethod, RaastQr};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use builder::RaastQrBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crc::{compute_crc16, format_crc};
    use rust_decimal_macros::dec;
    use proptest::prelude::*;

    #[test]
    fn test_readme_static_example_string_is_valid() {
        let raw = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";
        let parsed = RaastQr::parse(raw).expect("README example string must parse cleanly");
        assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
        assert_eq!(parsed.raast_id, "+923367865823");
        assert_eq!(parsed.amount, Some(dec!(1250.50)));
    }

    #[test]
    fn test_builder_and_parse_roundtrip() {
        let emv_string = RaastQr::builder()
            .initiation_method(InitiationMethod::Dynamic)
            .raast_alias("+923367865823")
            .merchant_name("Faizan Shurjeel")
            .merchant_city("Lahore")
            .mcc("5411")
            .amount(dec!(1250.50))
            .bill_reference("INV-2026-001")
            .build_emv_string()
            .expect("Building EMV string should succeed");

        let parsed = RaastQr::parse(&emv_string).expect("Parsing generated EMV string should succeed");

        assert_eq!(parsed.initiation_method, InitiationMethod::Dynamic);
        assert_eq!(parsed.mai_tag, "26");
        assert_eq!(parsed.scheme_guid, "pk.raast");
        assert_eq!(parsed.raast_id, "+923367865823");
        assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
        assert_eq!(parsed.merchant_city, "Lahore");
        assert_eq!(parsed.mcc, "5411");
        assert_eq!(parsed.currency, Currency::PKR);
        assert_eq!(parsed.amount, Some(dec!(1250.50)));
        assert_eq!(parsed.bill_reference, Some("INV-2026-001"));
    }

    #[test]
    fn test_multi_scheme_guid_disambiguation() {
        let prefix = "00020101021126210005other01081234567828290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304";
        let crc = compute_crc16(prefix.as_bytes());
        let crc_bytes = format_crc(crc);
        let payload = format!("{}{}", prefix, core::str::from_utf8(&crc_bytes).unwrap());

        let parsed = RaastQr::parse(&payload).expect("Should parse multi-MAI payload");
        assert_eq!(parsed.mai_tag, "28");
        assert_eq!(parsed.scheme_guid, "pk.raast");

        let parsed_other = RaastQr::parse_with_guid(&payload, "other").expect("Should parse specific scheme");
        assert_eq!(parsed_other.mai_tag, "26");
        assert_eq!(parsed_other.scheme_guid, "other");
    }

    #[test]
    fn test_mai_smuggling_multiple_preferred_guids_rejected() {
        // Tag 26 AND Tag 28 both claiming pk.raast (Smuggling attack)
        let prefix = "00020101021126290008pk.raast0113+92336786582328290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304";
        let crc = compute_crc16(prefix.as_bytes());
        let crc_bytes = format_crc(crc);
        let payload = format!("{}{}", prefix, core::str::from_utf8(&crc_bytes).unwrap());

        assert_eq!(RaastQr::parse(&payload), Err(RaastError::DuplicateTag("26..=51 (Multiple MAI tags claim preferred scheme GUID)")));
    }

    #[test]
    fn test_sub_paisa_amount_rejected_by_builder() {
        // scale 3 (0.005 PKR) must be rejected
        let res = RaastQr::builder()
            .raast_alias("+923367865823")
            .merchant_name("Faizan")
            .merchant_city("Lahore")
            .amount(dec!(10.005))
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::InvalidAmount(_))));
    }

    #[test]
    fn test_duplicate_tag_fails_closed_with_exact_variant() {
        let prefix = "00020101021126290008pk.raast0113+9233678658235204541153035865406100.005406200.005802PK5906Faizan6006Lahore6304";
        let crc = compute_crc16(prefix.as_bytes());
        let crc_bytes = format_crc(crc);
        let payload = format!("{}{}", prefix, core::str::from_utf8(&crc_bytes).unwrap());

        assert_eq!(RaastQr::parse(&payload), Err(RaastError::DuplicateTag("54")));
    }

    #[test]
    fn test_non_ascii_merchant_name_byte_count_enforced() {
        let urdu_name = "محمد فیضان شرجیل";
        assert!(urdu_name.chars().count() <= 25);
        assert!(urdu_name.len() > 25);

        let res = RaastQr::builder()
            .raast_alias("+923367865823")
            .merchant_name(urdu_name)
            .merchant_city("Lahore")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::FieldLengthExceeded(_))));
    }

    #[test]
    fn test_three_digit_mai_tag_rejected_by_builder() {
        let res = RaastQr::builder()
            .mai_tag("026")
            .raast_alias("+923367865823")
            .merchant_name("Faizan")
            .merchant_city("Lahore")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::MalformedTlv(_))));
    }

    proptest! {
        #[test]
        fn prop_test_parser_never_panics_on_arbitrary_input(s in any::<String>()) {
            let _ = RaastQr::parse(&s);
        }

        #[test]
        fn prop_test_parser_never_panics_on_mutated_payloads(
            mutation_idx in 0usize..150usize,
            mutation_char in any::<char>()
        ) {
            let valid = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";
            let mut chars: Vec<char> = valid.chars().collect();
            let idx = mutation_idx % chars.len();
            chars[idx] = mutation_char;
            let s: String = chars.into_iter().collect();
            let _ = RaastQr::parse(&s);
        }
    }
}
