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

/// Builder for constructing valid Raast EMVCo QR strings.
#[derive(Debug, Clone)]
pub struct RaastQrBuilder {
    initiation_method: InitiationMethod,
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

    /// Serializes and computes the full EMVCo string including the CRC checksum.
    pub fn build_emv_string(&self) -> Result<String, RaastError> {
        let raast_id = self
            .raast_id
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag("26.01 (Raast ID / Alias)"))?;
        let merchant_name = self
            .merchant_name
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag("59 (Merchant Name)"))?;
        let merchant_city = self
            .merchant_city
            .as_ref()
            .ok_or(RaastError::MissingMandatoryTag("60 (Merchant City)"))?;

        // Fail-Closed Length and Validation Checks
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
                "26.01 (Raast ID exceeds maximum length)",
            ));
        }

        if self.initiation_method == InitiationMethod::Dynamic && self.amount.is_none() {
            return Err(RaastError::InvalidAmount(
                "Dynamic QR requires a positive amount",
            ));
        }

        let mut buf = String::with_capacity(256);

        // Tag 00: Format Indicator
        buf.push_str("000201");

        // Tag 01: Point of Initiation
        write!(buf, "0102{}", self.initiation_method.as_code())
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 26: Raast Merchant Info (Sub-tags: 00=pk.raast, 01=ID, 02=BankCode)
        let mut tag26_sub = String::with_capacity(64);
        tag26_sub.push_str("0008pk.raast");
        write!(tag26_sub, "01{:02}{}", raast_id.len(), raast_id)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        if let Some(ref bank) = self.bank_code {
            if bank.len() > 20 {
                return Err(RaastError::FieldLengthExceeded(
                    "26.02 (Bank code exceeds maximum length)",
                ));
            }
            write!(tag26_sub, "02{:02}{}", bank.len(), bank)
                .map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }
        if tag26_sub.len() > 99 {
            return Err(RaastError::FieldLengthExceeded(
                "26 (Tag 26 payload exceeds 99 bytes)",
            ));
        }
        write!(buf, "26{:02}{}", tag26_sub.len(), tag26_sub)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 52: MCC
        write!(buf, "52{:02}{}", self.mcc.len(), self.mcc)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 53: Currency
        let curr = self.currency.as_code();
        write!(buf, "53{:02}{}", curr.len(), curr)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 54: Amount (if present)
        if let Some(amt) = self.amount {
            if amt <= Decimal::ZERO {
                return Err(RaastError::InvalidAmount(
                    "amount must be greater than zero",
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

        // Tag 58: Country Code
        write!(buf, "58{:02}{}", self.country_code.len(), self.country_code)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 59: Merchant Name
        write!(buf, "59{:02}{}", merchant_name.len(), merchant_name)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 60: Merchant City
        write!(buf, "60{:02}{}", merchant_city.len(), merchant_city)
            .map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 62: Additional Data (Bill Reference)
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

        // Tag 63: Framing & CRC16 Checksum
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
