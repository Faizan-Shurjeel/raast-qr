use raast_qr::{Currency, InitiationMethod, RaastQr};
use rust_decimal_macros::dec;

#[test]
fn test_official_emvco_reference_vector() {
    let raw = "00020101021226290008pk.raast0113+92336786582352045411530358654071250.505802PK5915Faizan Shurjeel6006Lahore62160112INV-2026-00163044A1D";
    let parsed = RaastQr::parse(raw).expect("Reference vector must parse cleanly");

    assert_eq!(parsed.initiation_method, InitiationMethod::Dynamic);
    assert_eq!(parsed.merchant_name, "Faizan Shurjeel");
    assert_eq!(parsed.merchant_city, "Lahore");
    assert_eq!(parsed.currency, Currency::PKR);
    assert_eq!(parsed.amount, Some(dec!(1250.50)));
}
