//! Ergonomic, panic-free builder and serializer for SBP Raast QR codes.

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use core::fmt::Write as CoreWrite;
use rust_decimal::Decimal;

use crate::crc::{compute_crc16, format_crc};
use crate::error::RaastError;
use crate::raast::{Currency, InitiationMethod};

#[derive(Debug, Clone)]
pub struct RaastQrBuilder {
    initiation_method: InitiationMethod,
    mai_tag: String,
    scheme_guid: String,
    raast_id: Option<String>,
    bank_code: Option<String>,
    mcc: String,
    currency: Currency,
    amount: Option<Decimal>,
    country_code: String,
    merchant_name: Option<String>,
    merchant_city: Option<String>,
    bill_reference: Option<String>,
}

impl Default for RaastQrBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RaastQrBuilder {
    pub fn new() -> Self {
        Self {
            initiation_method: InitiationMethod::Static,
            mai_tag: "26".to_string(),
            scheme_guid: "pk.raast".to_string(),
            raast_id: None,
            bank_code: None,
            mcc: "0000".to_string(),
            currency: Currency::PKR,
            amount: None,
            country_code: "PK".to_string(),
            merchant_name: None,
            merchant_city: None,
            bill_reference: None,
        }
    }

    pub fn initiation_method(mut self, method: InitiationMethod) -> Self {
        self.initiation_method = method;
        self
    }

    pub fn mai_tag(mut self, tag: impl Into<String>) -> Self {
        self.mai_tag = tag.into();
        self
    }

    pub fn scheme_guid(mut self, guid: impl Into<String>) -> Self {
        self.scheme_guid = guid.into();
        self
    }

    pub fn raast_alias(mut self, alias: impl Into<String>) -> Self {
        self.raast_id = Some(alias.into());
        self
    }

    pub fn raast_iban(mut self, iban: impl Into<String>) -> Self {
        self.raast_id = Some(iban.into());
        self
    }

    pub fn bank_code(mut self, code: impl Into<String>) -> Self {
        self.bank_code = Some(code.into());
        self
    }

    pub fn mcc(mut self, mcc: impl Into<String>) -> Self {
        self.mcc = mcc.into();
        self
    }

    pub fn currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    pub fn amount(mut self, amount: Decimal) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn country_code(mut self, code: impl Into<String>) -> Self {
        self.country_code = code.into();
        self
    }

    pub fn merchant_name(mut self, name: impl Into<String>) -> Self {
        self.merchant_name = Some(name.into());
        self
    }

    pub fn merchant_city(mut self, city: impl Into<String>) -> Self {
        self.merchant_city = Some(city.into());
        self
    }

    pub fn bill_reference(mut self, reference: impl Into<String>) -> Self {
        self.bill_reference = Some(reference.into());
        self
    }

    pub fn build_emv_string(&self) -> Result<String, RaastError> {
        let raast_id = self
            .raast_id
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag(
                "MAI Sub-tag 01 (Raast ID / Alias)",
            ))?;
        let merchant_name = self
            .merchant_name
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag("59 (Merchant Name)"))?;
        let merchant_city = self
            .merchant_city
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag("60 (Merchant City)"))?;

        if merchant_name.len() > 25 {
            return Err(RaastError::FieldLengthExceeded(
                "59 (Merchant Name exceeds 25 bytes)",
            ));
        }
        if merchant_city.len() > 15 {
            return Err(RaastError::FieldLengthExceeded(
                "60 (Merchant City exceeds 15 bytes)",
            ));
        }
        if self.mcc.len() != 4 || !self.mcc.chars().all(|c| c.is_ascii_digit()) {
            return Err(RaastError::MalformedTlv(
                "MCC must be exactly 4 ASCII digits",
            ));
        }
        if self.country_code.len() != 2 {
            return Err(RaastError::FieldLengthExceeded(
                "58 (Country code must be 2 bytes)",
            ));
        }
        if raast_id.len() > 90 {
            return Err(RaastError::FieldLengthExceeded(
                "MAI Sub-tag 01 (Raast ID exceeds maximum length)",
            ));
        }

        if self.mai_tag.len() != 2 {
            return Err(RaastError::MalformedTlv(
                "MAI Tag must be exactly 2 ASCII digits",
            ));
        }
        let tag_num = self.mai_tag.parse::<u8>().unwrap_or(0);
        if !(26..=51).contains(&tag_num) {
            return Err(RaastError::MalformedTlv(
                "MAI Tag must be in numeric range 26..=51",
            ));
        }

        if self.initiation_method == InitiationMethod::Dynamic && self.amount.is_none() {
            return Err(RaastError::InvalidAmount(
                "Dynamic QR requires a positive amount",
            ));
        }

        let mut buf = String::with_capacity(256);

        buf.push_str("000201");
        write!(buf, "0102{}", self.initiation_method.as_code())
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        let mut mai_sub = String::with_capacity(64);
        write!(
            mai_sub,
            "00{:02}{}",
            self.scheme_guid.len(),
            self.scheme_guid
        )
        .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        write!(mai_sub, "01{:02}{}", raast_id.len(), raast_id)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        if let Some(ref bank) = self.bank_code {
            if bank.len() > 20 {
                return Err(RaastError::FieldLengthExceeded(
                    "MAI Sub-tag 02 (Bank code exceeds maximum length)",
                ));
            }
            write!(mai_sub, "02{:02}{}", bank.len(), bank)
                .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }
        if mai_sub.len() > 99 {
            return Err(RaastError::FieldLengthExceeded(
                "MAI payload exceeds 99 bytes",
            ));
        }
        write!(buf, "{}{:02}{}", self.mai_tag, mai_sub.len(), mai_sub)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        write!(buf, "52{:02}{}", self.mcc.len(), self.mcc)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        let curr = self.currency.as_code();
        write!(buf, "53{:02}{}", curr.len(), curr)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        if let Some(amt) = self.amount {
            if amt <= Decimal::ZERO {
                return Err(RaastError::InvalidAmount(
                    "amount must be greater than zero",
                ));
            }
            // Strict Financial Invariant: Reject sub-paisa amounts (scale > 2)
            if amt.scale() > 2 {
                return Err(RaastError::InvalidAmount(
                    "amount cannot have more than 2 decimal places (sub-paisa not supported)",
                ));
            }
            let amt_str = format!("{:.2}", amt);
            if amt_str.len() > 13 {
                return Err(RaastError::FieldLengthExceeded(
                    "54 (Amount exceeds 13 characters)",
                ));
            }
            write!(buf, "54{:02}{}", amt_str.len(), amt_str)
                .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }

        write!(buf, "58{:02}{}", self.country_code.len(), self.country_code)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        write!(buf, "59{:02}{}", merchant_name.len(), merchant_name)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        write!(buf, "60{:02}{}", merchant_city.len(), merchant_city)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        if let Some(ref bill_ref) = self.bill_reference {
            if bill_ref.len() > 25 {
                return Err(RaastError::FieldLengthExceeded(
                    "62.01 (Bill reference exceeds 25 bytes)",
                ));
            }
            let mut tag62_sub = String::with_capacity(32);
            write!(tag62_sub, "01{:02}{}", bill_ref.len(), bill_ref)
                .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
            write!(buf, "62{:02}{}", tag62_sub.len(), tag62_sub)
                .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }

        buf.push_str("6304");
        let crc = compute_crc16(buf.as_bytes());
        let crc_bytes = format_crc(crc);
        for &b in &crc_bytes {
            buf.push(b as char);
        }

        if buf.len() > 512 {
            return Err(RaastError::PayloadTooLong);
        }

        Ok(buf)
    }
}
