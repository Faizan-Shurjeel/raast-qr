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

        // Verify the generated string parses back to identical values
        let parsed = RaastQr::parse(&emv_string).expect("Parsing generated EMV string should succeed");

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
    fn test_dynamic_qr_without_amount_fails() {
        let res = RaastQr::builder()
            .initiation_method(InitiationMethod::Dynamic)
            .raast_alias("+923367865823")
            .merchant_name("Faizan")
            .merchant_city("Lahore")
            .build_emv_string();

        assert!(matches!(res, Err(RaastError::InvalidAmount(_))));
    }

    #[test]
    fn test_tampered_payload_rejected_by_parse() {
        let mut valid_emv = RaastQr::builder()
            .initiation_method(InitiationMethod::Static)
            .raast_alias("+923367865823")
            .merchant_name("Store")
            .merchant_city("Karachi")
            .build_emv_string()
            .unwrap();

        // Mutate merchant name from 'Store' to 'Score'
        valid_emv = valid_emv.replace("Store", "Score");

        let res = RaastQr::parse(&valid_emv);
        assert!(matches!(res, Err(RaastError::InvalidChecksum { .. })));
    }
}
