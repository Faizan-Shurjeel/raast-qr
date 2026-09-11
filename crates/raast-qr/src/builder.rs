//! Ergonomic builder and serializer for SBP Raast QR codes.

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
        let raast_id = self.raast_id.as_ref()
            .ok_or(RaastError::MissingMandatoryTag("26.01 (Raast ID / Alias)"))?;
        let merchant_name = self.merchant_name.as_ref()
            .ok_or(RaastError::MissingMandatoryTag("59 (Merchant Name)"))?;
        let merchant_city = self.merchant_city.as_ref()
            .ok_or(RaastError::MissingMandatoryTag("60 (Merchant City)"))?;

        if self.initiation_method == InitiationMethod::Dynamic && self.amount.is_none() {
            return Err(RaastError::InvalidAmount("Dynamic QR requires a positive amount"));
        }

        let mut buf = String::with_capacity(256);

        // Tag 00: Format Indicator
        buf.push_str("000201");

        // Tag 01: Point of Initiation
        write!(buf, "0102{}", self.initiation_method.as_code()).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 26: Raast Merchant Info (Sub-tags: 00=pk.raast, 01=ID, 02=BankCode)
        let mut tag26_sub = String::with_capacity(64);
        tag26_sub.push_str("0008pk.raast");
        write!(tag26_sub, "01{:02}{}", raast_id.len(), raast_id).map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        if let Some(ref bank) = self.bank_code {
            write!(tag26_sub, "02{:02}{}", bank.len(), bank).map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }
        write!(buf, "26{:02}{}", tag26_sub.len(), tag26_sub).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 52: MCC
        write!(buf, "52{:02}{}", self.mcc.len(), self.mcc).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 53: Currency
        let curr = self.currency.as_code();
        write!(buf, "53{:02}{}", curr.len(), curr).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 54: Amount (if present)
        if let Some(amt) = self.amount {
            let amt_str = format!("{:.2}", amt);
            write!(buf, "54{:02}{}", amt_str.len(), amt_str).map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }

        // Tag 58: Country Code
        write!(buf, "58{:02}{}", self.country_code.len(), self.country_code).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 59: Merchant Name
        write!(buf, "59{:02}{}", merchant_name.len(), merchant_name).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 60: Merchant City
        write!(buf, "60{:02}{}", merchant_city.len(), merchant_city).map_err(|_| RaastError::MalformedTlv("fmt error"))?;

        // Tag 62: Additional Data (Bill Reference)
        if let Some(ref bill_ref) = self.bill_reference {
            let mut tag62_sub = String::with_capacity(32);
            write!(tag62_sub, "01{:02}{}", bill_ref.len(), bill_ref).map_err(|_| RaastError::MalformedTlv("fmt error"))?;
            write!(buf, "62{:02}{}", tag62_sub.len(), tag62_sub).map_err(|_| RaastError::MalformedTlv("fmt error"))?;
        }

        // Tag 63: Framing & CRC16 Checksum
        buf.push_str("6304");
        let crc = compute_crc16(buf.as_bytes());
        let crc_bytes = format_crc(crc);
        buf.push_str(core::str::from_utf8(&crc_bytes).unwrap());

        Ok(buf)
    }
}
