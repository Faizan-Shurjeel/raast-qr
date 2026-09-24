use raast_qr::crc::{compute_crc16, format_crc};
use raast_qr::{Fee, InitiationMethod, RaastError, RaastQr};
#[cfg(any(feature = "std", feature = "alloc"))]
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const PREFIX: &str = "00020101021126290008pk.raast0113+923367865823520454115303586";
const SUFFIX: &str = "5802PK5906Faizan6006Lahore6304";

fn with_crc(fields: &str) -> String {
    let prefix = format!("{PREFIX}{fields}{SUFFIX}");
    let crc = format_crc(compute_crc16(prefix.as_bytes()));
    format!("{prefix}{}", core::str::from_utf8(&crc).expect("ASCII CRC"))
}

fn numeric_tag(tag: &str, value: &str) -> String {
    format!("{tag}{:02}{value}", value.len())
}

#[cfg(any(feature = "std", feature = "alloc"))]
fn builder() -> raast_qr::RaastQrBuilder {
    RaastQr::builder()
        .raast_alias("+923367865823")
        .merchant_name("Faizan")
        .merchant_city("Lahore")
        .mcc("5411")
}

#[test]
fn valid_fee_modes_parse_without_base_amount() {
    for (fields, fee) in [
        ("550201".to_owned(), Fee::PromptTip),
        ("55020256040.01".to_owned(), Fee::Fixed(dec!(0.01))),
        ("550202560201".to_owned(), Fee::Fixed(dec!(1))),
        ("55020256021.".to_owned(), Fee::Fixed(dec!(1))),
        (
            "5502025613123456789012.".to_owned(),
            Fee::Fixed(dec!(123456789012)),
        ),
        (
            "55020256139999999999999".to_owned(),
            Fee::Fixed(dec!(9999999999999)),
        ),
        ("55020357040.01".to_owned(), Fee::Percentage(dec!(0.01))),
        ("55020357031.2".to_owned(), Fee::Percentage(dec!(1.2))),
        ("550203570599.99".to_owned(), Fee::Percentage(dec!(99.99))),
        ("550203570399.".to_owned(), Fee::Percentage(dec!(99))),
        // The indicator can occur after its numeric companion.
        ("570599.99550203".to_owned(), Fee::Percentage(dec!(99.99))),
    ] {
        let payload = with_crc(&fields);
        let parsed = RaastQr::parse(&payload).expect("valid fee payload");
        assert_eq!(parsed.initiation_method, InitiationMethod::Static);
        assert_eq!(parsed.amount, None);
        assert_eq!(parsed.fee, Some(fee));
    }
    let no_fee = with_crc("");
    assert_eq!(RaastQr::parse(&no_fee).expect("no fee").fee, None);
}

#[test]
fn invalid_fee_indicators_and_combinations_are_rejected() {
    for fields in [
        "550101",
        "5503012",
        "550204",
        "550200",
        "5500",
        "55020156011",
        "55020157011",
        "550202",
        "550203",
        "56011",
        "57011",
        "55020257011",
        "55020356011",
        "5502025601157011",
        "5502035701156011",
        "550201550201",
        "5502025601156011",
        "5502035701157011",
    ] {
        let payload = with_crc(fields);
        assert!(RaastQr::parse(&payload).is_err(), "accepted {fields}");
    }
    for (fields, tag) in [
        ("550201550201", "55"),
        ("5502025601156011", "56"),
        ("5502035701157011", "57"),
    ] {
        let payload = with_crc(fields);
        assert_eq!(RaastQr::parse(&payload), Err(RaastError::DuplicateTag(tag)));
    }
}

#[test]
fn malformed_fixed_fees_fail_even_with_valid_crc() {
    for value in [
        "0",
        "0.00",
        "0.",
        ".1",
        "1..2",
        "+1",
        "-1",
        "1E2",
        "1,2",
        "1 2",
        "1.234",
        "۱۲",
        "１２",
        "12345678901234",
    ] {
        let fields = format!("550202{}", numeric_tag("56", value));
        let payload = with_crc(&fields);
        assert!(
            RaastQr::parse(&payload).is_err(),
            "accepted fixed fee {value}"
        );
    }
}

#[test]
fn malformed_or_out_of_range_percentages_fail_even_with_valid_crc() {
    for value in [
        "0", "0.00", "0.", "0.001", "100", "100.", "100.00", "99.999", ".5", "1..2", "+1", "-1",
        "1e1", "1,2", "١", "000001",
    ] {
        let fields = format!("550203{}", numeric_tag("57", value));
        let payload = with_crc(&fields);
        assert!(
            RaastQr::parse(&payload).is_err(),
            "accepted percentage {value}"
        );
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn trailing_dot_fees_parse_and_roundtrip_through_builder() {
    for (fields, canonical, fee) in [
        ("55020256021.", "55020256011", Fee::Fixed(dec!(1))),
        ("550203570399.", "550203570299", Fee::Percentage(dec!(99))),
    ] {
        let payload = with_crc(fields);
        let parsed = RaastQr::parse(&payload).expect("trailing-dot fee parses");
        assert_eq!(parsed.fee, Some(fee));
        let rebuilt = builder()
            .fee(parsed.fee.expect("fee is present"))
            .build_emv_string()
            .expect("rebuild parsed fee");
        assert_eq!(rebuilt, with_crc(canonical));
        assert_eq!(RaastQr::parse(&rebuilt).expect("roundtrip").fee, Some(fee));
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn builder_roundtrips_each_fee_without_changing_tag54() {
    for fee in [
        Fee::PromptTip,
        Fee::Fixed(dec!(0.01)),
        Fee::Fixed(dec!(9999999999999)),
        Fee::Percentage(dec!(0.01)),
        Fee::Percentage(dec!(99.99)),
    ] {
        let static_payload = builder().fee(fee).build_emv_string().expect("static fee");
        let parsed = RaastQr::parse(&static_payload).expect("static roundtrip");
        assert_eq!(parsed.fee, Some(fee));
        assert_eq!(parsed.amount, None);

        let dynamic_payload = builder()
            .initiation_method(InitiationMethod::Dynamic)
            .amount(dec!(1250.50))
            .fee(fee)
            .build_emv_string()
            .expect("dynamic fee");
        let parsed = RaastQr::parse(&dynamic_payload).expect("dynamic roundtrip");
        assert_eq!(parsed.fee, Some(fee));
        assert_eq!(parsed.amount, Some(dec!(1250.50)));
    }
    let no_fee = builder().build_emv_string().expect("no fee");
    assert_eq!(no_fee, with_crc(""));

    let reference = builder()
        .initiation_method(InitiationMethod::Dynamic)
        .merchant_name("Faizan Shurjeel")
        .amount(dec!(1250.50))
        .bill_reference("INV-2026-001")
        .build_emv_string()
        .expect("reference QR without a fee");
    assert_eq!(reference, "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D");
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn builder_rejects_invalid_numeric_fees() {
    for fee in [
        Fee::Fixed(Decimal::ZERO),
        Fee::Fixed(dec!(-1)),
        Fee::Fixed(dec!(1.001)),
        Fee::Fixed(dec!(12345678901234)),
        Fee::Percentage(Decimal::ZERO),
        Fee::Percentage(dec!(-1)),
        Fee::Percentage(dec!(0.001)),
        Fee::Percentage(dec!(100)),
        Fee::Percentage(dec!(99.999)),
    ] {
        assert!(
            builder().fee(fee).build_emv_string().is_err(),
            "accepted {fee:?}"
        );
    }
}

#[cfg(feature = "serde")]
#[test]
fn fee_and_qr_implement_serialize() {
    fn assert_serialize<T: serde::Serialize>(_: &T) {}
    let payload = with_crc("55020357040.01");
    let parsed = RaastQr::parse(&payload).expect("valid fee");
    assert_serialize(&Fee::PromptTip);
    assert_serialize(&parsed);
}
