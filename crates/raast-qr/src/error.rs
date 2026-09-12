//! Fail-closed error types for Raast & EMVCo processing.

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum RaastError {
    #[cfg_attr(
        feature = "std",
        error("Invalid CRC16 checksum: expected {expected:#06X}, found {found:#06X}")
    )]
    InvalidChecksum { expected: u16, found: u16 },

    #[cfg_attr(
        feature = "std",
        error("Payload exceeds maximum EMVCo length (512 bytes)")
    )]
    PayloadTooLong,

    #[cfg_attr(feature = "std", error("Malformed Tag-Length-Value format: {0}"))]
    MalformedTlv(&'static str),

    #[cfg_attr(feature = "std", error("Missing mandatory tag: {0}"))]
    MissingMandatoryTag(&'static str),

    #[cfg_attr(feature = "std", error("Invalid amount decimal formatting: {0}"))]
    InvalidAmount(&'static str),

    #[cfg_attr(
        feature = "std",
        error("Unsupported currency: {0} (only 586 / PKR is supported for Raast)")
    )]
    UnsupportedCurrency(u16),

    #[cfg_attr(
        feature = "std",
        error("Tag 00 (Payload Format Indicator) must be the first data object")
    )]
    Tag00NotFirst,

    #[cfg_attr(feature = "std", error("Tag 63 (CRC) must be the final data object"))]
    Tag63NotLast,

    #[cfg_attr(feature = "std", error("Field value length exceeds EMVCo limits: {0}"))]
    FieldLengthExceeded(&'static str),

    #[cfg_attr(feature = "std", error("Duplicate tag encountered: {0}"))]
    DuplicateTag(&'static str),
}

#[cfg(not(feature = "std"))]
impl fmt::Display for RaastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChecksum { expected, found } => {
                write!(
                    f,
                    "Invalid CRC16 checksum: expected {expected:#06X}, found {found:#06X}"
                )
            }
            Self::PayloadTooLong => write!(f, "Payload exceeds maximum EMVCo length (512 bytes)"),
            Self::MalformedTlv(msg) => write!(f, "Malformed Tag-Length-Value format: {msg}"),
            Self::MissingMandatoryTag(tag) => write!(f, "Missing mandatory tag: {tag}"),
            Self::InvalidAmount(msg) => write!(f, "Invalid amount decimal formatting: {msg}"),
            Self::UnsupportedCurrency(code) => {
                write!(
                    f,
                    "Unsupported currency: {code} (only 586 / PKR is supported for Raast)"
                )
            }
            Self::Tag00NotFirst => write!(f, "Tag 00 (Payload Format Indicator) must be first"),
            Self::Tag63NotLast => write!(f, "Tag 63 (CRC) must be the final data object"),
            Self::FieldLengthExceeded(field) => {
                write!(f, "Field value length exceeds EMVCo limits: {field}")
            }
            Self::DuplicateTag(tag) => write!(f, "Duplicate tag encountered: {tag}"),
        }
    }
}
