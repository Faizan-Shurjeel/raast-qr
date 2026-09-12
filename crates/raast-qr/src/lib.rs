#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod crc;
pub mod error;
pub mod raast;
pub mod tlv;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod builder;

pub use error::RaastError;
pub use raast::{Currency, InitiationMethod, RaastQr};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use builder::RaastQrBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

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
        assert_eq!(parsed.raast_id, "+923367865823");
        assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
        assert_eq!(parsed.merchant_city, "Lahore");
        assert_eq!(parsed.mcc, "5411");
        assert_eq!(parsed.currency, Currency::PKR);
        assert_eq!(parsed.amount, Some(dec!(1250.50)));
        assert_eq!(parsed.bill_reference, Some("INV-2026-001"));
    }

    #[test]
    fn test_overflow_name_fails_closed() {
        let res = RaastQr::builder()
            .raast_alias("+923367865823")
            .merchant_name("This merchant name is way too long for EMVCo spec")
            .merchant_city("Lahore")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::FieldLengthExceeded(_))));
    }

    #[test]
    fn test_duplicate_tag_fails_closed() {
        // Construct payload with two Tag 54s
        let payload = "00020101021126290008pk.raast0113+9233678658235204541153035865406100.005406200.005802PK5906Faizan6006Lahore6304ABCD";
        assert!(matches!(
            RaastQr::parse(payload),
            Err(RaastError::InvalidChecksum { .. }) | Err(RaastError::DuplicateTag(_))
        ));
    }

    proptest! {
        #[test]
        fn prop_test_parser_never_panics_on_arbitrary_input(s in any::<String>()) {
            let _ = RaastQr::parse(&s);
        }
    }
}
