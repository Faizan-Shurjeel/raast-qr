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
    Static,
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
    PKR, // 586
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
    pub initiation_method: InitiationMethod,
    /// Merchant Account Information Tag (in the range 26..=51)
    pub mai_tag: &'a str,
    /// Scheme Identifier / GUID (Sub-tag 00 of MAI)
    pub scheme_guid: &'a str,
    /// Merchant Raast ID / Alias / IBAN (Sub-tag 01 of MAI)
    pub raast_id: &'a str,
    /// Optional Bank / Participant Code (Sub-tag 02 of MAI)
    pub bank_code: Option<&'a str>,
    pub mcc: &'a str,
    pub currency: Currency,
    pub amount: Option<Decimal>,
    pub country_code: &'a str,
    pub merchant_name: &'a str,
    pub merchant_city: &'a str,
    pub bill_reference: Option<&'a str>,
}

impl<'a> RaastQr<'a> {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn builder() -> RaastQrBuilder {
        RaastQrBuilder::new()
    }

    /// Parses and validates an EMVCo Raast QR string, preferring "pk.raast" if multiple MAI templates exist.
    pub fn parse(raw: &'a str) -> Result<Self, RaastError> {
        Self::parse_with_guid(raw, "pk.raast")
    }

    /// Parses and validates an EMVCo QR string, preferring `preferred_guid` if multiple MAI templates exist.
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
        let mut seen_tag01 = false;
        let mut seen_mai_mask = 0u64;

        let mut selected_mai_tag: Option<&'a str> = None;
        let mut selected_scheme_guid: Option<&'a str> = None;
        let mut selected_raast_id: Option<&'a str> = None;
        let mut selected_bank_code: Option<&'a str> = None;

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
                    if tlv.value != "01" {
                        return Err(RaastError::MalformedTlv(
                            "unsupported payload format version",
                        ));
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
                        _ => {
                            return Err(RaastError::MalformedTlv(
                                "invalid initiation method (expected 11 or 12)",
                            ))
                        }
                    }
                }
                // Tags 26..=51: Merchant Account Information (MAI)
                tag if (26..=51).contains(&tag.parse::<u8>().unwrap_or(0)) => {
                    let tag_num = tag.parse::<u8>().unwrap();
                    let bit = 1u64 << (tag_num - 26);
                    if (seen_mai_mask & bit) != 0 {
                        return Err(RaastError::DuplicateTag(
                            "26..=51 (Duplicate MAI tag encountered)",
                        ));
                    }
                    seen_mai_mask |= bit;

                    let mut current_guid = None;
                    let mut current_id = None;
                    let mut current_bank = None;

                    for sub in tlv.sub_tlvs() {
                        let sub_tlv = sub?;
                        match sub_tlv.tag {
                            "00" => {
                                if sub_tlv.value.len() > 32 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 00 (GUID exceeds 32 bytes)",
                                    ));
                                }
                                current_guid = Some(sub_tlv.value);
                            }
                            "01" => {
                                if sub_tlv.value.len() > 90 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 01 (Raast ID exceeds 90 bytes)",
                                    ));
                                }
                                current_id = Some(sub_tlv.value);
                            }
                            "02" => {
                                if sub_tlv.value.len() > 20 {
                                    return Err(RaastError::FieldLengthExceeded(
                                        "MAI Sub-tag 02 (Bank code exceeds 20 bytes)",
                                    ));
                                }
                                current_bank = Some(sub_tlv.value);
                            }
                            _ => {}
                        }
                    }

                    // Select this MAI if none selected yet, or if it matches the preferred scheme GUID
                    if selected_mai_tag.is_none() || current_guid == Some(preferred_guid) {
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
                        return Err(RaastError::MalformedTlv(
                            "MCC must be exactly 4 ASCII digits",
                        ));
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
                        return Err(RaastError::FieldLengthExceeded(
                            "54 (Amount exceeds 13 characters)",
                        ));
                    }
                    let dec = Decimal::from_str(tlv.value)
                        .map_err(|_| RaastError::InvalidAmount("cannot parse decimal value"))?;
                    if dec <= Decimal::ZERO {
                        return Err(RaastError::InvalidAmount(
                            "amount must be greater than zero",
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
                            "59 (Merchant Name exceeds 25 bytes)",
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
                            "60 (Merchant City exceeds 15 bytes)",
                        ));
                    }
                    merchant_city = Some(tlv.value);
                }
                "62" => {
                    for sub in tlv.sub_tlvs() {
                        let sub_tlv = sub?;
                        if sub_tlv.tag == "01" {
                            if sub_tlv.value.len() > 25 {
                                return Err(RaastError::FieldLengthExceeded(
                                    "62.01 (Bill reference exceeds 25 bytes)",
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
            "26-51 (Merchant Account Info)",
        ))?;
        let scheme_guid = selected_scheme_guid.ok_or(RaastError::MissingMandatoryTag(
            "MAI Sub-tag 00 (Scheme GUID)",
        ))?;
        let raast_id = selected_raast_id.ok_or(RaastError::MissingMandatoryTag(
            "MAI Sub-tag 01 (Raast ID / Alias)",
        ))?;
        let mcc = mcc.ok_or(RaastError::MissingMandatoryTag("52 (MCC)"))?;
        let currency = currency.ok_or(RaastError::MissingMandatoryTag("53 (Currency 586)"))?;
        let country_code =
            country_code.ok_or(RaastError::MissingMandatoryTag("58 (Country Code)"))?;
        let merchant_name =
            merchant_name.ok_or(RaastError::MissingMandatoryTag("59 (Merchant Name)"))?;
        let merchant_city =
            merchant_city.ok_or(RaastError::MissingMandatoryTag("60 (Merchant City)"))?;

        if initiation_method == InitiationMethod::Dynamic && amount.is_none() {
            return Err(RaastError::InvalidAmount(
                "dynamic initiation method requires an amount",
            ));
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
