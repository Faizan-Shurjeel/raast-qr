#![cfg_attr(not(feature = "std"), no_std)]

//! # raast-qr
//!
//! A high-performance, `#![no_std]`-compatible, fail-closed EMVCo Merchant-Presented Mode (MPM)
//! QR engine tailored for Pakistan's payment rails and the **State Bank of Pakistan (SBP) Raast**
//! instant payment specifications.
//!
//! ## Quick Start Example
//!
//! ```rust
//! # #[cfg(any(feature = "std", feature = "alloc"))]
//! # fn main() {
//! use raast_qr::{Currency, InitiationMethod, RaastQr};
//! use rust_decimal_macros::dec;
//!
//! let qr = RaastQr::builder()
//!     .initiation_method(InitiationMethod::Dynamic)
//!     .raast_alias("+923367865823")
//!     .merchant_name("Faizan Shurjeel")
//!     .merchant_city("Lahore")
//!     .mcc("5411")
//!     .amount(dec!(1250.50))
//!     .bill_reference("INV-2026-001")
//!     .build_emv_string()
//!     .expect("Building must succeed");
//!
//! let parsed = RaastQr::parse(&qr).expect("Parsing must succeed");
//! assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
//! assert_eq!(parsed.currency, Currency::PKR);
//! assert_eq!(parsed.amount, Some(dec!(1250.50)));
//! # }
//! # #[cfg(not(any(feature = "std", feature = "alloc")))]
//! # fn main() {}
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod crc;
pub mod error;
pub mod raast;
pub mod tlv;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod builder;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use builder::RaastQrBuilder;
pub use error::RaastError;
pub use raast::{Currency, Fee, InitiationMethod, RaastQr};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crc::{compute_crc16, format_crc};
    use proptest::prelude::*;
    #[cfg(any(feature = "std", feature = "alloc"))]
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::{format, string::String, vec::Vec};

    #[test]
    fn test_readme_static_example_string_is_valid() {
        let raw = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";
        let parsed = RaastQr::parse(raw).expect("README example string must parse cleanly");
        assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
        assert_eq!(parsed.raast_id, "+923367865823");
        assert_eq!(parsed.amount, Some(dec!(1250.50)));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
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

        let parsed =
            RaastQr::parse(&emv_string).expect("Parsing generated EMV string should succeed");

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

        let parsed_other =
            RaastQr::parse_with_guid(&payload, "other").expect("Should parse specific scheme");
        assert_eq!(parsed_other.mai_tag, "26");
        assert_eq!(parsed_other.scheme_guid, "other");
    }

    #[test]
    fn test_mai_smuggling_multiple_preferred_guids_rejected() {
        let prefix = "00020101021126290008pk.raast0113+92336786582328290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304";
        let crc = compute_crc16(prefix.as_bytes());
        let crc_bytes = format_crc(crc);
        let payload = format!("{}{}", prefix, core::str::from_utf8(&crc_bytes).unwrap());

        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag(
                "26..=51 (Multiple MAI tags claim preferred scheme GUID)"
            ))
        );
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn test_sub_paisa_amount_rejected_by_builder() {
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

        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag("54"))
        );
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
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

    #[cfg(any(feature = "std", feature = "alloc"))]
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

    fn with_crc(prefix: &str) -> String {
        let crc = compute_crc16(prefix.as_bytes());
        let crc_bytes = format_crc(crc);
        format!(
            "{}{}",
            prefix,
            core::str::from_utf8(&crc_bytes).unwrap_or("0000")
        )
    }

    #[test]
    fn test_scientific_notation_amount_rejected_on_parse() {
        let payload = with_crc("00020101021226290008pk.raast0113+92336786582352045411530358654031E45802PK5906Faizan6006Lahore6304");
        assert!(matches!(
            RaastQr::parse(&payload),
            Err(RaastError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_sub_paisa_amount_rejected_on_parse() {
        let payload = with_crc("00020101021226290008pk.raast0113+923367865823520454115303586540610.0055802PK5906Faizan6006Lahore6304");
        assert!(matches!(
            RaastQr::parse(&payload),
            Err(RaastError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_signed_amount_rejected_on_parse() {
        let payload = with_crc("00020101021226290008pk.raast0113+9233678658235204541153035865404+1005802PK5906Faizan6006Lahore6304");
        assert!(matches!(
            RaastQr::parse(&payload),
            Err(RaastError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_leading_zero_amount_rejected_on_parse() {
        let payload = with_crc("00020101021226290008pk.raast0113+92336786582352045411530358654070100.505802PK5906Faizan6006Lahore6304");
        assert!(matches!(
            RaastQr::parse(&payload),
            Err(RaastError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_plain_two_decimal_amount_still_accepted() {
        let payload = with_crc("00020101021226290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304");
        let parsed = RaastQr::parse(&payload).expect("canonical amount must parse");
        assert_eq!(parsed.amount, Some(dec!(100.00)));
    }

    #[test]
    fn test_foreign_scheme_guid_rejected() {
        let payload = with_crc("00020101021226290008com.evil0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304");
        assert!(matches!(
            RaastQr::parse(&payload),
            Err(RaastError::MissingMandatoryTag(_))
        ));
    }

    #[test]
    fn test_duplicate_mai_sub_tag_01_rejected() {
        let payload = with_crc("00020101021226460008pk.raast0113+9233678658230113+9299911122225204541153035865406100.005802PK5906Faizan6006Lahore6304");
        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag("MAI Sub-tag 01 (Raast ID)"))
        );
    }

    #[test]
    fn test_duplicate_mai_sub_tag_00_cannot_steer_guid_selection() {
        let payload = with_crc("00020101021226380005other0008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304");
        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag("MAI Sub-tag 00 (GUID)"))
        );
    }

    #[test]
    fn test_duplicate_top_level_tag_62_rejected() {
        let payload = with_crc("00020101021226290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore62160112INV-2026-00162160112INV-2026-0016304");
        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag("62"))
        );
    }

    #[test]
    fn test_duplicate_tag_00_rejected() {
        let payload = with_crc("00020100020101021226290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304");
        assert_eq!(
            RaastQr::parse(&payload),
            Err(RaastError::DuplicateTag("00"))
        );
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn test_builder_rejects_non_pk_country() {
        let res = RaastQr::builder()
            .raast_alias("+923367865823")
            .merchant_name("Faizan")
            .merchant_city("Lahore")
            .country_code("US")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::MalformedTlv(_))));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn test_builder_rejects_oversized_scheme_guid() {
        let res = RaastQr::builder()
            .scheme_guid("a".repeat(33))
            .raast_alias("+923367865823")
            .merchant_name("Faizan")
            .merchant_city("Lahore")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::FieldLengthExceeded(_))));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    proptest! {
        #[test]
        fn prop_builder_output_always_parses(
            name in "[A-Za-z][A-Za-z ]{0,24}",
            city in "[A-Za-z]{1,15}",
            digits in "[0-9]{10,12}",
            paisa in 1i64..99_999_999i64,
        ) {
            let amount = Decimal::new(paisa, 2);
            let built = RaastQr::builder()
                .initiation_method(InitiationMethod::Dynamic)
                .raast_alias(format!("+92{}", digits))
                .merchant_name(name.clone())
                .merchant_city(city.clone())
                .mcc("5411")
                .amount(amount)
                .build_emv_string();

            if let Ok(emv) = built {
                let parsed = RaastQr::parse(&emv)
                    .expect("builder output must always satisfy the parser");
                prop_assert_eq!(parsed.merchant_name, name.as_str());
                prop_assert_eq!(parsed.merchant_city, city.as_str());
                prop_assert_eq!(parsed.amount, Some(amount));
            }
        }
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
