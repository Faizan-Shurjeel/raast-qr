//! SBP Raast and EMVCo Merchant-Presented Mode domain models and parser.

use core::str::FromStr;
use rust_decimal::Decimal;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::crc::{compute_crc16, verify_crc};
use crate::error::RaastError;
use crate::tlv::TlvIter;

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::builder::RaastQrBuilder;

/// Point of Initiation Method (Tag 01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum InitiationMethod {
    /// Reusable static QR code (customer or terminal specifies amount).
    Static,
    /// Single-use dynamic QR code (strict, pre-set transaction amount).
    Dynamic,
}

impl InitiationMethod {
    #[inline]
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Static => "11",
            Self::Dynamic => "12",
        }
    }
}

/// Supported ISO 4217 Currency (Tag 53).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Currency {
    /// Pakistani Rupee (Numeric code 586).
    PKR,
}

impl Currency {
    #[inline]
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::PKR => "586",
        }
    }
}

/// Validated EMVCo / SBP Raast QR representation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RaastQr<'a> {
    /// Point of initiation: static or dynamic.
    pub initiation_method: InitiationMethod,
    /// Merchant Account Information Tag (range 26..=51).
    pub mai_tag: &'a str,
    /// Scheme Identifier / GUID (Sub-tag 00 of MAI).
    pub scheme_guid: &'a str,
    /// Merchant Raast ID / Alias / IBAN (Sub-tag 01 of MAI).
    pub raast_id: &'a str,
    /// Optional Bank / Participant Code (Sub-tag 02 of MAI).
    pub bank_code: Option<&'a str>,
    /// Merchant Category Code (Tag 52).
    pub mcc: &'a str,
    /// Transaction Currency (Tag 53, strictly PKR).
    pub currency: Currency,
    /// Transaction Amount in PKR (Tag 54).
    pub amount: Option<Decimal>,
    /// Country Code (Tag 58, strictly PK).
    pub country_code: &'a str,
    /// Merchant Name (Tag 59, max 25 bytes).
    pub merchant_name: &'a str,
    /// Merchant City (Tag 60, max 15 bytes).
    pub merchant_city: &'a str,
    /// Optional Bill / Invoice reference (Tag 62.01).
    pub bill_reference: Option<&'a str>,
}

/// Enforces the EMVCo Tag 54 grammar: one or more digits, optionally followed
/// by `.` and one or two digits.
///
/// Checked *before* `Decimal::from_str` so the accepted amount grammar is
/// defined by this crate rather than by whichever `rust_decimal` minor version
/// the resolver picks: 1.42.0 added a `from_scientific_lossy` fallback to
/// `FromStr` (so `1E4` parses as `10000`), and every version accepts a leading
/// `+`. Neither form can be produced by `builder.rs`.
#[inline]
fn validate_amount_grammar(value: &str) -> Result<(), RaastError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(RaastError::InvalidAmount("54 (amount is empty)"));
    }

    let mut decimal_point: Option<usize> = None;
    for (idx, &b) in bytes.iter().enumerate() {
        match b {
            b'0'..=b'9' => {}
            b'.' if decimal_point.is_none() && idx > 0 && idx + 1 < bytes.len() => {
                decimal_point = Some(idx);
            }
            _ => {
                return Err(RaastError::InvalidAmount(
                    "54 must be plain decimal digits with at most one interior '.' (no sign, no exponent)",
                ));
            }
        }
    }

    if let Some(idx) = decimal_point {
        if bytes.len() - idx - 1 > 2 {
            return Err(RaastError::InvalidAmount(
                "54 cannot have more than 2 decimal places (sub-paisa not supported)",
            ));
        }
    }

    // Canonical form: the builder never emits leading zeros, so reject them to
    // avoid two byte strings mapping to one semantic amount. Relax this if
    // Phase 2 calibration finds a live acquirer that emits them.
    if bytes[0] == b'0' && bytes.len() > 1 && bytes[1] != b'.' {
        return Err(RaastError::InvalidAmount(
            "54 must be canonical (no leading zeros)",
        ));
    }

    Ok(())
}

#[inline]
fn parse_tag_number(tag: &str) -> Option<u8> {
    let b = tag.as_bytes();
    if b.len() == 2 && b[0].is_ascii_digit() && b[1].is_ascii_digit() {
        Some((b[0] - b'0') * 10 + (b[1] - b'0'))
    } else {
        None
    }
}

impl<'a> RaastQr<'a> {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn builder() -> RaastQrBuilder {
        RaastQrBuilder::new()
    }

    /// Parses an EMVCo payload, requiring an MAI template whose scheme GUID is
    /// `"pk.raast"`.
    ///
    /// Payloads carrying only foreign scheme GUIDs are rejected with
    /// [`RaastError::MissingMandatoryTag`] rather than silently adopting the
    /// first MAI template present.
    pub fn parse(raw: &'a str) -> Result<Self, RaastError> {
        Self::parse_with_guid(raw, "pk.raast")
    }

    /// Parses an EMVCo payload, requiring an MAI template whose scheme GUID
    /// equals `preferred_guid`. Exactly one template may claim it.
    pub fn parse_with_guid(raw: &'a str, preferred_guid: &str) -> Result<Self, RaastError> {
        if raw.len() > 512 {
            return Err(RaastError::PayloadTooLong);
        }

        if !verify_crc(raw.as_bytes()) {
            if raw.len() >= 8
                && raw.is_char_boundary(raw.len() - 8)
                && raw.is_char_boundary(raw.len() - 4)
                && &raw[raw.len() - 8..raw.len() - 4] == "6304"
            {
                let data_part = &raw[..raw.len() - 4];
                let expected = compute_crc16(data_part.as_bytes());
                let found = u16::from_str_radix(&raw[raw.len() - 4..], 16).unwrap_or(0);
                return Err(RaastError::InvalidChecksum { expected, found });
            }
            return Err(RaastError::MalformedTlv("invalid or missing CRC framing"));
        }

        if !raw.starts_with("000201") {
            return Err(RaastError::Tag00NotFirst);
        }

        let payload_without_crc = &raw[..raw.len() - 8];
        let iter = TlvIter::new(payload_without_crc);

        let mut initiation_method = InitiationMethod::Static;
        let mut seen_tag00 = false;
        let mut seen_tag01 = false;
        let mut seen_tag62 = false;
        let mut seen_mai_mask = 0u64;

        let mut selected_mai_tag: Option<&'a str> = None;
        let mut selected_scheme_guid: Option<&'a str> = None;
        let mut selected_raast_id: Option<&'a str> = None;
        let mut selected_bank_code: Option<&'a str> = None;
        let mut matched_preferred_count = 0usize;

        let mut mcc: Option<&'a str> = None;
        let mut currency: Option<Currency> = None;
        let mut amount: Option<Decimal> = None;
        let mut country_code: Option<&'a str> = None;
        let mut merchant_name: Option<&'a str> = None;
        let mut merchant_city: Option<&'a str> = None;
        let mut bill_reference: Option<&'a str> = None;

        for tlv_res in iter {
            let tlv = tlv_res?;
            match tlv.tag {
                "00" => {
                    if seen_tag00 {
                        return Err(RaastError::DuplicateTag("00"));
                    }
                    seen_tag00 = true;
                    if tlv.value != "01" {
                        return Err(RaastError::MalformedTlv("unsupported format version"));
                    }
                }
                "01" => {
                    if seen_tag01 {
                        return Err(RaastError::DuplicateTag("01"));
                    }
                    seen_tag01 = true;
                    match tlv.value {
                        "11" => initiation_method = InitiationMethod::Static,
                        "12" => initiation_method = InitiationMethod::Dynamic,
                        _ => return Err(RaastError::MalformedTlv("invalid initiation method")),
                    }
                }
                tag if parse_tag_number(tag).is_some_and(|n| (26..=51).contains(&n)) => {
                    let tag_num = match parse_tag_number(tag) {
                        Some(n) => n,
                        None => continue,
                    };
                    let bit = 1u64 << (tag_num - 26);
                    if (seen_mai_mask & bit) != 0 {
                        return Err(RaastError::DuplicateTag("26..=51 (Duplicate MAI tag)"));
                    }
                    seen_mai_mask |= bit;

                    let mut current_guid = None;
                    let mut current_id = None;
                    let mut current_bank = None;

                    for sub in tlv.sub_tlvs() {
                        let sub_tlv = sub?;
                        match sub_tlv.tag {
                            "00" => {
                                if current_guid.is_some() {
                                    return Err(RaastError::DuplicateTag("MAI Sub-tag 00 (GUID)"));
                                }
                                if sub_tlv.value.len() > 32 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 00 (GUID > 32 bytes)",
                                    ));
                                }
                                current_guid = Some(sub_tlv.value);
                            }
                            "01" => {
                                if current_id.is_some() {
                                    return Err(RaastError::DuplicateTag(
                                        "MAI Sub-tag 01 (Raast ID)",
                                    ));
                                }
                                if sub_tlv.value.len() > 90 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 01 (ID > 90 bytes)",
                                    ));
                                }
                                current_id = Some(sub_tlv.value);
                            }
                            "02" => {
                                if current_bank.is_some() {
                                    return Err(RaastError::DuplicateTag(
                                        "MAI Sub-tag 02 (Bank code)",
                                    ));
                                }
                                if sub_tlv.value.len() > 20 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 02 (Bank code > 20 bytes)",
                                    ));
                                }
                                current_bank = Some(sub_tlv.value);
                            }
                            _ => {}
                        }
                    }

                    if current_guid == Some(preferred_guid) {
                        matched_preferred_count += 1;
                        if matched_preferred_count > 1 {
                            return Err(RaastError::DuplicateTag(
                                "26..=51 (Multiple MAI tags claim preferred scheme GUID)",
                            ));
                        }
                        selected_mai_tag = Some(tag);
                        selected_scheme_guid = current_guid;
                        selected_raast_id = current_id;
                        selected_bank_code = current_bank;
                    }
                }
                "52" => {
                    if mcc.is_some() {
                        return Err(RaastError::DuplicateTag("52"));
                    }
                    if tlv.length != 4 || !tlv.value.chars().all(|c| c.is_ascii_digit()) {
                        return Err(RaastError::MalformedTlv("MCC must be 4 ASCII digits"));
                    }
                    mcc = Some(tlv.value);
                }
                "53" => {
                    if currency.is_some() {
                        return Err(RaastError::DuplicateTag("53"));
                    }
                    if tlv.value != "586" {
                        let code: u16 = tlv.value.parse().unwrap_or(0);
                        return Err(RaastError::UnsupportedCurrency(code));
                    }
                    currency = Some(Currency::PKR);
                }
                "54" => {
                    if amount.is_some() {
                        return Err(RaastError::DuplicateTag("54"));
                    }
                    if tlv.value.len() > 13 {
                        return Err(RaastError::FieldLengthExceeded("54 (Amount > 13 chars)"));
                    }
                    validate_amount_grammar(tlv.value)?;
                    let dec = Decimal::from_str(tlv.value)
                        .map_err(|_| RaastError::InvalidAmount("cannot parse decimal value"))?;
                    if dec <= Decimal::ZERO {
                        return Err(RaastError::InvalidAmount("amount must be > 0"));
                    }
                    if dec.scale() > 2 {
                        return Err(RaastError::InvalidAmount(
                            "amount cannot have more than 2 decimal places (sub-paisa not supported)",
                        ));
                    }
                    amount = Some(dec);
                }
                "58" => {
                    if country_code.is_some() {
                        return Err(RaastError::DuplicateTag("58"));
                    }
                    if tlv.value != "PK" {
                        return Err(RaastError::MalformedTlv("Country code must be PK"));
                    }
                    country_code = Some(tlv.value);
                }
                "59" => {
                    if merchant_name.is_some() {
                        return Err(RaastError::DuplicateTag("59"));
                    }
                    if tlv.value.len() > 25 {
                        return Err(RaastError::FieldLengthExceeded(
                            "59 (Merchant Name > 25 bytes)",
                        ));
                    }
                    merchant_name = Some(tlv.value);
                }
                "60" => {
                    if merchant_city.is_some() {
                        return Err(RaastError::DuplicateTag("60"));
                    }
                    if tlv.value.len() > 15 {
                        return Err(RaastError::FieldLengthExceeded(
                            "60 (Merchant City > 15 bytes)",
                        ));
                    }
                    merchant_city = Some(tlv.value);
                }
                "62" => {
                    if seen_tag62 {
                        return Err(RaastError::DuplicateTag("62"));
                    }
                    seen_tag62 = true;
                    for sub in tlv.sub_tlvs() {
                        let sub_tlv = sub?;
                        if sub_tlv.tag == "01" {
                            if bill_reference.is_some() {
                                return Err(RaastError::DuplicateTag("62.01 (Bill reference)"));
                            }
                            if sub_tlv.value.len() > 25 {
                                return Err(RaastError::FieldLengthExceeded(
                                    "62.01 (Bill ref > 25 bytes)",
                                ));
                            }
                            bill_reference = Some(sub_tlv.value);
                        }
                    }
                }
                "63" => return Err(RaastError::Tag63NotLast),
                _ => {}
            }
        }

        let mai_tag = selected_mai_tag.ok_or(RaastError::MissingMandatoryTag(
            "26-51 (no MAI template matches the requested scheme GUID)",
        ))?;
        let scheme_guid =
            selected_scheme_guid.ok_or(RaastError::MissingMandatoryTag("MAI Sub-tag 00 (GUID)"))?;
        let raast_id =
            selected_raast_id.ok_or(RaastError::MissingMandatoryTag("MAI Sub-tag 01 (ID)"))?;
        let mcc = mcc.ok_or(RaastError::MissingMandatoryTag("52 (MCC)"))?;
        let currency = currency.ok_or(RaastError::MissingMandatoryTag("53 (Currency 586)"))?;
        let country_code = country_code.ok_or(RaastError::MissingMandatoryTag("58 (Country)"))?;
        let merchant_name =
            merchant_name.ok_or(RaastError::MissingMandatoryTag("59 (Merchant Name)"))?;
        let merchant_city =
            merchant_city.ok_or(RaastError::MissingMandatoryTag("60 (Merchant City)"))?;

        if initiation_method == InitiationMethod::Dynamic && amount.is_none() {
            return Err(RaastError::InvalidAmount("dynamic QR requires an amount"));
        }

        Ok(Self {
            initiation_method,
            mai_tag,
            scheme_guid,
            raast_id,
            bank_code: selected_bank_code,
            mcc,
            currency,
            amount,
            country_code,
            merchant_name,
            merchant_city,
            bill_reference,
        })
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        self.initiation_method == InitiationMethod::Dynamic
    }
}
